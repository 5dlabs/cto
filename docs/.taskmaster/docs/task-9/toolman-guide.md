# Toolman Guide: External Secrets for Agent Apps

## Overview

This task sets up External Secrets resources for Cleo and Tess GitHub Apps to enable secure credential management for multi-agent orchestration. You'll create External Secrets configurations that provide agent-specific GitHub App authentication following existing patterns.

## Tool Selection Strategy

### Primary Development Tools

**filesystem** - Essential for External Secrets configuration and integration
- Create External Secret YAML configurations for Cleo and Tess
- Read existing External Secrets patterns for consistency
- Modify controller code for agent-specific secret mounting
- Create helper scripts for GitHub token generation

**kubernetes** - Critical for Kubernetes resource management and testing
- Apply External Secret configurations to cluster
- Test secret creation and mounting in agent pods
- Validate secret content and structure
- Debug External Secrets sync status and issues

### Research and Documentation Tools

**memory_create_entities** - Store implementation knowledge
- Document External Secrets patterns and configuration options
- Track secret structure requirements and mapping strategies
- Remember GitHub App authentication flows and token generation
- Store testing scenarios and validation approaches

**brave_web_search** - Supplemental research tool
- Research External Secrets Operator configuration best practices
- Find GitHub App authentication and JWT generation examples
- Research Kubernetes secret mounting and security patterns
- Lookup AWS Secrets Manager integration approaches

## Implementation Workflow

### Phase 1: Research and Analysis
```
Tools: filesystem, memory_create_entities, brave_web_search
```

1. **Examine Existing External Secrets**
   - Use `filesystem` to read current External Secrets configurations
   - Study existing secret structures and naming patterns
   - Document current ClusterSecretStore setup and capabilities

2. **Research GitHub App Authentication**
   - Use `brave_web_search` for GitHub App JWT generation examples
   - Study GitHub App installation token flow
   - Plan secret structure requirements for each agent

3. **Plan Agent-Specific Requirements**
   - Use `memory_create_entities` to document each agent's secret needs
   - Define secret mapping from backend store to Kubernetes secrets
   - Plan controller integration for agent-specific secret mounting

### Phase 2: External Secrets Configuration
```
Tools: filesystem, kubernetes, memory_create_entities
```

1. **Create Cleo External Secret**
   ```yaml
   # Focus areas for Cleo External Secret
   - Proper secret store references and data mapping
   - Required credentials: app-id, private-key, client-id, installation-id
   - Template configuration for proper secret generation
   - Security labels and metadata
   ```

2. **Create Tess External Secret**
   ```yaml
   # Focus areas for Tess External Secret
   - Enhanced secret requirements including webhook-secret
   - Testing and deployment specific credential needs
   - Proper backend path configuration
   - Error handling and validation
   ```

3. **Test External Secrets Deployment**
   - Use `kubernetes` to apply External Secret configurations
   - Validate secrets are created correctly
   - Test secret content and structure match requirements

### Phase 3: Controller Integration
```
Tools: filesystem, kubernetes, memory_create_entities
```

1. **Modify Controller Secret Mounting**
   ```rust
   // Focus areas for controller changes
   - Agent-specific secret selection based on github_app field
   - Volume and volume mount creation for each agent type
   - Error handling for missing or invalid secrets
   - Backward compatibility with existing Rex workflows
   ```

2. **Test Agent Secret Integration**
   ```bash
   # Testing secret mounting in agent pods
   - Create test CodeRun CRDs for each agent type
   - Verify correct secrets mounted in containers
   - Test secret file permissions and accessibility
   - Validate environment variable setup
   ```

3. **Validate GitHub Authentication**
   - Test GitHub token generation from mounted secrets
   - Verify GitHub API access works for each agent
   - Test GitHub CLI integration and functionality

### Phase 4: Template Integration and Testing
```
Tools: filesystem, kubernetes, memory_create_entities
```

1. **Update Agent Container Templates**
   - Modify container templates to use agent-specific credentials
   - Add GitHub authentication setup for each agent type
   - Include token generation and GitHub CLI setup

2. **Integration Testing**
   - Test complete agent lifecycle with new secrets
   - Test secret rotation behavior with running agents
   - Test GitHub API operations from agent containers

3. **Security and Performance Validation**
   - Validate secret mounting security (read-only, proper permissions)
   - Test External Secrets sync performance and reliability
   - Test error handling for various failure scenarios

## Best Practices

### External Secrets Configuration
- **Consistent Patterns**: Follow existing External Secrets naming and structure
- **Security First**: Use proper labels, annotations, and creation policies
- **Error Handling**: Include proper error handling for sync failures
- **Backend Organization**: Organize secrets logically in backend store

### Secret Content Management
- **Complete Credentials**: Ensure all required GitHub App credentials included
- **Proper Encoding**: Use correct base64 encoding for secret data
- **Template Structure**: Use consistent template structure for secret generation
- **Validation**: Include validation for required fields and formats

### Controller Integration
- **Agent-Specific Logic**: Clear logic for mapping agents to secrets
- **Error Handling**: Graceful handling of missing or invalid secrets
- **Security**: Read-only mounts with appropriate file permissions
- **Backward Compatibility**: Ensure existing workflows continue working

## Tool Usage Examples

### Reading Existing Configurations
```bash
# Use filesystem to examine current External Secrets
filesystem.read_file("infra/external-secrets/github-app-rex.yaml")
filesystem.list_directory("infra/external-secrets/")
```

### Testing Kubernetes Resources
```bash
# Use kubernetes for External Secrets operations
kubernetes.kubectl_apply("external-secrets-cleo.yaml")
kubernetes.kubectl_get("externalsecret", "-n", "agents-platform")
kubernetes.kubectl_describe("secret", "github-app-5dlabs-cleo")
```

### Creating Configurations
```bash
# Use filesystem to create External Secret configurations
filesystem.write_file("infra/external-secrets/github-app-cleo.yaml", cleo_config)
filesystem.write_file("infra/external-secrets/github-app-tess.yaml", tess_config)
```

### Documentation and Tracking
```bash
# Use memory_create_entities to document requirements
memory_create_entities([{
  "name": "GitHub App Secret Structure",
  "description": "Required fields for agent GitHub App authentication",
  "fields": ["app-id", "private-key", "client-id", "installation-id"]
}])
```

## Common Pitfalls to Avoid

1. **Incomplete Secret Structure**: Ensure all required GitHub App fields included
2. **Backend Path Errors**: Verify secret store paths exist and are accessible
3. **Permission Issues**: Set correct file permissions for private keys
4. **Template Syntax Errors**: Validate External Secret template syntax
5. **Controller Integration**: Test secret mounting works with controller changes
6. **Rotation Impact**: Ensure secret rotation doesn't disrupt running agents

## External Secrets Configuration Patterns

### Basic External Secret Structure
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
```

### Secret Store Backend Structure
```json
{
  "app_id": "123456",
  "private_key": "-----BEGIN RSA PRIVATE KEY-----...",
  "client_id": "Iv1.abcdef123456",
  "installation_id": "78901234"
}
```

### Controller Secret Mounting Pattern
```rust
match github_app {
    "5DLabs-Cleo" => mount_secret("github-app-5dlabs-cleo"),
    "5DLabs-Tess" => mount_secret("github-app-5dlabs-tess"),
    _ => mount_secret("github-app-5dlabs-rex"),
}
```

## Success Validation

### External Secrets Quality Checks
- [ ] External Secrets apply successfully to cluster
- [ ] Kubernetes secrets generated with correct structure
- [ ] All required GitHub App credentials present in secrets
- [ ] Secret sync status shows healthy state

### Controller Integration Quality Checks
- [ ] Controller mounts correct secrets for each agent type
- [ ] Secret volumes and mounts configured properly
- [ ] Error handling works for missing secrets
- [ ] Backward compatibility maintained for Rex agents

### Agent Authentication Quality Checks
- [ ] Agents can read secret files from mounted volumes
- [ ] GitHub token generation works from secret credentials
- [ ] GitHub API authentication succeeds for all agents
- [ ] GitHub CLI integration functions correctly

### Security and Performance Checks
- [ ] Secrets mounted read-only with correct permissions
- [ ] Each agent only accesses its designated secrets
- [ ] Secret rotation doesn't disrupt running agents
- [ ] External Secrets sync performance meets requirements

This implementation requires careful attention to existing patterns while providing secure, isolated credential management for each agent. Focus on creating reliable External Secrets configurations that integrate seamlessly with the controller and provide robust GitHub authentication for specialized agent workflows.