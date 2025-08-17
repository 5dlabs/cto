# Task 9: Configure External Secrets for Agent Apps

## Overview

Setup External Secrets resources for Cleo and Tess GitHub Apps with proper secret store integration. This enables the new agent GitHub Apps to authenticate with GitHub API for their specialized workflows while maintaining secure credential management.

## Technical Context

Multi-agent orchestration requires each agent to have its own GitHub App identity for authentication and API access. Cleo needs GitHub API access for PR labeling, while Tess requires GitHub access for PR reviews and approvals. External Secrets ensures secure credential management with automatic rotation.

## Implementation Guide

### Phase 1: Analyze Existing External Secrets Pattern

1. **Review Current Implementation**
   ```bash
   # Examine existing Rex External Secrets configuration
   kubectl get externalsecret -n agents-platform github-app-5dlabs-rex -o yaml
   
   # Check ClusterSecretStore configuration
   kubectl get clustersecretstore -o yaml
   ```

2. **Document Required Secret Structure**
   ```yaml
   # Required GitHub App credential structure
   apiVersion: v1
   kind: Secret
   metadata:
     name: github-app-5dlabs-cleo
   data:
     app-id: <base64-encoded-app-id>
     private-key: <base64-encoded-private-key>
     client-id: <base64-encoded-client-id>
     installation-id: <base64-encoded-installation-id>
   ```

### Phase 2: Create Cleo External Secrets Configuration

1. **Cleo GitHub App External Secret**
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
       creationPolicy: Owner
       template:
         type: Opaque
         data:
           app-id: "{{ .appId }}"
           private-key: "{{ .privateKey }}"
           client-id: "{{ .clientId }}"
           installation-id: "{{ .installationId }}"
     data:
     - secretKey: appId
       remoteRef:
         key: github-apps/5dlabs-cleo
         property: app_id
     - secretKey: privateKey
       remoteRef:
         key: github-apps/5dlabs-cleo
         property: private_key
     - secretKey: clientId
       remoteRef:
         key: github-apps/5dlabs-cleo
         property: client_id
     - secretKey: installationId
       remoteRef:
         key: github-apps/5dlabs-cleo
         property: installation_id
   ```

2. **Secret Store Backend Configuration**
   ```yaml
   # AWS Secrets Manager entries required
   # Path: github-apps/5dlabs-cleo
   {
     "app_id": "123456",
     "private_key": "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----",
     "client_id": "Iv1.abcdef123456",
     "installation_id": "78901234"
   }
   ```

### Phase 3: Create Tess External Secrets Configuration

1. **Tess GitHub App External Secret**
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
       creationPolicy: Owner
       template:
         type: Opaque
         data:
           app-id: "{{ .appId }}"
           private-key: "{{ .privateKey }}"
           client-id: "{{ .clientId }}"
           installation-id: "{{ .installationId }}"
           webhook-secret: "{{ .webhookSecret }}"
     data:
     - secretKey: appId
       remoteRef:
         key: github-apps/5dlabs-tess
         property: app_id
     - secretKey: privateKey
       remoteRef:
         key: github-apps/5dlabs-tess
         property: private_key
     - secretKey: clientId
       remoteRef:
         key: github-apps/5dlabs-tess
         property: client_id
     - secretKey: installationId
       remoteRef:
         key: github-apps/5dlabs-tess
         property: installation_id
     - secretKey: webhookSecret
       remoteRef:
         key: github-apps/5dlabs-tess
         property: webhook_secret
   ```

2. **Tess-Specific Secret Requirements**
   ```yaml
   # Additional secrets for Tess testing capabilities
   # Path: github-apps/5dlabs-tess
   {
     "app_id": "234567", 
     "private_key": "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----",
     "client_id": "Iv1.fedcba654321",
     "installation_id": "89012345",
     "webhook_secret": "random-webhook-secret-string"
   }
   ```

### Phase 4: Integrate with Agent Container Templates

1. **Secret Mounting in Container Templates**
   ```handlebars
   # In container-cleo.sh.hbs
   {{#if (eq github_app "5DLabs-Cleo")}}
   # Cleo GitHub App authentication
   export GITHUB_APP_ID=$(cat /etc/github-app/app-id)
   export GITHUB_PRIVATE_KEY_FILE=/etc/github-app/private-key
   export GITHUB_CLIENT_ID=$(cat /etc/github-app/client-id)
   export GITHUB_INSTALLATION_ID=$(cat /etc/github-app/installation-id)
   
   # Generate GitHub token for API operations
   GITHUB_TOKEN=$(generate-github-token.sh)
   export GITHUB_TOKEN
   {{/if}}
   ```

2. **Secret Volume Mounts in CodeRun Controller**
   ```rust
   // In controller code - add secret volume mounts based on github_app
   match code_run.spec.github_app.as_str() {
       "5DLabs-Cleo" => {
           pod_spec.volumes.push(Volume {
               name: "github-app-credentials".to_string(),
               secret: Some(SecretVolumeSource {
                   secret_name: Some("github-app-5dlabs-cleo".to_string()),
                   ..Default::default()
               }),
               ..Default::default()
           });
       }
       "5DLabs-Tess" => {
           pod_spec.volumes.push(Volume {
               name: "github-app-credentials".to_string(), 
               secret: Some(SecretVolumeSource {
                   secret_name: Some("github-app-5dlabs-tess".to_string()),
                   ..Default::default()
               }),
               ..Default::default()
           });
       }
       _ => {} // Rex uses existing secret
   }
   ```

### Phase 5: Create GitHub Token Generation Helper

1. **Token Generation Script Template**
   ```bash
   #!/bin/bash
   # generate-github-token.sh - Generate GitHub App installation token
   
   set -euo pipefail
   
   APP_ID=$(cat /etc/github-app/app-id)
   PRIVATE_KEY_FILE=/etc/github-app/private-key
   INSTALLATION_ID=$(cat /etc/github-app/installation-id)
   
   # Generate JWT for GitHub App authentication
   JWT=$(create-jwt.py "$APP_ID" "$PRIVATE_KEY_FILE")
   
   # Get installation access token
   TOKEN_RESPONSE=$(curl -s \
     -X POST \
     -H "Authorization: Bearer $JWT" \
     -H "Accept: application/vnd.github.v3+json" \
     "https://api.github.com/app/installations/${INSTALLATION_ID}/access_tokens")
   
   # Extract token from response
   echo "$TOKEN_RESPONSE" | jq -r '.token'
   ```

2. **JWT Creation Helper Script**
   ```python
   #!/usr/bin/env python3
   # create-jwt.py - Create GitHub App JWT
   
   import jwt
   import time
   import sys
   
   def create_github_app_jwt(app_id, private_key_path):
       with open(private_key_path, 'r') as key_file:
           private_key = key_file.read()
       
       now = int(time.time())
       payload = {
           'iat': now,
           'exp': now + 600,  # 10 minutes
           'iss': app_id
       }
       
       return jwt.encode(payload, private_key, algorithm='RS256')
   
   if __name__ == "__main__":
       if len(sys.argv) != 3:
           print("Usage: create-jwt.py <app_id> <private_key_path>")
           sys.exit(1)
           
       app_id = sys.argv[1]
       private_key_path = sys.argv[2]
       
       token = create_github_app_jwt(app_id, private_key_path)
       print(token)
   ```

## Code Examples

### Complete Cleo External Secret Configuration
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-cleo
  namespace: agents-platform
  labels:
    app.kubernetes.io/name: github-app-cleo
    app.kubernetes.io/component: external-secrets
    app.kubernetes.io/part-of: taskmaster-agents
spec:
  refreshInterval: 1h
  secretStoreRef:
    kind: ClusterSecretStore
    name: aws-secrets-manager
  target:
    name: github-app-5dlabs-cleo
    creationPolicy: Owner
    deletionPolicy: Retain
    template:
      type: Opaque
      metadata:
        labels:
          app.kubernetes.io/name: github-app-cleo
          app.kubernetes.io/managed-by: external-secrets
      data:
        app-id: "{{ .appId }}"
        private-key: "{{ .privateKey }}"
        client-id: "{{ .clientId }}"
        installation-id: "{{ .installationId }}"
  data:
  - secretKey: appId
    remoteRef:
      key: github-apps/5dlabs-cleo
      property: app_id
  - secretKey: privateKey
    remoteRef:
      key: github-apps/5dlabs-cleo
      property: private_key  
  - secretKey: clientId
    remoteRef:
      key: github-apps/5dlabs-cleo
      property: client_id
  - secretKey: installationId
    remoteRef:
      key: github-apps/5dlabs-cleo
      property: installation_id
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-tess
  namespace: agents-platform
  labels:
    app.kubernetes.io/name: github-app-tess
    app.kubernetes.io/component: external-secrets
    app.kubernetes.io/part-of: taskmaster-agents
spec:
  refreshInterval: 1h
  secretStoreRef:
    kind: ClusterSecretStore
    name: aws-secrets-manager
  target:
    name: github-app-5dlabs-tess
    creationPolicy: Owner
    deletionPolicy: Retain
    template:
      type: Opaque
      metadata:
        labels:
          app.kubernetes.io/name: github-app-tess
          app.kubernetes.io/managed-by: external-secrets
      data:
        app-id: "{{ .appId }}"
        private-key: "{{ .privateKey }}"
        client-id: "{{ .clientId }}"
        installation-id: "{{ .installationId }}"
        webhook-secret: "{{ .webhookSecret }}"
  data:
  - secretKey: appId
    remoteRef:
      key: github-apps/5dlabs-tess
      property: app_id
  - secretKey: privateKey
    remoteRef:
      key: github-apps/5dlabs-tess
      property: private_key
  - secretKey: clientId
    remoteRef:
      key: github-apps/5dlabs-tess
      property: client_id
  - secretKey: installationId
    remoteRef:
      key: github-apps/5dlabs-tess
      property: installation_id
  - secretKey: webhookSecret
    remoteRef:
      key: github-apps/5dlabs-tess
      property: webhook_secret
```

### Controller Integration for Secret Mounting
```rust
// In controller/src/tasks/code/resources.rs
fn create_agent_secret_volumes(github_app: &str) -> Vec<Volume> {
    let mut volumes = Vec::new();
    
    match github_app {
        "5DLabs-Cleo" => {
            volumes.push(Volume {
                name: "github-app-credentials".to_string(),
                secret: Some(SecretVolumeSource {
                    secret_name: Some("github-app-5dlabs-cleo".to_string()),
                    default_mode: Some(0o400), // Read-only for security
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        "5DLabs-Tess" => {
            volumes.push(Volume {
                name: "github-app-credentials".to_string(),
                secret: Some(SecretVolumeSource {
                    secret_name: Some("github-app-5dlabs-tess".to_string()),
                    default_mode: Some(0o400),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        "5DLabs-Rex" | _ => {
            // Rex uses existing github-app-5dlabs-rex secret
            volumes.push(Volume {
                name: "github-app-credentials".to_string(),
                secret: Some(SecretVolumeSource {
                    secret_name: Some("github-app-5dlabs-rex".to_string()),
                    default_mode: Some(0o400),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
    }
    
    volumes
}

fn create_agent_volume_mounts() -> Vec<VolumeMount> {
    vec![
        VolumeMount {
            name: "github-app-credentials".to_string(),
            mount_path: "/etc/github-app".to_string(),
            read_only: Some(true),
            ..Default::default()
        }
    ]
}
```

### GitHub API Authentication in Agents
```handlebars
# In container-cleo.sh.hbs
{{#if (eq github_app "5DLabs-Cleo")}}
#!/bin/bash
echo "🎯 Cleo: Setting up GitHub API authentication"

# GitHub App credentials
export GITHUB_APP_ID=$(cat /etc/github-app/app-id 2>/dev/null || echo "")
export GITHUB_INSTALLATION_ID=$(cat /etc/github-app/installation-id 2>/dev/null || echo "")

if [ -n "$GITHUB_APP_ID" ] && [ -n "$GITHUB_INSTALLATION_ID" ]; then
    echo "📱 Generating GitHub App token..."
    
    # Generate installation access token
    GITHUB_TOKEN=$(python3 /usr/local/bin/generate-github-token.py \
        "$GITHUB_APP_ID" \
        "/etc/github-app/private-key" \
        "$GITHUB_INSTALLATION_ID")
    
    export GITHUB_TOKEN
    
    # Verify token works
    if gh auth status --hostname github.com >/dev/null 2>&1; then
        echo "✅ GitHub API authentication successful"
    else
        echo "⚠️  GitHub API authentication failed, some features may not work"
    fi
else
    echo "⚠️  GitHub App credentials not available"
fi
{{/if}}
```

## Architecture Patterns

### Secret Management Flow
```
AWS Secrets Manager → External Secrets Operator → Kubernetes Secret → Pod Volume Mount → Agent Container
```

### Agent-Specific Secret Mapping
- **Rex**: Uses existing `github-app-5dlabs-rex` secret
- **Cleo**: Uses new `github-app-5dlabs-cleo` secret for PR labeling
- **Tess**: Uses new `github-app-5dlabs-tess` secret for PR reviews

### Automatic Rotation
External Secrets refreshes secrets every hour, ensuring:
- Credential rotation compliance
- Automatic token refresh
- Minimal downtime during rotation

## Testing Strategy

### Secret Creation Testing
1. **External Secret Validation**
   ```bash
   # Apply External Secret configurations
   kubectl apply -f external-secrets-cleo.yaml
   kubectl apply -f external-secrets-tess.yaml
   
   # Verify External Secrets created successfully
   kubectl get externalsecret -n agents-platform
   
   # Check secret creation
   kubectl get secret github-app-5dlabs-cleo -n agents-platform -o yaml
   kubectl get secret github-app-5dlabs-tess -n agents-platform -o yaml
   ```

2. **Secret Content Validation**
   ```bash
   # Verify all required keys present
   kubectl get secret github-app-5dlabs-cleo -o jsonpath='{.data}' | jq keys
   
   # Test base64 decoding works
   kubectl get secret github-app-5dlabs-cleo -o jsonpath='{.data.app-id}' | base64 -d
   ```

### Agent Integration Testing
1. **Secret Mounting Verification**
   ```bash
   # Create test CodeRun for Cleo
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: test-cleo-secrets
   spec:
     github_app: "5DLabs-Cleo"
     service: "cto"
   EOF
   
   # Verify secret mounted correctly
   kubectl exec test-cleo-pod -- ls -la /etc/github-app/
   kubectl exec test-cleo-pod -- cat /etc/github-app/app-id
   ```

2. **GitHub API Authentication Testing**
   ```bash
   # Test GitHub token generation inside agent container
   kubectl exec test-cleo-pod -- generate-github-token.sh
   
   # Test GitHub API access
   kubectl exec test-cleo-pod -- gh api /user
   ```

## Key Design Decisions

1. **Separate GitHub Apps**: Each agent has its own GitHub App for security and audit isolation
2. **External Secrets Integration**: Follows existing pattern for consistent credential management
3. **Automatic Rotation**: 1-hour refresh interval balances security and performance
4. **Template-Based Secret Generation**: Consistent secret structure across all agents
5. **Volume Mount Security**: Read-only mounts with restricted file permissions

## Security Considerations

### Secret Access Control
- Secrets mounted read-only in agent containers
- File permissions set to 0400 for private key protection
- Each agent only accesses its own GitHub App credentials

### Credential Isolation
- Separate GitHub Apps prevent cross-agent access
- External Secrets operator manages credential lifecycle
- AWS Secrets Manager provides backend security

### Audit and Monitoring
- External Secrets logs all sync operations
- Secret access tracked through Kubernetes audit logs
- GitHub App activity visible in GitHub audit logs

## References

- [External Secrets Operator Documentation](https://external-secrets.io/)
- [GitHub Apps Authentication](https://docs.github.com/en/developers/apps/building-github-apps/authenticating-with-github-apps)
- [AWS Secrets Manager Integration](https://external-secrets.io/v0.7.2/provider/aws-secrets-manager/)
- [Kubernetes Secret Management Best Practices](https://kubernetes.io/docs/concepts/configuration/secret/)