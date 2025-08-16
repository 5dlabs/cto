# Toolman Guide: GitHub App Token Generation System

## Overview

This guide provides comprehensive instructions for using the GitHub App Token Generation System tools. The system provides secure, PAT-free authentication for GitHub operations through External Secrets integration and automated token generation.

## Available Tools

### 1. GitHub App Token Generator (`github-app-token-generator`)

**Purpose**: Generate GitHub App installation tokens securely without using Personal Access Tokens.

**Base URL**: `http://github-app-token-gen.workflows.svc.cluster.local:8080`

#### Endpoints

##### Generate Token
```http
POST /generate-token
Content-Type: application/json

{
  "app_id": "123456",
  "installation_id": "789012",  // Optional - auto-discovered if omitted
  "private_key": "LS0tLS1CRUdJTi..."  // Base64 encoded PEM
}
```

**Response**:
```json
{
  "token": "ghs_...",
  "expires_at": "2024-01-01T12:00:00Z",
  "installation_id": 789012
}
```

**Usage Example**:
```bash
# Generate token for rex GitHub App
curl -X POST http://github-app-token-gen.workflows.svc.cluster.local:8080/generate-token \
  -H "Content-Type: application/json" \
  -d '{
    "app_id": "'$(kubectl get secret github-app-rex -o jsonpath='{.data.appId}' | base64 -d)'",
    "private_key": "'$(kubectl get secret github-app-rex -o jsonpath='{.data.privateKey}')'"
  }'
```

##### Health Check
```http
GET /health
```

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "version": "1.0.0"
}
```

##### List Installations
```http
GET /installations/{app_id}
Authorization: Bearer {jwt_token}
```

**Response**:
```json
{
  "installations": [
    {
      "id": 789012,
      "account": {"login": "myorg", "type": "Organization"},
      "app_id": 123456
    }
  ]
}
```

### 2. External Secrets API (`external-secrets-api`)

**Purpose**: Monitor and manage External Secret synchronization status.

**Base URL**: `https://kubernetes.default.svc`

#### Endpoints

##### List External Secrets
```http
GET /apis/external-secrets.io/v1beta1/namespaces/workflows/externalsecrets
Authorization: Bearer {k8s_token}
```

**Usage Example**:
```bash
# List all GitHub App External Secrets
kubectl get externalsecrets -n workflows -l app=github-app
```

##### Get Sync Status
```http
GET /apis/external-secrets.io/v1beta1/namespaces/workflows/externalsecrets/{name}/status
```

**Usage Example**:
```bash
# Check sync status for rex GitHub App
kubectl get externalsecret github-app-rex -n workflows -o jsonpath='{.status.conditions}'
```

### 3. Kubernetes Secrets API (`kubernetes-secrets-api`)

**Purpose**: Access GitHub App secrets created by External Secrets.

#### Endpoints

##### List Secrets
```http
GET /api/v1/namespaces/workflows/secrets
Authorization: Bearer {k8s_token}
```

**Usage Example**:
```bash
# List all GitHub App secrets
kubectl get secrets -n workflows -l external-secrets.io/backend-type
```

##### Get Secret
```http
GET /api/v1/namespaces/workflows/secrets/{secret-name}
```

**Usage Example**:
```bash
# Safely view GitHub App secret (without private key)
kubectl get secret github-app-rex -n workflows -o yaml | grep -v privateKey
```

## Local Development Tools

### Token Generator Local Server

**Purpose**: Run token generator locally for development and testing.

**Command**: `./scripts/run-token-generator.sh --port 8080 --config ./config/token-gen.yaml`

**Configuration File** (`./config/token-gen.yaml`):
```yaml
github:
  api_url: "https://api.github.com"
  timeout: 30s
  retry:
    attempts: 5
    backoff: 500ms

server:
  port: 8080
  host: "localhost"
  
logging:
  level: "info"
  format: "json"

security:
  token_file_mode: 0600
  output_dir: "/tmp/tokens"
```

**Usage**:
```bash
# Start local token generator
./scripts/run-token-generator.sh

# Test token generation
curl -X POST http://localhost:8080/generate-token \
  -H "Content-Type: application/json" \
  -d '{
    "app_id": "123456",
    "private_key": "'$(base64 -w 0 < test-private-key.pem)'"
  }'
```

### Secret Sync Monitor

**Purpose**: Monitor External Secret synchronization status locally.

**Command**: `./scripts/monitor-secrets.sh --namespace workflows --interval 30s`

**Usage**:
```bash
# Monitor secret sync status
./scripts/monitor-secrets.sh --namespace workflows

# Check specific GitHub App
./scripts/monitor-secrets.sh --namespace workflows --app rex
```

## Common Usage Patterns

### 1. Workflow Integration

**InitContainer Pattern**:
```yaml
initContainers:
- name: gh-token
  image: ghcr.io/myorg/ghapp-token-gen:v1.0.0
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
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github

containers:
- name: main
  env:
    - name: GITHUB_TOKEN_FILE
      value: /var/run/github/token
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github
```

### 2. Manual Token Generation

```bash
#!/bin/bash
# generate-github-token.sh

APP_NAME=${1:-rex}
NAMESPACE=${2:-workflows}

echo "Generating token for GitHub App: $APP_NAME"

# Get app credentials
APP_ID=$(kubectl get secret github-app-$APP_NAME -n $NAMESPACE -o jsonpath='{.data.appId}' | base64 -d)
PRIVATE_KEY=$(kubectl get secret github-app-$APP_NAME -n $NAMESPACE -o jsonpath='{.data.privateKey}')

# Generate token
TOKEN=$(curl -s -X POST http://github-app-token-gen.workflows.svc.cluster.local:8080/generate-token \
  -H "Content-Type: application/json" \
  -d "{\"app_id\": \"$APP_ID\", \"private_key\": \"$PRIVATE_KEY\"}" | \
  jq -r '.token')

echo "Token: $TOKEN"
echo "Use with: export GITHUB_TOKEN=$TOKEN"
```

### 3. Secret Rotation Testing

```bash
#!/bin/bash
# test-secret-rotation.sh

APP_NAME=${1:-rex}
NAMESPACE=${2:-workflows}

echo "Testing secret rotation for $APP_NAME..."

# Get current secret version
CURRENT_VERSION=$(kubectl get secret github-app-$APP_NAME -n $NAMESPACE -o jsonpath='{.metadata.resourceVersion}')
echo "Current secret version: $CURRENT_VERSION"

# Update secret in external store (implementation specific)
# This would update AWS Secrets Manager, Azure Key Vault, etc.
echo "Update the secret in your external secret store now..."
echo "Press Enter when done..."
read

# Wait for External Secret to sync
echo "Waiting for External Secret sync..."
while true; do
  NEW_VERSION=$(kubectl get secret github-app-$APP_NAME -n $NAMESPACE -o jsonpath='{.metadata.resourceVersion}')
  if [ "$NEW_VERSION" != "$CURRENT_VERSION" ]; then
    echo "Secret updated! New version: $NEW_VERSION"
    break
  fi
  sleep 5
done

# Test token generation with new credentials
echo "Testing token generation with new credentials..."
./generate-github-token.sh $APP_NAME $NAMESPACE
```

## Troubleshooting

### Common Issues

#### 1. External Secret Not Syncing

**Symptoms**:
- ExternalSecret status shows "Error" or "NotSynced"
- Kubernetes secret is not created or outdated

**Diagnosis**:
```bash
# Check ExternalSecret status
kubectl describe externalsecret github-app-rex -n workflows

# Check External Secrets Operator logs
kubectl logs -n external-secrets-system deployment/external-secrets -f
```

**Solutions**:
- Verify SecretStore configuration and permissions
- Check network connectivity to external secret store
- Validate secret paths in external store
- Restart External Secrets Operator if needed

#### 2. Token Generation Failures

**Symptoms**:
- Token generator returns 401/403 errors
- Generated tokens don't work with GitHub API

**Diagnosis**:
```bash
# Test token generator health
curl -s http://github-app-token-gen.workflows.svc.cluster.local:8080/health

# Check token generator logs
kubectl logs -n workflows deployment/github-app-token-gen -f

# Validate GitHub App configuration
gh auth status -h github.com -t $GITHUB_TOKEN
```

**Solutions**:
- Verify GitHub App installation and permissions
- Check private key format (must be PEM)
- Validate JWT expiration settings
- Ensure GitHub API connectivity

#### 3. Workflow Authentication Errors

**Symptoms**:
- Workflows fail with authentication errors
- Token file is missing or empty

**Diagnosis**:
```bash
# Check initContainer logs
kubectl logs pod/workflow-pod-name -c gh-token

# Verify token file permissions
kubectl exec pod/workflow-pod-name -c main -- ls -la /var/run/github/token

# Test token validity
kubectl exec pod/workflow-pod-name -c main -- cat /var/run/github/token | \
  gh auth status -h github.com -t -
```

**Solutions**:
- Check initContainer completion status
- Verify volume mounting configuration
- Ensure proper file permissions (0600)
- Validate token expiration time

### Debug Commands

```bash
# Check all GitHub App secrets
kubectl get secrets -n workflows -l app=github-app

# Monitor External Secret sync status
watch kubectl get externalsecrets -n workflows

# Test token generator directly
kubectl run debug --rm -it --image=curlimages/curl:latest -- \
  curl -X POST http://github-app-token-gen.workflows.svc.cluster.local:8080/health

# Check RBAC permissions
kubectl auth can-i get secrets --as=system:serviceaccount:workflows:workflows-sa -n workflows

# Validate JWT structure
echo "$JWT" | cut -d. -f1 | base64 -d | jq .
echo "$JWT" | cut -d. -f2 | base64 -d | jq .
```

### Performance Monitoring

```bash
# Monitor token generation metrics
curl -s http://github-app-token-gen.workflows.svc.cluster.local:8080/metrics | \
  grep github_token_generation

# Check secret sync metrics
kubectl get externalsecrets -n workflows -o json | \
  jq '.items[] | {name: .metadata.name, status: .status.conditions[-1].type}'

# Monitor workflow resource usage
kubectl top pods -n workflows --sort-by=cpu
```

## Best Practices

### Security

1. **Never Log Secrets**: Ensure private keys and tokens are not logged
2. **Proper Permissions**: Use minimal RBAC permissions for service accounts
3. **Token Rotation**: Implement regular token rotation and expiration monitoring
4. **Audit Logging**: Enable audit logging for secret access

### Performance

1. **Token Caching**: Cache tokens until expiration to reduce API calls
2. **Retry Logic**: Implement exponential backoff for API failures
3. **Resource Limits**: Set appropriate CPU/memory limits for containers
4. **Monitoring**: Monitor token generation latency and success rates

### Reliability

1. **Health Checks**: Implement proper liveness and readiness probes
2. **Circuit Breaker**: Use circuit breakers for external API calls
3. **Backup Secrets**: Maintain backup access methods for emergencies
4. **Documentation**: Keep runbooks and troubleshooting guides updated

## Support and Maintenance

### Regular Maintenance Tasks

1. **Weekly**:
   - Review token generation success rates
   - Check External Secret sync status
   - Monitor resource usage trends

2. **Monthly**:
   - Update container images with security patches
   - Review and rotate GitHub App keys
   - Validate backup and recovery procedures

3. **Quarterly**:
   - Conduct security review of the entire system
   - Performance testing and optimization
   - Documentation updates and training

### Escalation Path

1. **Level 1**: Check common issues and run diagnostic commands
2. **Level 2**: Review logs, check External Secret store connectivity
3. **Level 3**: Engage GitHub App administrators and security team
4. **Level 4**: Contact external secret store provider support

For additional support, consult the main documentation or reach out to the DevOps team.