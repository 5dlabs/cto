//! Document webhook handler for CTO config sync.
//!
//! Syncs Linear `cto-config.json` documents to Kubernetes `ConfigMaps`
//! for project-specific Play workflow configuration.

use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{Api, ObjectMeta, Patch, PatchParams},
    Client as KubeClient,
};
use std::collections::BTreeMap;
use tracing::{debug, info};

use crate::webhooks::DocumentWebhookData;

/// Namespace where project `ConfigMaps` are created.
const CONFIG_NAMESPACE: &str = "cto";

/// `ConfigMap` name prefix for project configs.
const CONFIG_PREFIX: &str = "cto-config-project";

/// Sync a CTO config document to a Kubernetes `ConfigMap`.
///
/// Creates or updates a `ConfigMap` named `cto-config-project-{project-id}`
/// in the `cto` namespace with the document's JSON content.
///
/// # Arguments
/// * `document` - The document webhook data containing the config content
/// * `project_id` - The Linear project ID to associate the config with
///
/// # Returns
/// The name of the created/updated `ConfigMap`
pub async fn sync_document_to_configmap(
    document: &DocumentWebhookData,
    project_id: &str,
) -> Result<String> {
    // Extract JSON content from the document
    let json_content = extract_json_from_document(document)?;

    // Validate it's valid JSON
    let _: serde_json::Value =
        serde_json::from_str(&json_content).context("Document content is not valid JSON")?;

    // Generate ConfigMap name from project ID
    let configmap_name = format!("{CONFIG_PREFIX}-{}", sanitize_project_id(project_id));

    info!(
        configmap_name = %configmap_name,
        project_id = %project_id,
        document_id = %document.id,
        "Syncing CTO config to ConfigMap"
    );

    // Create Kubernetes client
    let client = KubeClient::try_default()
        .await
        .context("Failed to create Kubernetes client")?;

    let api: Api<ConfigMap> = Api::namespaced(client, CONFIG_NAMESPACE);

    // Build ConfigMap
    let mut data = BTreeMap::new();
    data.insert("cto-config.json".to_string(), json_content);

    // Add metadata about the source
    data.insert(
        "source.json".to_string(),
        serde_json::json!({
            "linearDocumentId": document.id,
            "linearProjectId": project_id,
            "documentUrl": document.url,
            "syncedAt": chrono::Utc::now().to_rfc3339()
        })
        .to_string(),
    );

    // Use JSON merge patch to update only the specific keys without removing others
    // This preserves prd.txt, architecture.md, etc. when syncing cto-config.json
    let patch = serde_json::json!({
        "metadata": {
            "labels": {
                "app.kubernetes.io/name": "cto-config",
                "app.kubernetes.io/component": "project-config",
                "linear.app/project-id": project_id,
                "linear.app/document-id": document.id
            },
            "annotations": {
                "linear.app/document-url": document.url.clone().unwrap_or_default(),
                "cto.5dlabs.ai/synced-at": chrono::Utc::now().to_rfc3339()
            }
        },
        "data": data
    });

    let patch_params = PatchParams::apply("pm-server").force();
    api.patch(&configmap_name, &patch_params, &Patch::Merge(patch))
        .await
        .context("Failed to create/update ConfigMap")?;

    info!(
        configmap_name = %configmap_name,
        namespace = %CONFIG_NAMESPACE,
        "Successfully synced CTO config to ConfigMap"
    );

    Ok(configmap_name)
}

/// Extract JSON content from a Linear document.
///
/// The document content may be:
/// 1. Raw JSON
/// 2. Markdown with JSON in a code fence
fn extract_json_from_document(document: &DocumentWebhookData) -> Result<String> {
    let content = document
        .content
        .as_ref()
        .ok_or_else(|| anyhow!("Document has no content"))?;

    // Try to extract from markdown code fence first
    if let Some(json) = extract_json_from_code_fence(content) {
        debug!("Extracted JSON from code fence");
        return Ok(json);
    }

    // Try to parse as raw JSON
    if content.trim().starts_with('{') {
        debug!("Content appears to be raw JSON");
        return Ok(content.trim().to_string());
    }

    // Last resort: look for JSON-like content anywhere
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            if end > start {
                let potential_json = &content[start..=end];
                // Validate it's actually JSON
                if serde_json::from_str::<serde_json::Value>(potential_json).is_ok() {
                    debug!("Found embedded JSON in document");
                    return Ok(potential_json.to_string());
                }
            }
        }
    }

    Err(anyhow!(
        "Could not extract JSON from document content. Expected raw JSON or markdown with ```json code fence."
    ))
}

/// Extract JSON from a markdown code fence.
///
/// Looks for ```json ... ``` blocks in the content.
fn extract_json_from_code_fence(content: &str) -> Option<String> {
    // Look for ```json block
    let json_fence_start = content.find("```json")?;
    let content_start = json_fence_start + "```json".len();

    // Find the closing fence
    let remaining = &content[content_start..];
    let fence_end = remaining.find("```")?;

    let json_content = remaining[..fence_end].trim();

    if json_content.is_empty() {
        return None;
    }

    Some(json_content.to_string())
}

/// Sanitize a Linear project ID for use in a Kubernetes resource name.
///
/// Linear project IDs are UUIDs, which are already valid for K8s names,
/// but we ensure lowercase and handle any edge cases.
fn sanitize_project_id(project_id: &str) -> String {
    project_id
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Get the `ConfigMap` name for a project.
#[must_use]
pub fn configmap_name_for_project(project_id: &str) -> String {
    format!("{CONFIG_PREFIX}-{}", sanitize_project_id(project_id))
}

/// Store PRD and architecture content in the project `ConfigMap`.
///
/// This is called during initial intake setup to make the workflow
/// Linear-independent. The `ConfigMap` becomes the source of truth for
/// PRD and architecture content.
///
/// # Arguments
/// * `kube_client` - Kubernetes client
/// * `project_id` - The Linear project ID
/// * `prd_content` - The PRD markdown content
/// * `architecture_content` - Optional architecture markdown content
/// * `repository_url` - Optional repository URL (if provided, intake uses existing repo)
///
/// # Returns
/// The name of the updated `ConfigMap`
pub async fn store_intake_content(
    kube_client: &KubeClient,
    project_id: &str,
    prd_content: &str,
    architecture_content: Option<&str>,
    repository_url: Option<&str>,
) -> Result<String> {
    let configmap_name = configmap_name_for_project(project_id);

    info!(
        configmap_name = %configmap_name,
        project_id = %project_id,
        prd_len = prd_content.len(),
        has_arch = architecture_content.is_some(),
        has_repo = repository_url.is_some(),
        "Storing PRD and architecture in project ConfigMap"
    );

    let api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), CONFIG_NAMESPACE);

    // Try to get existing ConfigMap to preserve cto-config.json if present
    let existing_data = match api.get(&configmap_name).await {
        Ok(cm) => cm.data.unwrap_or_default(),
        Err(_) => BTreeMap::new(),
    };

    // Build new data, preserving existing keys
    let mut data = existing_data;
    data.insert("prd.txt".to_string(), prd_content.to_string());

    if let Some(arch) = architecture_content {
        data.insert("architecture.md".to_string(), arch.to_string());
    }

    // Store repository URL if provided (for intake to use existing repo)
    if let Some(repo) = repository_url {
        data.insert("repository_url.txt".to_string(), repo.to_string());
    }

    // Update source metadata
    let source_json = data
        .get("source.json")
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let updated_source = serde_json::json!({
        "linearProjectId": project_id,
        "syncedAt": chrono::Utc::now().to_rfc3339(),
        "hasPrd": true,
        "hasArchitecture": architecture_content.is_some(),
        "repositoryUrl": repository_url,
        // Preserve existing fields
        "linearDocumentId": source_json.get("linearDocumentId"),
        "documentUrl": source_json.get("documentUrl"),
    });
    data.insert("source.json".to_string(), updated_source.to_string());

    let configmap = ConfigMap {
        metadata: ObjectMeta {
            name: Some(configmap_name.clone()),
            namespace: Some(CONFIG_NAMESPACE.to_string()),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/name".to_string(),
                    "cto-config".to_string(),
                ),
                (
                    "app.kubernetes.io/component".to_string(),
                    "project-config".to_string(),
                ),
                ("linear.app/project-id".to_string(), project_id.to_string()),
                ("cto.5dlabs.io/has-prd".to_string(), "true".to_string()),
                (
                    "cto.5dlabs.io/has-architecture".to_string(),
                    architecture_content.is_some().to_string(),
                ),
            ])),
            annotations: Some(BTreeMap::from([(
                "cto.5dlabs.ai/synced-at".to_string(),
                chrono::Utc::now().to_rfc3339(),
            )])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    };

    // Use server-side apply for idempotent create/update
    let patch_params = PatchParams::apply("pm-server").force();
    api.patch(&configmap_name, &patch_params, &Patch::Apply(configmap))
        .await
        .context("Failed to create/update project ConfigMap with PRD/architecture")?;

    info!(
        configmap_name = %configmap_name,
        namespace = %CONFIG_NAMESPACE,
        "Successfully stored PRD and architecture in project ConfigMap"
    );

    Ok(configmap_name)
}

/// Sync architecture document content to the project `ConfigMap`.
///
/// Called when an Architecture document is updated in Linear via webhook.
///
/// # Arguments
/// * `kube_client` - Kubernetes client
/// * `project_id` - The Linear project ID
/// * `content` - The architecture document content
///
/// # Returns
/// The name of the updated `ConfigMap`
pub async fn sync_architecture_to_configmap(
    kube_client: &KubeClient,
    project_id: &str,
    content: &str,
) -> Result<String> {
    let configmap_name = configmap_name_for_project(project_id);

    info!(
        configmap_name = %configmap_name,
        project_id = %project_id,
        content_len = content.len(),
        "Syncing architecture document to project ConfigMap"
    );

    let api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), CONFIG_NAMESPACE);

    // Try to get existing ConfigMap to preserve other keys
    let existing_data = match api.get(&configmap_name).await {
        Ok(cm) => cm.data.unwrap_or_default(),
        Err(_) => BTreeMap::new(),
    };

    // Build new data, preserving existing keys
    let mut data = existing_data;
    data.insert("architecture.md".to_string(), content.to_string());

    // Update source metadata
    let source_json = data
        .get("source.json")
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let updated_source = serde_json::json!({
        "linearProjectId": project_id,
        "syncedAt": chrono::Utc::now().to_rfc3339(),
        "hasArchitecture": true,
        // Preserve existing fields
        "hasPrd": source_json.get("hasPrd"),
        "linearDocumentId": source_json.get("linearDocumentId"),
        "documentUrl": source_json.get("documentUrl"),
    });
    data.insert("source.json".to_string(), updated_source.to_string());

    // Use JSON merge patch to update only specific keys without removing others
    // This preserves prd.txt, cto-config.json, etc. when syncing architecture.md
    let patch = serde_json::json!({
        "metadata": {
            "labels": {
                "app.kubernetes.io/name": "cto-config",
                "app.kubernetes.io/component": "project-config",
                "linear.app/project-id": project_id,
                "cto.5dlabs.io/has-architecture": "true"
            },
            "annotations": {
                "cto.5dlabs.ai/synced-at": chrono::Utc::now().to_rfc3339()
            }
        },
        "data": data
    });

    let patch_params = PatchParams::apply("pm-server").force();
    api.patch(&configmap_name, &patch_params, &Patch::Merge(patch))
        .await
        .context("Failed to sync architecture to project ConfigMap")?;

    info!(
        configmap_name = %configmap_name,
        namespace = %CONFIG_NAMESPACE,
        "Successfully synced architecture to project ConfigMap"
    );

    Ok(configmap_name)
}

/// Intake content read from the project `ConfigMap`.
pub struct IntakeContent {
    /// PRD markdown content
    pub prd: String,
    /// Optional architecture markdown content
    pub architecture: Option<String>,
    /// Optional repository URL (if provided, intake uses existing repo)
    pub repository_url: Option<String>,
}

/// Read PRD and architecture content from the project `ConfigMap`.
///
/// # Arguments
/// * `kube_client` - Kubernetes client
/// * `project_id` - The Linear project ID
///
/// # Returns
/// `IntakeContent` with PRD, architecture, and repository URL
pub async fn read_intake_content(
    kube_client: &KubeClient,
    project_id: &str,
) -> Result<IntakeContent> {
    let configmap_name = configmap_name_for_project(project_id);

    debug!(
        configmap_name = %configmap_name,
        project_id = %project_id,
        "Reading PRD and architecture from project ConfigMap"
    );

    let api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), CONFIG_NAMESPACE);

    let cm = api
        .get(&configmap_name)
        .await
        .context("Project ConfigMap not found")?;

    let data = cm.data.ok_or_else(|| anyhow!("ConfigMap has no data"))?;

    let prd_content = data
        .get("prd.txt")
        .cloned()
        .ok_or_else(|| anyhow!("ConfigMap missing prd.txt"))?;

    let architecture_content = data.get("architecture.md").cloned().filter(|s| !s.is_empty());

    // Read repository URL if stored (for using existing repo instead of creating new)
    let repository_url = data
        .get("repository_url.txt")
        .cloned()
        .filter(|s| !s.is_empty());

    info!(
        configmap_name = %configmap_name,
        prd_len = prd_content.len(),
        has_arch = architecture_content.is_some(),
        has_repo = repository_url.is_some(),
        "Read intake content from project ConfigMap"
    );

    Ok(IntakeContent {
        prd: prd_content,
        architecture: architecture_content,
        repository_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_code_fence() {
        let content = r#"# Config
Some description

```json
{
  "version": "1.0",
  "test": true
}
```

More text
"#;
        let result = extract_json_from_code_fence(content);
        assert!(result.is_some());
        let json = result.unwrap();
        assert!(json.contains("\"version\": \"1.0\""));
    }

    #[test]
    fn test_extract_json_raw() {
        let document = DocumentWebhookData {
            id: "doc-123".to_string(),
            title: "cto-config.json".to_string(),
            content: Some(r#"{"version": "1.0"}"#.to_string()),
            project_id: Some("proj-456".to_string()),
            project: None,
            url: None,
        };

        let result = extract_json_from_document(&document);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_project_id() {
        assert_eq!(sanitize_project_id("abc123-def456"), "abc123-def456");
        assert_eq!(sanitize_project_id("ABC123-DEF456"), "abc123-def456");
    }

    #[test]
    fn test_configmap_name_for_project() {
        // Standard UUID-style project ID (most common case)
        assert_eq!(
            configmap_name_for_project("abc123-def456-789ghi"),
            "cto-config-project-abc123-def456-789ghi"
        );

        // Uppercase should be lowercased
        assert_eq!(
            configmap_name_for_project("ABC123-DEF456"),
            "cto-config-project-abc123-def456"
        );
    }

    /// Test that `ConfigMap` naming in document.rs matches the controller's expectations.
    ///
    /// The controller (resources.rs) mounts the `ConfigMap` using:
    /// ```ignore
    /// format!("cto-config-project-{}", project_id.to_lowercase())
    /// ```
    ///
    /// This test ensures our naming matches that format for valid Linear project IDs.
    /// Linear project IDs are UUIDs, which only contain alphanumerics and hyphens.
    #[test]
    fn test_configmap_naming_matches_controller_expectations() {
        // Simulate the controller's naming logic
        fn controller_configmap_name(project_id: &str) -> String {
            format!("cto-config-project-{}", project_id.to_lowercase())
        }

        // Standard Linear project UUID (most common case)
        let project_id = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
        assert_eq!(
            configmap_name_for_project(project_id),
            controller_configmap_name(project_id),
            "ConfigMap naming should match controller for standard UUIDs"
        );

        // Uppercase UUID (Linear sometimes returns mixed case)
        let project_id = "F47AC10B-58CC-4372-A567-0E02B2C3D479";
        assert_eq!(
            configmap_name_for_project(project_id),
            controller_configmap_name(project_id),
            "ConfigMap naming should match controller for uppercase UUIDs"
        );

        // Short alphanumeric IDs
        let project_id = "abc123";
        assert_eq!(
            configmap_name_for_project(project_id),
            controller_configmap_name(project_id),
            "ConfigMap naming should match controller for short IDs"
        );
    }

    // =========================================================================
    // JSON Extraction Edge Case Tests
    // =========================================================================

    #[test]
    fn test_extract_json_from_code_fence_with_extra_whitespace() {
        let content = r#"# Config

Some description with lots of whitespace


```json

{
  "version": "1.0",
  "test": true
}

```

More text
"#;
        let result = extract_json_from_code_fence(content);
        assert!(result.is_some());
        let json = result.unwrap();
        // Should still parse correctly after trimming
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "1.0");
    }

    #[test]
    fn test_extract_json_from_code_fence_multiple_fences() {
        // Should extract from the first ```json fence
        let content = r#"# Config

Here's some bash:
```bash
echo "hello"
```

Here's the JSON config:
```json
{"first": true}
```

And another one:
```json
{"second": true}
```
"#;
        let result = extract_json_from_code_fence(content);
        assert!(result.is_some());
        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        // Should get the first json fence, not the bash or second json
        assert_eq!(parsed["first"], true);
    }

    #[test]
    fn test_extract_json_from_document_invalid_json_in_fence() {
        let document = DocumentWebhookData {
            id: "doc-123".to_string(),
            title: "cto-config.json".to_string(),
            content: Some(
                r"# Config
```json
{ invalid json here
```
"
                .to_string(),
            ),
            project_id: Some("proj-456".to_string()),
            project: None,
            url: None,
        };

        // The extraction succeeds (gets the text from fence)
        // but later JSON validation would fail
        let result = extract_json_from_document(&document);
        // The function extracts raw content, so it may succeed
        // The JSON validation happens at sync_document_to_configmap
        // This tests that invalid content is handled gracefully
        if let Ok(content) = result {
            // If extraction succeeded, JSON parse should fail
            let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&content);
            assert!(parse_result.is_err(), "Invalid JSON should fail to parse");
        }
    }

    #[test]
    fn test_extract_json_from_document_empty_content() {
        let document = DocumentWebhookData {
            id: "doc-123".to_string(),
            title: "cto-config.json".to_string(),
            content: None,
            project_id: Some("proj-456".to_string()),
            project: None,
            url: None,
        };

        let result = extract_json_from_document(&document);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no content"));
    }

    #[test]
    fn test_extract_json_from_document_embedded_in_prose() {
        // JSON can be found even when embedded in prose text
        let document = DocumentWebhookData {
            id: "doc-123".to_string(),
            title: "cto-config.json".to_string(),
            content: Some(
                r#"This is a document with embedded JSON.
Here it is: {"version": "1.0", "embedded": true}
And some more text after.
"#
                .to_string(),
            ),
            project_id: Some("proj-456".to_string()),
            project: None,
            url: None,
        };

        let result = extract_json_from_document(&document);
        assert!(result.is_ok());
        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["embedded"], true);
    }

    #[test]
    fn test_extract_json_from_code_fence_empty_fence() {
        let content = r"# Config

```json
```

Text after empty fence
";
        let result = extract_json_from_code_fence(content);
        // Empty fence should return None
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_json_from_document_no_json_anywhere() {
        let document = DocumentWebhookData {
            id: "doc-123".to_string(),
            title: "cto-config.json".to_string(),
            content: Some("Just plain text with no JSON at all.".to_string()),
            project_id: Some("proj-456".to_string()),
            project: None,
            url: None,
        };

        let result = extract_json_from_document(&document);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not extract JSON"));
    }
}
