# Headless Visual Testing Specification

## Overview

This specification defines the implementation of headless browser visual testing capabilities for CTO platform agents. This enables agents like Rex to:

1. Render and visually verify UI code they generate
2. Capture screenshots of their work
3. Include visual previews in PR descriptions for reviewers

## Goals

- **Zero Human Interaction**: Entire workflow runs headless in K8s containers
- **Self-Verification**: Agents can validate their UI implementations before submitting PRs
- **Visual Documentation**: PRs include screenshot previews for faster human review
- **Self-Hosted Storage**: In-cluster MinIO for S3-compatible object storage

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CodeRun Job Pod                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐     ┌──────────────────┐     ┌────────────────────────┐  │
│  │  Agent CLI   │────▶│  Tools Server    │────▶│  Playwright MCP        │  │
│  │  (Claude,    │     │  (MCP Proxy)     │     │  (Docker container)    │  │
│  │   Codex)     │     │                  │     │                        │  │
│  └──────────────┘     └──────────────────┘     │  - browser_navigate    │  │
│         │                                       │  - browser_snapshot    │  │
│         │                                       │  - browser_screenshot  │  │
│         │                                       └───────────┬────────────┘  │
│         │                                                   │               │
│         │              ┌──────────────────┐                 │               │
│         │              │  Screenshot      │◀────────────────┘               │
│         │              │  Upload Service  │    (saves to /tmp)              │
│         │              │                  │                                 │
│         │              └────────┬─────────┘                                 │
│         │                       │                                           │
└─────────│───────────────────────│───────────────────────────────────────────┘
          │                       │
          │                       ▼
          │         ┌─────────────────────────┐
          │         │   MinIO (in-cluster)    │
          │         │   minio-screenshots ns  │
          │         │                         │
          │         │   /repo/pr-123/img.png  │
          │         └───────────┬─────────────┘
          │                     │
          │                     │ Presigned URL or Public Endpoint
          │                     ▼
          │         https://screenshots.5dlabs.io/repo/pr-123/img.png
          │         (via Cloudflare Tunnel or Ingress)
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           GitHub PR                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  ## Changes                                                                  │
│  - Implemented login form component                                          │
│                                                                              │
│  ## Visual Preview                                                           │
│  ![Login Form](https://screenshots.5dlabs.io/myrepo/pr-123/login-form.png)  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components

### 1. MinIO Operator & Tenant

**Purpose**: S3-compatible object storage running in-cluster

**Implementation**: MinIO Operator with a lightweight tenant

**ArgoCD Applications**:

- `infra/gitops/applications/minio-operator.yaml` - Operator deployment
- `infra/gitops/applications/minio-tenant.yaml` - Screenshot storage tenant

**Tenant Configuration**:

```yaml
# Single-server, single-drive tenant (minimal footprint)
tenant:
  name: screenshots
  pools:
    - servers: 1
      volumesPerServer: 1
      size: 10Gi
      storageClassName: local-path
  buckets:
    - name: screenshots
```

**Internal Endpoint**: `http://minio.minio-screenshots.svc.cluster.local`

**Console**: `http://screenshots-console.minio-screenshots.svc.cluster.local:9090`

### 2. Playwright MCP Server

**Purpose**: Provides headless browser automation via MCP protocol

**Implementation**: Use Microsoft's official Playwright MCP Docker image

**Configuration** (`infra/charts/tools/values.yaml`):

```yaml
config:
  servers:
    playwright:
      name: "Playwright Browser"
      description: "Headless browser automation for visual testing - navigate, interact, screenshot"
      transport: "stdio"
      command: "docker"
      args:
        - "run"
        - "-i"
        - "--rm"
        - "--init"
        - "--network"
        - "host"
        - "-v"
        - "/tmp/playwright-output:/output"
        - "mcr.microsoft.com/playwright/mcp"
        - "--headless"
        - "--browser"
        - "chromium"
        - "--no-sandbox"
        - "--output-dir"
        - "/output"
        - "--caps=vision"
      workingDirectory: "/tmp"
```

**Key Tools Exposed**:

| Tool | Description | Use Case |
|------|-------------|----------|
| `browser_navigate` | Navigate to URL | Open localhost dev server |
| `browser_snapshot` | Get accessibility tree | Verify elements exist (no vision needed) |
| `browser_take_screenshot` | Capture PNG image | Visual documentation |
| `browser_click` | Click elements | Test interactions |
| `browser_type` | Type into inputs | Test forms |
| `browser_wait_for` | Wait for content | Handle async loading |

### 3. Screenshot Upload Service

**Purpose**: Upload screenshots to MinIO and return public/presigned URLs

**Implementation Options**:

#### Option A: Standalone MCP Server (Recommended)

Create a lightweight MCP server that handles screenshot uploads:

```rust
// tools/src/screenshot/mod.rs

use aws_sdk_s3::Client as S3Client;
use mcp_sdk::{Tool, ToolResult};

pub struct ScreenshotUploader {
    s3_client: S3Client,
    bucket: String,
    public_url_base: String,  // https://screenshots.5dlabs.io
}

impl ScreenshotUploader {
    /// Upload a screenshot file and return public URL
    pub async fn upload_screenshot(
        &self,
        file_path: &str,
        repo: &str,
        pr_number: u32,
        name: &str,
    ) -> Result<String, Error> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let key = format!("{repo}/pr-{pr_number}/{timestamp}-{name}.png");
        
        let body = tokio::fs::read(file_path).await?;
        
        self.s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body.into())
            .content_type("image/png")
            .send()
            .await?;
        
        Ok(format!("{}/{}", self.public_url_base, key))
    }
}

// MCP Tool Definition
pub fn upload_screenshot_tool() -> Tool {
    Tool {
        name: "upload_screenshot",
        description: "Upload a screenshot to MinIO storage and get a public URL for PR embedding",
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the screenshot file (e.g., /tmp/playwright-output/screenshot.png)"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name (e.g., 5dlabs/myproject)"
                },
                "pr_number": {
                    "type": "integer",
                    "description": "Pull request number"
                },
                "name": {
                    "type": "string",
                    "description": "Descriptive name for the screenshot (e.g., login-form, dashboard-view)"
                }
            },
            "required": ["file_path", "repo", "pr_number", "name"]
        }),
    }
}
```

#### Option B: Shell Script via Bash Tool

Simpler alternative using `mc` (MinIO client) or `aws` CLI:

```bash
#!/bin/bash
# scripts/upload-screenshot.sh

FILE_PATH="$1"
REPO="$2"
PR_NUMBER="$3"
NAME="$4"

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
KEY="${REPO}/pr-${PR_NUMBER}/${TIMESTAMP}-${NAME}.png"

# Upload to MinIO using mc (configured with minio alias)
mc cp "$FILE_PATH" "minio/screenshots/${KEY}"

# Output the public URL
echo "https://screenshots.5dlabs.io/${KEY}"
```

### 4. MinIO Storage Configuration

**Bucket Name**: `screenshots`

**Namespace**: `minio-screenshots`

**Storage**: 10Gi via `local-path` StorageClass

**Public Access**: Via Cloudflare Tunnel or Ingress to expose MinIO API

**Lifecycle Policy**: Consider auto-expiration via MinIO ILM (90 days)

**Setup Commands**:

```bash
# After tenant deployment, configure via mc (MinIO Client)
mc alias set minio http://minio.minio-screenshots.svc.cluster.local:80 $ACCESS_KEY $SECRET_KEY

# Create bucket (if not auto-created by tenant)
mc mb minio/screenshots

# Set bucket policy for public read (optional - can use presigned URLs instead)
mc anonymous set download minio/screenshots

# Set lifecycle rule for auto-expiration
mc ilm add --expiry-days 90 minio/screenshots
```

**Secrets Configuration** (`infra/vault/secrets/minio.yaml`):

```yaml
# MinIO tenant root credentials (in minio-screenshots namespace)
vault kv put secret/minio-screenshots-credentials \
  config.env="export MINIO_ROOT_USER=minio-admin
export MINIO_ROOT_PASSWORD=<secure-password>"

# MinIO client credentials for tools server (in cto namespace)
vault kv put secret/tools-minio \
  MINIO_ENDPOINT=http://minio.minio-screenshots.svc.cluster.local:80 \
  MINIO_ACCESS_KEY=screenshots-uploader \
  MINIO_SECRET_KEY=<secure-password> \
  MINIO_BUCKET=screenshots \
  MINIO_REGION=us-east-1
```

### 5. External Access via Cloudflare Tunnel

To make screenshots publicly accessible for GitHub PR embeds:

**Option A: Cloudflare Tunnel (Recommended)**

Add to existing tunnel config (`infra/gitops/resources/cloudflare-tunnel/`):

```yaml
# Add to cluster-tunnel.yaml ingress rules
- hostname: screenshots.5dlabs.io
  service: http://minio.minio-screenshots.svc.cluster.local:80
```

**Option B: Ingress + Cert-Manager**

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: minio-screenshots
  namespace: minio-screenshots
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - screenshots.5dlabs.io
      secretName: minio-screenshots-tls
  rules:
    - host: screenshots.5dlabs.io
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: minio
                port:
                  number: 80
```

### 6. Agent Configuration Updates

**Enable Playwright for agents** (`infra/charts/controller/values.yaml`):

```yaml
agents:
  rex:
    github_app: "5DLabs-Rex"
    cli: "Claude"
    model: "claude-sonnet-4-20250514"
    tools:
      remote:
        - context7
        - github
        - kubernetes
        - argocd
        - playwright        # NEW: Browser automation
        - screenshot-upload # NEW: Upload to MinIO
```

### 7. Agent Prompt Instructions

Add to agent memory/prompt templates:

```markdown
## Visual Verification Capability

You have access to headless browser tools for visual testing. Use these when implementing UI components.

### Available Tools

1. **playwright_browser_navigate** - Navigate to a URL
2. **playwright_browser_snapshot** - Get page structure (accessibility tree)
3. **playwright_browser_take_screenshot** - Capture PNG screenshot
4. **screenshot_upload** - Upload screenshot and get public URL

### Workflow for UI Implementation

When implementing frontend/UI components:

1. **Implement the code** as normal
2. **Start dev server** (if applicable):
   ```bash
   npm run dev &
   sleep 5  # Wait for server to start
   ```
3. **Navigate to the page**:
   - Use `browser_navigate` with `http://localhost:3000` (or appropriate URL)
4. **Verify structure**:
   - Use `browser_snapshot` to confirm expected elements exist
   - Check for buttons, forms, text content in the accessibility tree
5. **Capture screenshot**:
   - Use `browser_take_screenshot` with descriptive filename
   - Screenshot saves to `/tmp/playwright-output/`
6. **Upload and get URL**:
   - Use `screenshot_upload` with the file path, repo, PR number, and name
   - Returns URL like: `https://screenshots.5dlabs.io/repo/pr-123/login-form.png`
7. **Include in PR**:
   - Add screenshot URL to PR description using markdown:
   ```markdown
   ## Visual Preview
   ![Component Name](https://screenshots.5dlabs.io/...)
   ```

### Example Session

```
Agent: I'll implement the login form and verify it visually.

[Writes LoginForm.tsx component]

Agent: Let me verify this renders correctly.

[Calls browser_navigate with url="http://localhost:3000/login"]
[Calls browser_snapshot]

Result shows:
- form "Login"
  - textbox "Email"
  - textbox "Password" 
  - button "Sign In"

Agent: Structure looks correct. Let me capture a screenshot.

[Calls browser_take_screenshot with filename="login-form.png"]
[Calls screenshot_upload with file_path="/tmp/playwright-output/login-form.png", 
                            repo="5dlabs/myapp", 
                            pr_number=42, 
                            name="login-form"]

Returns: https://screenshots.5dlabs.io/5dlabs/myapp/pr-42/20250128-143022-login-form.png

Agent: I'll include this in the PR description.

[Updates PR body to include]:
## Visual Preview
![Login Form](https://screenshots.5dlabs.io/5dlabs/myapp/pr-42/20250128-143022-login-form.png)
```
```

## Implementation Phases

### Phase 1: MinIO Operator & Tenant (Week 1)

**Tasks**:
1. Deploy MinIO Operator via ArgoCD (`minio-operator.yaml`)
2. Deploy MinIO Tenant via ArgoCD (`minio-tenant.yaml`)
3. Configure Vault secrets for MinIO credentials
4. Verify tenant is healthy and bucket is created
5. Test S3 API access from within cluster

**Validation**:
- MinIO console accessible via port-forward
- Can upload/download files via `mc` client
- Bucket `screenshots` exists and is writable

### Phase 2: Playwright MCP Integration (Week 1-2)

**Tasks**:
1. Add Playwright MCP server to `tools/values.yaml`
2. Update tools server to route `playwright_*` tools
3. Configure shared volume for screenshot output
4. Test basic browser automation from agent container

**Validation**:
- Agent can navigate to a URL
- Agent can take a screenshot that saves to container filesystem
- Screenshot file is accessible

### Phase 3: Screenshot Upload Service (Week 2)

**Tasks**:
1. Implement screenshot upload MCP tool (Rust or shell)
2. Add MinIO credentials to tools server environment
3. Configure Cloudflare Tunnel for public screenshot access
4. Test end-to-end upload and URL generation

**Validation**:
- Upload a test image via the MCP tool
- Verify public URL is accessible via `screenshots.5dlabs.io`
- Verify image displays correctly in GitHub markdown

### Phase 4: Agent Workflow Integration (Week 2-3)

**Tasks**:
1. Update agent tool configurations to include playwright and upload tools
2. Add visual verification instructions to agent prompts
3. Update PR templates to include visual preview section
4. Test end-to-end workflow with Rex

**Validation**:
- Rex implements a UI component
- Rex successfully captures and uploads screenshot
- PR includes working screenshot preview
- Screenshot is viewable by reviewers

### Phase 5: Refinement (Week 3-4)

**Tasks**:
1. Add screenshot cleanup via MinIO ILM (delete after 90 days)
2. Implement retry logic for flaky screenshots
3. Add support for multiple screenshots per PR
4. Consider video capture for complex interactions
5. Document workflow in agent guides

**Validation**:
- Screenshots are cleaned up appropriately
- Multiple screenshots work correctly
- Documentation is complete

## API Reference

### Playwright MCP Tools

#### `browser_navigate`
```json
{
  "url": "string (required) - URL to navigate to"
}
```

#### `browser_snapshot`
```json
{
  // No parameters - returns accessibility tree of current page
}
```

#### `browser_take_screenshot`
```json
{
  "filename": "string (optional) - Output filename, defaults to page-{timestamp}.png",
  "fullPage": "boolean (optional) - Capture full scrollable page",
  "element": "string (optional) - Element description to screenshot",
  "ref": "string (optional) - Element reference from snapshot"
}
```

### Screenshot Upload Tool

#### `upload_screenshot`
```json
{
  "file_path": "string (required) - Path to screenshot file",
  "repo": "string (required) - Repository in format owner/name",
  "pr_number": "integer (required) - Pull request number",
  "name": "string (required) - Descriptive name for the screenshot"
}
```

**Returns**:
```json
{
  "url": "string - Public URL of uploaded screenshot",
  "key": "string - MinIO object key",
  "expires_at": "string - ISO timestamp when screenshot will be deleted (90 days)"
}
```

## Security Considerations

1. **Network Isolation**: Playwright runs with `--network host` only within the pod
2. **No External Access**: Browser cannot access external sites (can be restricted further)
3. **Screenshot Retention**: 90-day auto-deletion via MinIO ILM prevents indefinite storage
4. **MinIO Credentials**: Scoped access key for screenshots bucket only, write-only from agents
5. **Public URLs**: Screenshots are public but use unpredictable timestamped paths
6. **In-Cluster Storage**: Data stays in your cluster, not sent to external cloud

## Cost Estimation

| Component | Monthly Cost |
|-----------|--------------|
| MinIO Storage (10GB PVC) | $0 (uses local-path) |
| MinIO Operator | $0 (open source) |
| Cloudflare Tunnel | $0 (included in current setup) |
| Playwright Docker pulls | $0 (public image) |
| **Total** | **$0/month** |

## Success Metrics

1. **Adoption**: % of UI-related PRs that include visual previews
2. **Review Time**: Reduction in time-to-first-review for UI PRs
3. **Rework Rate**: Reduction in "visual bug" feedback iterations
4. **Agent Accuracy**: % of screenshots that correctly show implemented component

## Open Questions

1. Should we support video capture for interaction-heavy components?
2. Should screenshots be gated behind authentication for private repos?
3. Should we integrate with visual regression testing (comparing to baseline)?
4. Should Tess (QA agent) automatically verify screenshots against design specs?

## Files Created

| File | Purpose |
|------|---------|
| `infra/gitops/applications/minio-operator.yaml` | MinIO Operator ArgoCD Application |
| `infra/gitops/applications/minio-tenant.yaml` | MinIO Tenant for screenshots bucket |
| `infra/vault/secrets/minio.yaml` | Vault secrets for MinIO credentials |

## References

- [MinIO Operator](https://github.com/minio/operator)
- [MinIO Kubernetes Documentation](https://min.io/docs/minio/kubernetes/upstream/index.html)
- [Microsoft Playwright MCP](https://github.com/microsoft/playwright-mcp)
- [MinIO Client (mc)](https://min.io/docs/minio/linux/reference/minio-mc.html)
- [GitHub Markdown Image Syntax](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#images)
