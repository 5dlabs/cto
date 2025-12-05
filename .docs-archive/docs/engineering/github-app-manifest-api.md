# GitHub App Manifest API Integration



## Overview

This document details how to programmatically create GitHub Apps using the Manifest API, which is crucial for the Agent Persona Creator tool.

## GitHub App Manifest Flow

### Standard Flow (Web-based)


1. User clicks "Create GitHub App" button


2. Redirected to GitHub with manifest in URL


3. User reviews and approves


4. GitHub creates app and redirects back with temporary code


5. Exchange code for app credentials



### Programmatic Flow (Our Approach)
Since we want full automation, we'll use a hybrid approach:


1. Use GitHub REST API with PAT/OAuth for org app creation


2. Or use pre-registered "App Factory" app to create child apps


3. Store credentials securely in K8s secrets

## Manifest API Details

### Endpoint for Manifest Creation





```bash
# Redirect URL format (requires user interaction)
https://github.com/organizations/{org}/settings/apps/new?state={state}&manifest={manifest}

# For user-owned apps
https://github.com/settings/apps/new?state={state}&manifest={manifest}








```

### Full Manifest Schema





```json
{
  "name": "Agent-Morgan-Security",
  "url": "https://github.com/5dlabs/cto",
  "description": "AI Security Analyst - Automated vulnerability detection and code review",
  "hook_attributes": {
    "url": "https://webhooks.5dlabs.com/agents/morgan",
    "active": true,
    "events": [
      "pull_request",
      "pull_request_review",
      "pull_request_review_comment",
      "issues",
      "issue_comment",
      "push",
      "check_suite",
      "check_run",
      "workflow_job",
      "workflow_run"
    ]
  },
  "redirect_url": "https://platform.5dlabs.com/agents/setup-complete",
  "callback_urls": [
    "https://platform.5dlabs.com/oauth/callback"
  ],
  "request_oauth_on_install": false,
  "setup_on_update": true,
  "public": false,
  "default_permissions": {
    "actions": "read",
    "checks": "write",
    "contents": "read",
    "deployments": "read",
    "issues": "write",
    "metadata": "read",
    "packages": "read",
    "pages": "read",
    "pull_requests": "write",
    "repository_hooks": "read",
    "repository_projects": "read",
    "security_events": "write",
    "statuses": "write",
    "vulnerability_alerts": "read",
    "workflows": "write"
  },
  "default_events": [
    "check_run",
    "check_suite",
    "issues",
    "issue_comment",
    "pull_request",
    "pull_request_review",
    "pull_request_review_comment",
    "push",
    "workflow_job",
    "workflow_run"
  ]
}








```

## Programmatic App Creation Strategy

### Option 1: Using GitHub REST API (Recommended)





```rust
// Rust implementation for MCP tool
use octocrab::{Octocrab, models::AppId};
use serde_json::json;

async fn create_github_app(
    org: &str,
    manifest: AppManifest,
    pat_token: &str
) -> Result<GitHubApp, Error> {
    // Note: Direct API creation requires GitHub Enterprise
    // or use of OAuth App with proper permissions

    // Step 1: Create manifest URL
    let manifest_json = serde_json::to_string(&manifest)?;
    let encoded_manifest = urlencoding::encode(&manifest_json);

    // Step 2: Use browser automation or OAuth flow
    // This is a limitation - full automation requires Enterprise

    // Alternative: Use pre-existing app to manage others
    create_via_management_app(org, manifest).await
}








```

### Option 2: Management App Pattern

Create a single "Agent Factory" GitHub App with permissions to manage other apps:





```yaml
# Agent Factory App Permissions
administration: write  # Manage apps in org
metadata: read         # Basic access








```

Then use this app to create child apps:





```rust
async fn create_via_management_app(
    org: &str,
    manifest: AppManifest
) -> Result<GitHubApp, Error> {
    let mgmt_client = create_authenticated_client(
        FACTORY_APP_ID,
        FACTORY_PRIVATE_KEY
    )?;

    // Use management endpoints (if available)
    // Or implement custom workflow
}








```

### Option 3: Pre-Registration Pattern

Pre-create a pool of GitHub Apps and assign them dynamically:





```yaml


# Pool of pre-created apps
app_pool:
  - name: agent-pool-001
    status: available
  - name: agent-pool-002
    status: assigned
    assigned_to: Morgan








```

## Implementation Code Structure

### 1. Manifest Builder





```rust
use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub struct AppManifest {
    pub name: String,
    pub url: String,
    pub description: String,
    pub hook_attributes: WebhookConfig,
    pub redirect_url: String,
    pub callback_urls: Vec<String>,
    pub public: bool,
    pub default_permissions: Permissions,
    pub default_events: Vec<String>,
}

impl AppManifest {
    pub fn for_agent(agent: &AgentPersona) -> Self {
        Self {
            name: format!("Agent-{}-{}", agent.name, agent.role),
            url: "https://github.com/5dlabs/cto",
            description: agent.generate_description(),
            hook_attributes: WebhookConfig::for_agent(agent),
            redirect_url: format!("https://platform.5dlabs.com/agents/{}/setup", agent.name.to_lowercase()),
            callback_urls: vec![
                "https://platform.5dlabs.com/oauth/callback".to_string()
            ],
            public: false,
            default_permissions: agent.infer_permissions(),
            default_events: agent.required_events(),
        }
    }
}








```

### 2. Permission Inference Engine





```rust


#[derive(Debug, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_events: Option<String>,
}

impl AgentPersona {
    pub fn infer_permissions(&self) -> Permissions {
        let mut perms = Permissions::default();

        // Analyze purpose keywords
        let purpose_lower = self.purpose.to_lowercase();

        if purpose_lower.contains("review") || purpose_lower.contains("audit") {
            perms.contents = Some("read".to_string());
            perms.pull_requests = Some("write".to_string());
        }

        if purpose_lower.contains("security") || purpose_lower.contains("vulnerability") {
            perms.security_events = Some("write".to_string());
            perms.contents = Some("read".to_string());
        }

        if purpose_lower.contains("test") || purpose_lower.contains("quality") {
            perms.checks = Some("write".to_string());
            perms.actions = Some("read".to_string());
        }

        if purpose_lower.contains("deploy") || purpose_lower.contains("release") {
            perms.deployments = Some("write".to_string());
            perms.packages = Some("write".to_string());
        }

        // Always need metadata
        perms.metadata = Some("read".to_string());

        perms
    }
}








```

### 3. Webhook Configuration





```rust


#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub active: bool,
    pub events: Vec<String>,
    pub content_type: String,
    pub secret: Option<String>,
}

impl WebhookConfig {
    pub fn for_agent(agent: &AgentPersona) -> Self {
        Self {
            url: format!("https://webhooks.5dlabs.com/agents/{}", agent.name.to_lowercase()),
            active: true,
            events: agent.required_events(),
            content_type: "json".to_string(),
            secret: Some(generate_webhook_secret()),
        }
    }
}

fn generate_webhook_secret() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}








```

## API Authentication & Creation Flow

### Step 1: Generate Manifest




```rust
let manifest = AppManifest::for_agent(&agent_persona);
let manifest_json = serde_json::to_string_pretty(&manifest)?;








```

### Step 2: Create App (Requires Manual Step)




```rust
// Generate creation URL
let creation_url = format!(
    "https://github.com/organizations/{}/settings/apps/new?manifest={}",
    org,
    urlencoding::encode(&manifest_json)
);

// Store state for callback
let state = uuid::Uuid::new_v4().to_string();
redis_client.set(&format!("app_creation:{}", state), &agent_persona)?;

// Return URL for manual creation (or automate with browser)
println!("Visit this URL to create the app: {}", creation_url);








```

### Step 3: Handle Callback




```rust
// After app is created, GitHub redirects with code
async fn handle_app_creation_callback(
    code: String,
    state: String
) -> Result<AppCredentials, Error> {
    // Retrieve agent info from state
    let agent_persona = redis_client.get(&format!("app_creation:{}", state))?;

    // Exchange code for app credentials
    let response = client
        .post(&format!("https://api.github.com/app-manifests/{}/conversions", code))
        .send()
        .await?;

    let app_info: AppCredentials = response.json().await?;

    // Store credentials securely
    store_app_credentials(&agent_persona.name, &app_info).await?;

    Ok(app_info)
}








```

## Alternative: GitHub CLI Extension

Create a gh extension for semi-automated creation:





```bash
#!/bin/bash
# gh-create-agent

MANIFEST="$1"
ORG="${2:-5dlabs}"

# Create app using gh api
gh api \


  --method POST \
  -H "Accept: application/vnd.github+json" \
  "/orgs/${ORG}/apps" \


  --input "${MANIFEST}"








```

## Kubernetes Secret Storage

After app creation, store credentials:





```yaml
apiVersion: v1
kind: Secret
metadata:
  name: agent-morgan-github
  namespace: cto
  labels:
    app: morgan
    type: github-app
    created-by: agent-persona-creator
type: Opaque
data:
  app-id: <base64 encoded>
  client-id: <base64 encoded>
  client-secret: <base64 encoded>
  private-key: <base64 encoded>
  webhook-secret: <base64 encoded>
  installation-id: <base64 encoded>








```

## Error Handling

### Common Issues & Solutions



1. **Rate Limiting**
   ```rust
   if response.status() == 429 {
       let retry_after = response.headers()
           .get("Retry-After")
           .and_then(|v| v.to_str().ok())
           .and_then(|v| v.parse::<u64>().ok())
           .unwrap_or(60);

       tokio::time::sleep(Duration::from_secs(retry_after)).await;
       // Retry
   }







```



2. **Duplicate Names**
   ```rust
   // Append timestamp or increment
   let unique_name = format!("{}-{}", base_name, timestamp);







```



3. **Invalid Permissions**
   ```rust
   // Validate against known permissions
   const VALID_PERMISSIONS: &[&str] = &[
       "actions", "checks", "contents", "deployments",
       "issues", "metadata", "packages", "pages",
       "pull_requests", "repository_hooks", "repository_projects",
       "security_events", "statuses", "vulnerability_alerts"
   ];







```

## Testing Approach

### 1. Mock Manifest Generation




```rust


#[cfg(test)]
mod tests {
    #[test]
    fn test_manifest_generation() {
        let agent = AgentPersona {
            name: "TestBot".to_string(),
            purpose: "Testing and validation".to_string(),
            // ...
        };

        let manifest = AppManifest::for_agent(&agent);
        assert_eq!(manifest.name, "Agent-TestBot-Testing");
        assert!(manifest.default_permissions.checks.is_some());
    }
}








```

### 2. Integration Test with GitHub




```rust


#[tokio::test]
async fn test_github_app_creation() {
    // Use test org or sandbox
    let result = create_github_app(
        "test-org",
        test_manifest(),
        &test_token()
    ).await;

    assert!(result.is_ok());
    // Clean up test app
}








```



## Security Best Practices



1. **Never log private keys**


2. **Rotate webhook secrets regularly**


3. **Use least-privilege permissions**


4. **Encrypt manifest in transit**


5. **Validate webhook signatures**


6. **Store credentials in external secrets**


7. **Audit app usage regularly**

## Conclusion

While GitHub doesn't provide a fully programmatic API for app creation (requires user consent), we can:


1. Automate 95% of the process


2. Use a management app pattern for org apps


3. Pre-create app pools for instant assignment


4. Provide excellent UX with minimal manual steps
