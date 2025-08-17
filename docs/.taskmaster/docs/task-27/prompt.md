# Autonomous Agent Prompt: Comprehensive Admin Access for Tess Agent

## Objective

Configure comprehensive administrative access for the Tess agent, providing full infrastructure credentials for PostgreSQL, Redis, and Argo CD systems. Implement secure secret management, automated credential rotation, and comprehensive audit logging beyond standard Kubernetes RBAC.

## Context

You are configuring critical infrastructure access for the Tess agent in the Task Master platform. Tess requires elevated privileges for live deployment testing, database administration, and infrastructure management operations. This configuration must balance security with operational flexibility while maintaining comprehensive audit trails.

## Implementation Requirements

### External Secrets Management

Configure External Secrets Operator integration:

1. **Secret Store Configuration**
   - Set up AWS Secrets Manager or HashiCorp Vault backend
   - Configure authentication using IAM roles or service account tokens
   - Implement secret path organization: `/infrastructure/tess-admin/{service}`
   - Enable automatic secret refresh every 24 hours

2. **PostgreSQL Admin Secrets**
   - Store connection parameters: host, port, username, password, database
   - Create connection URL format for application consumption
   - Implement SUPERUSER privileges for full database access
   - Configure SSL/TLS certificate requirements

3. **Redis Admin Secrets**
   - Store cluster connection details and authentication
   - Configure ACL-based permissions for comprehensive access
   - Support both single-instance and cluster configurations
   - Include connection pooling parameters

4. **Argo CD Admin Access**
   - Generate long-lived admin token with full permissions
   - Create dedicated AppProject with elevated privileges
   - Configure server URL and authentication details
   - Enable application lifecycle management capabilities

### Kubernetes Integration

Implement comprehensive Kubernetes access:

1. **Service Account Configuration**
   - Create dedicated service account with admin permissions
   - Configure IAM role binding for AWS integration
   - Enable pod identity and workload identity where applicable
   - Implement proper annotation for external systems

2. **RBAC Configuration**
   - Create ClusterRole with comprehensive permissions
   - Grant access to secrets, configmaps, and application resources
   - Enable Argo Workflows and Applications management
   - Configure External Secrets access permissions

3. **Pod Security Configuration**
   - Implement init container for connection validation
   - Configure proper security context and capabilities
   - Mount TLS certificates for encrypted connections
   - Enable audit logging for all administrative operations

### Database Admin Privileges

Configure database-level administrative access:

1. **PostgreSQL Setup**
   - Create SUPERUSER account with full privileges
   - Grant CREATEDB and CREATEROLE permissions
   - Configure database-level auditing and logging
   - Implement connection pooling and timeout settings

2. **Redis Configuration**
   - Configure ACL with +@all permissions (excluding dangerous commands)
   - Enable command logging and audit trail
   - Configure cluster endpoint discovery
   - Implement connection retry and failover logic

### Security Implementation

Implement comprehensive security measures:

1. **Certificate Management**
   - Use cert-manager for TLS certificate lifecycle
   - Configure client certificates for database connections
   - Implement certificate rotation policies
   - Enable mutual TLS where supported

2. **Audit Logging**
   - Configure Kubernetes audit policy for Tess operations
   - Enable database query logging and monitoring
   - Implement structured logging with correlation IDs
   - Create audit trail for all administrative actions

3. **Break-Glass Access**
   - Configure emergency access procedures
   - Implement time-limited elevated permissions
   - Create approval workflows for emergency access
   - Document recovery procedures and contact information

### Monitoring and Alerting

Configure comprehensive monitoring:

1. **Connection Health Monitoring**
   - Implement liveness and readiness probes
   - Monitor database connection pools and performance
   - Track Redis cluster health and failover events
   - Monitor Argo CD API availability and response times

2. **Security Monitoring**
   - Alert on failed authentication attempts
   - Monitor for suspicious administrative operations
   - Track secret rotation events and failures
   - Implement anomaly detection for admin activities

3. **Performance Monitoring**
   - Track database query performance and slow queries
   - Monitor Redis memory usage and key distribution
   - Measure Argo CD operation latency and success rates
   - Monitor secret refresh cycles and errors

## Expected Deliverables

1. **External Secrets Configuration**
   - SecretStore definitions for all backends
   - ExternalSecret resources for each service
   - Refresh policies and rotation strategies

2. **Kubernetes Manifests**
   - Service account and RBAC configurations
   - Pod specifications with security context
   - Init containers for validation and setup

3. **Database Configurations**
   - PostgreSQL user creation and privilege scripts
   - Redis ACL configuration and cluster setup
   - Connection pooling and performance tuning

4. **Argo CD Integration**
   - AppProject configuration with admin permissions
   - Service account integration and token management
   - Application deployment and lifecycle policies

5. **Monitoring Setup**
   - ServiceMonitor and PrometheusRule configurations
   - Grafana dashboard specifications
   - Alert routing and notification policies

6. **Security Documentation**
   - Audit policy configurations
   - Break-glass access procedures
   - Security incident response playbooks

## Acceptance Criteria

- Tess agent connects successfully to PostgreSQL with SUPERUSER privileges
- Redis admin operations (CONFIG SET, FLUSHDB) execute without errors
- Argo CD applications can be created, modified, and deleted via Tess
- Secret rotation occurs automatically every 24 hours
- TLS certificates validate properly for all encrypted connections
- Audit logs capture all administrative operations with proper correlation
- Break-glass access procedures work under emergency conditions
- Monitoring alerts fire appropriately for connection failures
- Init container validation blocks pod startup on credential failures
- Other agents cannot access Tess administrative secrets

## Quality Standards

- Follow Kubernetes security best practices
- Implement principle of least privilege where possible
- Use structured configuration with proper versioning
- Include comprehensive error handling and recovery
- Maintain audit trail for all administrative operations
- Document all access paths and permission matrices
- Test all failure scenarios and recovery procedures
- Implement proper secret lifecycle management

## Security Requirements

- Store all credentials in external secret management systems
- Use encrypted connections for all database communication
- Implement proper access controls and audit logging
- Follow secure coding practices for credential handling
- Enable monitoring for unauthorized access attempts
- Implement emergency access controls with approval workflows
- Document all security configurations and procedures

## Resources

- External Secrets Operator documentation and examples
- Kubernetes RBAC and security best practices
- PostgreSQL and Redis administrative documentation
- Argo CD administration and security guides
- Cert-manager certificate lifecycle management
- Prometheus monitoring and alerting configurations

Focus on creating a secure, well-monitored administrative access system that enables Tess to perform infrastructure operations while maintaining comprehensive security and audit capabilities.