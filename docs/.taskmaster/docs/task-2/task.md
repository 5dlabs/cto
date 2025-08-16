# Task 2: External Secrets for GitHub App Credentials and Token Generation Pattern

## Overview

This task extends our existing, PAT-free authentication using External Secrets and in-container token minting. We source GitHub App credentials via External Secrets and mint installation tokens inside running containers (no separate token service).

Reality alignment with our platform:
- Do NOT re-implement functionality that already exists. Extend the existing assets instead.
- We already manage GitHub App secrets for Morgan, Rex, Blaze, and Cipher via External Secrets at `infra/secret-store/agent-secrets-external-secrets.yaml`.
- Our agent containers already mint GitHub App installation tokens inside the container when credentials are present (no separate token microservice required).
- What remains is to add new GitHub Apps and ExternalSecrets for the additional agents introduced in Task 1 (Clippy, QA, Triage, Security), and wire them into Helm values.

Admin secret (separate from per-agent GitHub Apps):
- We use a single admin secret named `agent-admin-secrets` for administrative operations (cluster and Argo CD administration, and a repo/org-level GitHub admin PAT when absolutely necessary).
- Contents typically include: `KUBECONFIG` (or `KUBECONFIG_B64`), `ARGOCD_SERVER`, `ARGOCD_AUTH_TOKEN` (or password), and `GITHUB_ADMIN_TOKEN`.
- This secret is not used by role-specific agents; those use their own GitHub App secrets via ExternalSecrets.

Existing ExternalSecrets (under `infra/secret-store/agent-secrets-external-secrets.yaml`):
- `github-app-5dlabs-rex`
- `github-app-5dlabs-blaze`
- `github-app-5dlabs-morgan`
- `github-app-5dlabs-cipher`
Extend with:
- `github-app-5dlabs-clippy`
- `github-app-5dlabs-qa`
- `github-app-5dlabs-triage`
- `github-app-5dlabs-security`

Naming carried over from Task 1 (friendly → GitHub App):
- Clippy → 5DLabs-Clippy
- QA → 5DLabs-QA
- Triage → 5DLabs-Triage
- Security → 5DLabs-Security

## Architecture

The solution consists of three main components:

1. **External Secrets Integration**: Syncs GitHub App credentials (appId, privateKey) to Kubernetes Secrets
2. **Token Generation (existing)**: Containers mint installation tokens from mounted secrets
3. **Workflow Integration**: Templates consume tokens via shared volumes

## Implementation Overview

### GitHub App Secrets Structure

The system manages credentials for five GitHub Apps:
- `github-app-rex` (Implementation agent)
- `github-app-clippy` (Code formatting agent) 
- `github-app-qa` (Quality assurance agent)
- `github-app-triage` (Issue triage agent)
- `github-app-security` (Security scanning agent)

Each secret contains:
- `appId`: GitHub App ID
- `privateKey`: GitHub App private key (PEM format)

### Token Generation Flow

1. InitContainer mounts GitHub App secret and reads appId/privateKey
2. Creates RS256 JWT for GitHub App authentication
3. Calls GitHub API to get installation ID (if not provided)
4. Exchanges JWT for installation access token
5. Writes token to shared volume at `/var/run/github/token`
6. Main containers read token from shared volume

### Security Features

- No Personal Access Tokens (PATs) used anywhere
- Private keys stored securely in external secret stores
- Tokens written with 0600 permissions
- Automatic token rotation support
- Least-privilege RBAC for secret access
- No secrets logged or exposed in environment dumps

## Directory Structure

```
.taskmaster/docs/task-2/
├── task.md                    # This comprehensive guide
├── prompt.md                  # Autonomous implementation prompt
├── acceptance-criteria.md     # Testing and validation criteria
├── client-config.json        # MCP client configuration
└── toolman-guide.md          # Tool usage documentation
```

## Key Components

### External Secrets Configuration (example)

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-rex
  namespace: workflows
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: cluster-secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-rex
    template:
      type: Opaque
  data:
    - secretKey: appId
      remoteRef: { key: /github-apps/rex/appId }
    - secretKey: privateKey
      remoteRef: { key: /github-apps/rex/privateKey }
```

### Token Generation (existing)

We reuse container-embedded token minting already present in our images/templates (intake/docs/code). Ensure the new GitHub App secrets are mounted where required. Tokens are written to `/var/run/github/token` with 0600 perms.

### Workflow Integration Pattern

```yaml
initContainers:
- name: gh-token
  image: ghcr.io/5dlabs/cto/runtime:latest
  env:
    - name: APP_ID
      valueFrom:
        secretKeyRef:
          name: github-app-rex
          key: appId
    - name: PRIVATE_KEY
      valueFrom:
        secretKeyRef:
          name: github-app-rex
          key: privateKey
    - name: OUTPUT_PATH
      value: /var/run/github/token
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github

containers:
- name: runner
  env:
    - name: GITHUB_TOKEN_FILE
      value: /var/run/github/token
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github
```

## Implementation Steps

### Phase 1: External Secrets Setup
1. Create/extend ExternalSecret resources for each GitHub App using our existing ClusterSecretStore (`secret-store`)
2. Verify Kubernetes Secrets are created and synced
3. Test secret rotation by updating backend values

### Phase 2: Token Generation (reuse existing)
1. Confirm containers mint tokens from mounted secrets and write to `/var/run/github/token`
2. Validate permissions (0600) and error handling

### Phase 3: Workflow Integration
1. Create parameterized WorkflowTemplate mounting patterns
2. Implement RBAC for secret access
3. Add token validation and error handling
4. Test end-to-end token generation and usage

### Phase 4: Production Hardening
1. Monitoring and alerting for ExternalSecrets sync and token usage
2. Rotation and failure handling
3. Operational runbooks

## Security Considerations

### Secret Management
- Store private keys in external secret stores with encryption at rest
- Use least-privilege IAM/RBAC policies for secret access
- Rotate keys regularly and test rotation procedures
- Never log private keys or tokens

### Token Handling
- Write tokens with 0600 file permissions
- Use atomic file operations (write to temp, then rename)
- Clear token buffers from memory after use
- Set appropriate token expiration times

### Network Security
- Use TLS for all GitHub API communications
- Implement proper certificate validation
- Apply network policies to restrict pod-to-pod communication
- Monitor for suspicious API usage patterns

## Monitoring and Observability

### Key Metrics
- Token generation success/failure rates
- Token validation errors
- Secret rotation events
- API rate limit consumption
- JWT expiration warnings

### Logging
- Successful token generations (without sensitive data)
- Authentication failures with error codes
- Rate limiting events
- Secret rotation activities

### Alerting
- Token generation failures
- Secret sync failures
- Rate limit threshold breaches
- Expired or invalid credentials

## Troubleshooting

### Common Issues

**ExternalSecret not syncing:**
- Check SecretStore configuration and permissions
- Verify external secret store connectivity
- Review ExternalSecret status conditions

**Token generation failures:**
- Validate GitHub App installation
- Check private key format and encoding
- Verify API rate limits and quotas
- Review network connectivity to GitHub API

**Workflow authentication errors:**
- Confirm token file permissions and location
- Check token expiration time
- Validate GitHub App permissions for target repositories

### Debug Commands

```bash
# Check ExternalSecret status
kubectl describe externalsecret github-app-rex -n workflows

# Verify secret contents (safely)
kubectl get secret github-app-rex -n workflows -o yaml | grep -v "privateKey:"

# Test token generation manually
kubectl run debug --rm -it --image=ghcr.io/ORG/ghapp-token-gen:TAG \
  --env="APP_ID=$(kubectl get secret github-app-rex -o jsonpath='{.data.appId}' | base64 -d)" \
  -- /app/generate-token

# Check workflow pod logs
kubectl logs -f pod/workflow-pod-name -c gh-token
```

## Dependencies

### External Dependencies
- External Secrets Operator
- External secret store (AWS Secrets Manager, Azure Key Vault, etc.)
- GitHub Apps with proper permissions
- Container registry for token generator image

### Internal Dependencies
- Kubernetes cluster with RBAC enabled
- Argo Workflows for template execution
- Monitoring stack (Prometheus, Grafana)
- Log aggregation system

## References

- [GitHub Apps Authentication](https://docs.github.com/en/developers/apps/building-github-apps/authenticating-with-github-apps)
- [External Secrets Operator](https://external-secrets.io/)
- [JWT Specification (RFC 7519)](https://tools.ietf.org/html/rfc7519)
- [Kubernetes Secret Management](https://kubernetes.io/docs/concepts/configuration/secret/)