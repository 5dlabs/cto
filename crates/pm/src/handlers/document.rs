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
                ("linear.app/document-id".to_string(), document.id.clone()),
            ])),
            annotations: Some(BTreeMap::from([
                (
                    "linear.app/document-url".to_string(),
                    document.url.clone().unwrap_or_default(),
                ),
                (
                    "cto.5dlabs.ai/synced-at".to_string(),
                    chrono::Utc::now().to_rfc3339(),
                ),
            ])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    };

    // Use server-side apply for idempotent create/update
    let patch_params = PatchParams::apply("pm-server").force();
    api.patch(&configmap_name, &patch_params, &Patch::Apply(configmap))
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
