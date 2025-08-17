# External Secrets for Agent Apps Implementation

You are setting up External Secrets resources for Cleo and Tess GitHub Apps to enable secure credential management for multi-agent orchestration. Create External Secrets configurations that follow existing patterns while providing agent-specific GitHub App authentication.

## Objective

Configure External Secrets resources for the new Cleo and Tess GitHub Apps with proper secret store integration, enabling secure GitHub API access for specialized agent workflows.

## Context

Multi-agent orchestration requires each agent to have distinct GitHub App identities:
- **Cleo**: Needs GitHub API access for PR labeling and code quality workflows
- **Tess**: Requires GitHub access for PR reviews, approvals, and testing workflows
- **Rex**: Continues using existing `github-app-5dlabs-rex` secret

Each agent needs secure, automatically-rotated credentials for GitHub App authentication.

## Implementation Requirements

### 1. Create Cleo External Secrets Configuration

Create External Secret for 5DLabs-Cleo GitHub App:
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-cleo
  namespace: agents-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    kind: ClusterSecretStore
    name: aws-secrets-manager
  target:
    name: github-app-5dlabs-cleo
    template:
      data:
        app-id: "{{ .appId }}"
        private-key: "{{ .privateKey }}"
        client-id: "{{ .clientId }}"
        installation-id: "{{ .installationId }}"
```

### 2. Create Tess External Secrets Configuration

Create External Secret for 5DLabs-Tess GitHub App:
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-tess
  namespace: agents-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    kind: ClusterSecretStore 
    name: aws-secrets-manager
  target:
    name: github-app-5dlabs-tess
    template:
      data:
        app-id: "{{ .appId }}"
        private-key: "{{ .privateKey }}"
        client-id: "{{ .clientId }}"
        installation-id: "{{ .installationId }}"
        webhook-secret: "{{ .webhookSecret }}"
```

### 3. Update Controller Secret Mounting

Modify CodeRun controller to mount agent-specific secrets:
```rust
fn create_agent_secret_volumes(github_app: &str) -> Vec<Volume> {
    match github_app {
        "5DLabs-Cleo" => mount_secret("github-app-5dlabs-cleo"),
        "5DLabs-Tess" => mount_secret("github-app-5dlabs-tess"),
        _ => mount_secret("github-app-5dlabs-rex"), // Default
    }
}
```

### 4. Integrate with Agent Container Templates

Update container templates to use agent-specific credentials:
```handlebars
{{#if (eq github_app "5DLabs-Cleo")}}
export GITHUB_APP_ID=$(cat /etc/github-app/app-id)
export GITHUB_INSTALLATION_ID=$(cat /etc/github-app/installation-id)
# Generate GitHub token for Cleo operations
{{else if (eq github_app "5DLabs-Tess")}}
export GITHUB_APP_ID=$(cat /etc/github-app/app-id)  
export GITHUB_INSTALLATION_ID=$(cat /etc/github-app/installation-id)
# Generate GitHub token for Tess operations
{{/if}}
```

## Technical Specifications

### Secret Structure
Each GitHub App secret contains:
- `app-id`: GitHub App ID for authentication
- `private-key`: RSA private key for JWT signing
- `client-id`: OAuth client ID for GitHub App
- `installation-id`: Installation ID for repository access
- `webhook-secret`: (Tess only) Webhook validation secret

### Secret Store Backend
Store credentials in AWS Secrets Manager:
- `github-apps/5dlabs-cleo`: Cleo GitHub App credentials
- `github-apps/5dlabs-tess`: Tess GitHub App credentials

### Refresh Interval
Set 1-hour refresh interval for:
- Automatic credential rotation
- Security compliance
- Minimal disruption during updates

### Volume Mount Security
- Mount secrets read-only at `/etc/github-app/`
- Set file permissions to 0400 for private keys
- Use dedicated service accounts with minimal permissions

## Integration Points

### Controller Code Changes
- Modify resource creation to include agent-specific secret volumes
- Add secret volume mounts to pod specifications
- Update template context with secret availability

### Template Updates
- Add agent-specific GitHub authentication logic
- Include token generation scripts in container templates
- Provide fallback behavior when secrets unavailable

### Secret Store Configuration
- Ensure AWS Secrets Manager contains required credential paths
- Verify ClusterSecretStore has proper permissions
- Test External Secrets Operator can access backend

## Success Criteria

1. **Secret Creation**: External Secrets creates agent-specific Kubernetes secrets
2. **Secret Content**: All required GitHub App credentials present and valid
3. **Controller Integration**: CodeRun controller mounts correct secrets per agent
4. **Template Integration**: Agent containers can access and use credentials
5. **Authentication**: Agents can successfully authenticate with GitHub API
6. **Rotation**: Secret rotation works without disrupting running agents
7. **Security**: Secrets properly secured with read-only mounts and restricted permissions

## Testing Requirements

### Secret Management Testing
- Verify External Secrets creates secrets correctly
- Test secret rotation doesn't disrupt running agents
- Validate secret content matches expected structure
- Test missing secrets are handled gracefully

### Agent Integration Testing
- Create test CodeRun CRDs for each agent type
- Verify correct secrets mounted in each agent container
- Test GitHub API authentication works for each agent
- Validate token generation and GitHub CLI integration

### Security Validation
- Verify secrets mounted read-only with correct permissions
- Test agents can't access other agents' secrets
- Validate secret rotation maintains security boundaries
- Test secret cleanup when agents complete

## Implementation Deliverables

### External Secrets Configuration
- External Secret resources for Cleo and Tess GitHub Apps
- Proper secret store references and data mapping
- Security labels and metadata for resource management

### Controller Updates
- Modified resource creation to support agent-specific secrets
- Volume mount logic for GitHub App credentials
- Error handling for missing or invalid secrets

### Template Integration
- Updated container templates with agent-specific authentication
- GitHub token generation scripts and utilities
- Fallback logic for scenarios without GitHub access

### Documentation and Testing
- Configuration validation scripts and procedures
- Integration testing for each agent type
- Troubleshooting guide for secret-related issues

Focus on following existing External Secrets patterns while providing secure, isolated credential management for each agent's specialized GitHub App requirements.