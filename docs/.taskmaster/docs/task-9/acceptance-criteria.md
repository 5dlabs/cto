# Acceptance Criteria: External Secrets for Agent Apps

## Core External Secrets Configuration Requirements

### ✅ External Secret Resource Creation
- [ ] **Cleo External Secret**: `github-app-5dlabs-cleo` ExternalSecret resource created
- [ ] **Tess External Secret**: `github-app-5dlabs-tess` ExternalSecret resource created  
- [ ] **Namespace Targeting**: Both External Secrets deployed to `agents-platform` namespace
- [ ] **Resource Labels**: Proper labels applied for resource organization and management
- [ ] **Secret Store Reference**: Correct ClusterSecretStore referenced for credential backend

### ✅ Secret Store Integration
- [ ] **Backend Configuration**: AWS Secrets Manager paths configured for both agents
- [ ] **Credential Structure**: Proper secret key mapping from store to Kubernetes secret
- [ ] **Access Permissions**: External Secrets Operator has required permissions to access backend
- [ ] **Path Organization**: Secrets organized logically in backend store (github-apps/5dlabs-cleo, github-apps/5dlabs-tess)
- [ ] **Data Validation**: All required credential fields present in backend store

## Secret Content and Structure Requirements

### ✅ Cleo Secret Requirements
- [ ] **App ID**: GitHub App ID for 5DLabs-Cleo app present and valid
- [ ] **Private Key**: RSA private key for JWT signing present and properly formatted
- [ ] **Client ID**: OAuth client ID for GitHub App present and valid
- [ ] **Installation ID**: GitHub App installation ID for repository access present
- [ ] **Secret Format**: All values properly base64 encoded in Kubernetes secret

### ✅ Tess Secret Requirements  
- [ ] **App ID**: GitHub App ID for 5DLabs-Tess app present and valid
- [ ] **Private Key**: RSA private key for JWT signing present and properly formatted
- [ ] **Client ID**: OAuth client ID for GitHub App present and valid
- [ ] **Installation ID**: GitHub App installation ID for repository access present
- [ ] **Webhook Secret**: Webhook validation secret present for PR event validation

### ✅ Secret Generation and Templates
- [ ] **Template Configuration**: External Secret template generates correct secret structure
- [ ] **Key Mapping**: All backend keys properly mapped to Kubernetes secret data
- [ ] **Encoding**: All sensitive values properly base64 encoded
- [ ] **Metadata**: Generated secrets include proper labels and annotations
- [ ] **Creation Policy**: Secrets created with correct ownership and lifecycle management

## Refresh and Rotation Requirements

### ✅ Automatic Refresh Configuration
- [ ] **Refresh Interval**: 1-hour refresh interval configured for both secrets
- [ ] **Rotation Testing**: Secret rotation works without disrupting running agents
- [ ] **Sync Status**: External Secrets sync status shows healthy state
- [ ] **Error Handling**: Sync failures logged and alerted appropriately
- [ ] **Backend Connectivity**: Continuous connectivity to secret store maintained

### ✅ Rotation Impact Management
- [ ] **Running Agent Protection**: Secret rotation doesn't interrupt running agents
- [ ] **Token Refresh**: GitHub tokens refreshed when underlying secrets rotate
- [ ] **Graceful Updates**: Pod restart handled gracefully during secret updates
- [ ] **Consistency**: All agent containers see consistent secret versions
- [ ] **Rollback Support**: Failed rotations can be rolled back safely

## Controller Integration Requirements

### ✅ Secret Volume Mounting
- [ ] **Agent-Specific Mounting**: Correct secrets mounted based on `github_app` field
- [ ] **Cleo Agent**: Cleo agents get `github-app-5dlabs-cleo` secret mounted
- [ ] **Tess Agent**: Tess agents get `github-app-5dlabs-tess` secret mounted
- [ ] **Rex Agent**: Rex agents continue using existing `github-app-5dlabs-rex` secret
- [ ] **Mount Path**: Secrets mounted at `/etc/github-app/` in all agent containers

### ✅ Controller Code Changes
- [ ] **Secret Selection Logic**: Controller selects correct secret based on agent type
- [ ] **Volume Creation**: Proper volume configurations created for each agent type
- [ ] **Mount Configuration**: Volume mounts configured with correct paths and permissions
- [ ] **Error Handling**: Missing secrets handled gracefully with clear error messages
- [ ] **Backward Compatibility**: Existing Rex workflows continue working unchanged

## Security Requirements

### ✅ Secret Access Control
- [ ] **Read-Only Mounts**: All secrets mounted read-only in agent containers
- [ ] **File Permissions**: Private keys have 0400 permissions (read-only for owner)
- [ ] **Service Account Isolation**: Each agent uses appropriate service account
- [ ] **Namespace Isolation**: Secrets properly isolated within agents-platform namespace
- [ ] **RBAC Compliance**: External Secrets Operator has minimal required permissions

### ✅ Credential Isolation
- [ ] **Agent Separation**: Each agent only accesses its own GitHub App credentials
- [ ] **Cross-Agent Protection**: Agents cannot access other agents' secrets
- [ ] **Secret Cleanup**: Secrets cleaned up when agents complete
- [ ] **Audit Trail**: All secret access logged for security auditing
- [ ] **Network Security**: Secret transmission encrypted in transit

## Agent Template Integration Requirements

### ✅ Container Template Updates
- [ ] **Authentication Setup**: Agent containers set up GitHub authentication correctly
- [ ] **Environment Variables**: Required GitHub App variables set from secret files
- [ ] **Token Generation**: GitHub installation tokens generated successfully
- [ ] **API Access**: Agents can authenticate with GitHub API using generated tokens
- [ ] **Fallback Handling**: Graceful behavior when GitHub credentials unavailable

### ✅ Agent-Specific Configuration
- [ ] **Cleo Template**: Container-cleo.sh.hbs includes GitHub API setup for PR labeling
- [ ] **Tess Template**: Container-tess.sh.hbs includes GitHub API setup for PR reviews
- [ ] **Rex Compatibility**: Existing Rex templates continue working with current secrets
- [ ] **Conditional Logic**: Template logic correctly handles different agent types
- [ ] **Error Handling**: Template errors handled gracefully with clear messages

## Testing Requirements

### ✅ Secret Creation Testing
- [ ] **ExternalSecret Apply**: External Secret resources apply successfully to cluster
- [ ] **Secret Generation**: Kubernetes secrets generated correctly from External Secrets
- [ ] **Content Validation**: Secret content matches expected structure and values
- [ ] **Sync Status**: External Secrets show healthy sync status
- [ ] **Backend Connectivity**: External Secrets can access secret store backend

### ✅ Agent Integration Testing
- [ ] **Cleo CodeRun Test**: Create Cleo CodeRun and verify secret mounting works
- [ ] **Tess CodeRun Test**: Create Tess CodeRun and verify secret mounting works
- [ ] **Secret Access Test**: Agent containers can read all required secret files
- [ ] **GitHub Auth Test**: Agents can generate valid GitHub tokens from secrets
- [ ] **API Operation Test**: Agents can perform GitHub API operations successfully

### ✅ Rotation and Lifecycle Testing
- [ ] **Secret Rotation Test**: Trigger secret rotation and verify agents continue working
- [ ] **Sync Failure Test**: Test behavior when secret store is unavailable
- [ ] **Invalid Secret Test**: Test handling of corrupted or invalid secret data
- [ ] **Pod Restart Test**: Test agent restart behavior during secret updates
- [ ] **Cleanup Test**: Test secret cleanup when CodeRun resources deleted

## GitHub Integration Requirements

### ✅ Authentication Functionality
- [ ] **JWT Generation**: Agents can generate valid GitHub App JWTs
- [ ] **Installation Token**: Agents can obtain installation access tokens
- [ ] **API Authentication**: GitHub API calls authenticate successfully
- [ ] **Permission Validation**: Agents have required GitHub App permissions
- [ ] **Token Refresh**: Tokens refresh automatically before expiration

### ✅ Agent-Specific GitHub Operations
- [ ] **Cleo Operations**: Cleo can perform PR labeling operations
- [ ] **Tess Operations**: Tess can perform PR review and approval operations
- [ ] **Repository Access**: Agents can access target repositories through GitHub App
- [ ] **Webhook Validation**: Tess can validate incoming webhook events
- [ ] **Rate Limit Handling**: GitHub API rate limiting handled appropriately

## Performance Requirements

### ✅ Secret Management Performance
- [ ] **Secret Sync Time**: External Secrets sync within 30 seconds of changes
- [ ] **Agent Startup Time**: Secret mounting doesn't significantly delay agent startup
- [ ] **Token Generation Time**: GitHub token generation completes within 10 seconds
- [ ] **Rotation Impact**: Secret rotation completes within 5 minutes
- [ ] **Resource Usage**: External Secrets don't consume excessive cluster resources

### ✅ Scalability Requirements
- [ ] **Multiple Agents**: System handles multiple concurrent agents per type
- [ ] **High Frequency Rotation**: Frequent secret rotations don't cause performance issues
- [ ] **Large Secret Content**: Large private keys don't cause mounting issues
- [ ] **Concurrent Access**: Multiple pods can access same secrets simultaneously
- [ ] **Secret Store Load**: Secret store can handle required access patterns

## Monitoring and Observability Requirements

### ✅ Logging and Metrics
- [ ] **Sync Logging**: External Secrets sync operations logged with details
- [ ] **Error Logging**: All secret-related errors logged with context
- [ ] **Performance Metrics**: Secret sync timing and success rates tracked
- [ ] **Agent Metrics**: GitHub authentication success/failure rates tracked
- [ ] **Security Audit**: Secret access events logged for security monitoring

### ✅ Alerting Configuration
- [ ] **Sync Failure Alerts**: Alerts configured for External Secrets sync failures
- [ ] **Secret Expiration Alerts**: Alerts for secrets approaching expiration
- [ ] **Authentication Failure Alerts**: Alerts for GitHub authentication failures
- [ ] **Performance Alerts**: Alerts for slow secret operations or high error rates
- [ ] **Security Alerts**: Alerts for suspicious secret access patterns

## Documentation Requirements

### ✅ Technical Documentation
- [ ] **Implementation Guide**: Complete External Secrets setup documented
- [ ] **Configuration Reference**: All External Secret options and settings documented
- [ ] **Integration Guide**: Controller and template integration documented
- [ ] **Security Guide**: Security best practices and requirements documented
- [ ] **Troubleshooting Guide**: Common issues and resolution steps documented

### ✅ Operational Documentation
- [ ] **Deployment Procedures**: How to deploy External Secrets for new agents
- [ ] **Rotation Procedures**: How to handle secret rotation and updates
- [ ] **Monitoring Procedures**: How to monitor External Secrets health and performance
- [ ] **Emergency Procedures**: How to handle External Secrets failures and recovery
- [ ] **Maintenance Procedures**: Regular maintenance tasks and schedules

## Validation Checklist

Before marking this task complete, verify:

1. **Secret Creation**: All External Secrets create proper Kubernetes secrets
2. **Agent Integration**: All agent types can access their designated secrets
3. **Authentication**: GitHub API authentication works for all agent types
4. **Security**: All security requirements met with proper access controls
5. **Performance**: System meets all performance and scalability requirements
6. **Testing**: All unit, integration, and operational tests pass
7. **Documentation**: All technical and operational documentation complete
8. **Monitoring**: All logging, metrics, and alerting properly configured

## Success Metrics

- **Secret Sync Success Rate**: >99% of External Secrets sync operations succeed
- **Agent Authentication Success Rate**: >99% of GitHub authentication attempts succeed
- **Secret Rotation Impact**: <2 minutes downtime during secret rotation
- **Agent Startup Time**: <60 seconds additional time for secret mounting
- **System Availability**: External Secrets system maintains >99.9% availability