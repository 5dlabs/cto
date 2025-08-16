# Autonomous Implementation Prompt: GitHub App Token Generation System

## Mission Statement

You are implementing a secure, PAT-free GitHub App authentication system. Your goal is to create a production-ready solution that sources GitHub App credentials from External Secrets and provides installation tokens to workflows without using Personal Access Tokens.

Important:
- Do NOT re-implement functionality that already exists. Extend the existing assets instead.
- We already use External Secrets and in-container token minting; focus on adding the new apps and wiring.
- Scope: Rust-only for now; multi-language support is out of scope.

## Context

This system replaces insecure PAT-based authentication with a proper GitHub App credential management system. The implementation must handle five GitHub Apps (rex, clippy, qa, triage, security) and provide tokens to Argo Workflows via shared volumes.

## Technical Requirements

### Must Implement

1. **External Secrets Integration**
   - Configure ExternalSecret resources for all five GitHub Apps
   - Support multiple secret store providers (AWS Secrets Manager priority)
   - Handle secret rotation with 1-hour refresh interval
   - Implement proper error handling and retry logic

2. **Token Generator Service**
   - Build lightweight container (Node.js or Go preferred)
   - Create RS256 JWT with proper claims structure
   - Auto-discover installation IDs when not provided  
   - Exchange JWT for installation tokens via GitHub API
   - Write tokens atomically with 0600 permissions
   - Handle rate limiting and API errors gracefully

3. **Workflow Integration**
   - Design initContainer pattern for token generation
   - Mount secrets as environment variables or volumes
   - Share tokens via emptyDir at `/var/run/github/token`
   - Support parameterized GitHub App selection
   - Implement proper RBAC for secret access

4. **Security Implementation**
   - Never log private keys or tokens
   - Use atomic file operations for token writes
   - Implement least-privilege service accounts
   - Handle token expiration and refresh
   - Zero memory buffers after token operations

### Configuration Schema

```yaml
# ExternalSecret Pattern (repeat for each app)
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-{APP_NAME}
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets
    kind: SecretStore
  target:
    name: github-app-{APP_NAME}
  data:
    - secretKey: appId
      remoteRef: { key: /github-apps/{APP_NAME}/appId }
    - secretKey: privateKey  
      remoteRef: { key: /github-apps/{APP_NAME}/privateKey }
```

### Token Generator Requirements

**Input Parameters:**
- `APP_ID` (required): GitHub App ID
- `PRIVATE_KEY` (required): PEM-formatted private key
- `INSTALLATION_ID` (optional): Installation ID for token scope
- `OUTPUT_PATH` (optional): Token output path, default `/var/run/github/token`
- `GITHUB_API_URL` (optional): GitHub API base URL

**API Endpoints to Use:**
- `GET /app/installations` - List app installations
- `POST /app/installations/{id}/access_tokens` - Create installation token

**Error Handling Requirements:**
- Retry 5xx errors with exponential backoff (5 attempts, 500ms base, max 5s)
- Fast-fail on 401/403 with clear error messages
- Handle rate limiting (429) with Retry-After header
- Validate JWT expiration and refresh as needed

### Container Specifications

**Base Image:** Use distroless or alpine for minimal attack surface
**Security:** Non-root user (65532:65532), read-only root filesystem
**Size Target:** <50MB compressed
**Health Check:** Simple HTTP endpoint on `/healthz`
**Logging:** Structured JSON logs, no secrets in output

## Implementation Approach

### Phase 1: External Secrets Setup
```bash
# 1. Install External Secrets Operator
kubectl apply -f https://raw.githubusercontent.com/external-secrets/external-secrets/main/deploy/crds/bundle.yaml

# 2. Configure your SecretStore (example for AWS)
kubectl apply -f - <<EOF
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: aws-secrets
  namespace: workflows
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-east-1
      auth:
        jwt:
          serviceAccountRef:
            name: external-secrets-sa
EOF

# 3. Create ExternalSecrets for each app
for app in rex clippy qa triage security; do
  kubectl apply -f externalsecret-github-app-$app.yaml
done
```

### Phase 2: Token Generator Development

**JWT Creation Pattern (Node.js):**
```javascript
const jwt = require('jsonwebtoken');

function createAppJWT(appId, privateKey) {
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    iss: appId,
    iat: now - 60,  // 1 minute ago (to account for clock skew)
    exp: now + 540  // 9 minutes from now
  };
  return jwt.sign(payload, privateKey, { algorithm: 'RS256' });
}
```

**Installation Token Exchange:**
```javascript
async function getInstallationToken(jwt, installationId) {
  const response = await fetch(
    `https://api.github.com/app/installations/${installationId}/access_tokens`,
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${jwt}`,
        'Accept': 'application/vnd.github+json'
      }
    }
  );
  
  if (!response.ok) {
    throw new Error(`Token exchange failed: ${response.status}`);
  }
  
  const data = await response.json();
  return data.token;
}
```

### Phase 3: Workflow Template Integration

**InitContainer Pattern:**
```yaml
initContainers:
- name: gh-token
  image: ghcr.io/YOUR_ORG/ghapp-token-gen:latest
  env:
    - name: APP_ID
      valueFrom:
        secretKeyRef:
          name: github-app-{{workflow.parameters.githubApp}}
          key: appId
    - name: PRIVATE_KEY
      valueFrom:
        secretKeyRef:
          name: github-app-{{workflow.parameters.githubApp}}
          key: privateKey
    - name: OUTPUT_PATH
      value: /var/run/github/token
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github
  securityContext:
    runAsNonRoot: true
    runAsUser: 65532
    readOnlyRootFilesystem: true
```

### Phase 4: RBAC Configuration

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: workflows-sa
  namespace: workflows
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: github-app-secrets-read
  namespace: workflows
rules:
- apiGroups: [""]
  resources: ["secrets"]
  resourceNames: 
    - "github-app-rex"
    - "github-app-clippy" 
    - "github-app-qa"
    - "github-app-triage"
    - "github-app-security"
  verbs: ["get"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: workflows-sa-secrets-read
  namespace: workflows
subjects:
- kind: ServiceAccount
  name: workflows-sa
roleRef:
  kind: Role
  name: github-app-secrets-read
  apiGroup: rbac.authorization.k8s.io
```

## Validation Requirements

### Test Cases to Implement

1. **Secret Sync Validation**
   - Verify all five GitHub App secrets are created
   - Test secret rotation by updating backend values
   - Confirm ExternalSecret status shows "Synced"

2. **Token Generation Testing**
   - Generate tokens for each GitHub App
   - Validate JWT structure and claims
   - Test installation ID auto-discovery
   - Verify token permissions scope

3. **Workflow Integration Testing**  
   - Run dry-run workflow with token mounting
   - Verify `gh auth status` works with generated tokens
   - Test parameterized GitHub App selection
   - Confirm no secrets appear in logs

4. **Error Handling Testing**
   - Test with invalid private keys (expect fast failure)
   - Simulate GitHub API errors (test retry logic)
   - Test rate limiting scenarios
   - Verify graceful degradation

### Acceptance Criteria Validation

**Deployment Success:**
```bash
# Verify External Secrets
kubectl get externalsecrets -n workflows
kubectl get secrets -n workflows | grep github-app

# Test token generation
kubectl run test-token --rm -it \
  --image=ghcr.io/YOUR_ORG/ghapp-token-gen:latest \
  --env="APP_ID=12345" \
  --env="PRIVATE_KEY=$(cat test-key.pem)" \
  -- sh -c 'echo "Testing token generation..."'

# Validate workflow execution
argo submit --from workflowtemplate/test-github-app \
  -p github-app=rex --watch
```

## Security Checklist

- [ ] Private keys never appear in logs or environment dumps
- [ ] Tokens written with 0600 permissions using atomic operations
- [ ] Service account uses least-privilege RBAC
- [ ] Container runs as non-root user
- [ ] No PATs used anywhere in the system
- [ ] JWT expiration properly validated
- [ ] Rate limiting handled gracefully
- [ ] Secret rotation tested and documented

## Monitoring Implementation

**Required Metrics:**
- Token generation success/failure rates
- JWT expiration warnings  
- Secret sync status
- API rate limit consumption
- Authentication error rates

**Logging Requirements:**
- Structured JSON format
- Include correlation IDs
- No sensitive data in logs
- Proper log levels (DEBUG, INFO, WARN, ERROR)

## Success Criteria

Your implementation is complete when:

1. All five GitHub App secrets sync from External Secrets
2. Token generator creates valid installation tokens
3. Workflows can authenticate with GitHub using generated tokens
4. No PATs are used anywhere in the system
5. Secret rotation works without service interruption
6. All security requirements are met
7. Comprehensive tests pass for all scenarios
8. Documentation and runbooks are complete

## Delivery Artifacts

Create these files in the task directory:
- Working Kubernetes manifests
- Container source code and Dockerfile  
- CI/CD pipeline configuration
- Test suite with all validation cases
- Operational documentation and runbooks
- Security review checklist

Remember: Security is paramount. Never compromise on proper secret handling, and always validate that no credentials leak into logs or environment dumps.