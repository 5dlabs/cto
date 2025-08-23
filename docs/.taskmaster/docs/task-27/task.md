# Task 27: Configure Comprehensive Admin Access for Tess Agent

## Overview

This task configures comprehensive administrative access for the Tess agent, providing full infrastructure credentials for databases, Argo CD, and other critical systems. The implementation goes beyond standard Kubernetes RBAC to enable live deployment testing and administrative operations.

## Technical Requirements

### Infrastructure Access Components

1. **Database Administrative Access**
   - PostgreSQL superuser credentials (host, port, username, password, database)
   - Redis admin access (host, port, password, cluster endpoints)
   - Connection pooling and TLS certificate management

2. **Argo CD Administrative Rights**
   - Admin token for full Argo CD operations
   - AppProject configuration with elevated permissions
   - Application lifecycle management capabilities

3. **Secret Management Infrastructure**
   - External Secrets Operator integration
   - AWS Secrets Manager or HashiCorp Vault backend
   - Dynamic secret injection via Kubernetes CSI

## Implementation Guide

### Step 1: External Secrets Configuration

Create External Secrets Store configuration:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: tess-admin-secrets
  namespace: taskmaster
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-west-2
      auth:
        secretRef:
          accessKeyId:
            name: aws-creds
            key: access-key-id
          secretAccessKey:
            name: aws-creds
            key: secret-access-key
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: tess-postgres-admin
  namespace: taskmaster
spec:
  secretStoreRef:
    name: tess-admin-secrets
    kind: SecretStore
  target:
    name: tess-postgres-admin
    creationPolicy: Owner
  data:
  - secretKey: host
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: host
  - secretKey: port
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: port
  - secretKey: username
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: username
  - secretKey: password
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: password
  - secretKey: database
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: database
  - secretKey: url
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: connection_url
  refreshInterval: 24h
```

### Step 2: Redis Admin Secrets

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: tess-redis-admin
  namespace: taskmaster
spec:
  secretStoreRef:
    name: tess-admin-secrets
    kind: SecretStore
  target:
    name: tess-redis-admin
    creationPolicy: Owner
  data:
  - secretKey: host
    remoteRef:
      key: /infrastructure/tess-admin/redis
      property: host
  - secretKey: port
    remoteRef:
      key: /infrastructure/tess-admin/redis
      property: port
  - secretKey: password
    remoteRef:
      key: /infrastructure/tess-admin/redis
      property: password
  - secretKey: cluster-endpoints
    remoteRef:
      key: /infrastructure/tess-admin/redis
      property: cluster_endpoints
  - secretKey: url
    remoteRef:
      key: /infrastructure/tess-admin/redis
      property: connection_url
  refreshInterval: 24h
```

### Step 3: Argo CD Admin Configuration

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: tess-argocd-admin
  namespace: taskmaster
spec:
  secretStoreRef:
    name: tess-admin-secrets
    kind: SecretStore
  target:
    name: tess-argocd-admin
    creationPolicy: Owner
  data:
  - secretKey: token
    remoteRef:
      key: /infrastructure/tess-admin/argocd
      property: admin_token
  - secretKey: server
    remoteRef:
      key: /infrastructure/tess-admin/argocd
      property: server_url
  - secretKey: username
    remoteRef:
      key: /infrastructure/tess-admin/argocd
      property: admin_username
  refreshInterval: 24h
---
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: tess-admin-project
  namespace: argocd
spec:
  description: "Administrative project for Tess agent operations"
  sourceRepos:
  - 'https://github.com/5dlabs/cto'
  - '*'
  destinations:
  - namespace: '*'
    server: '*'
  clusterResourceWhitelist:
  - group: '*'
    kind: '*'
  namespaceResourceWhitelist:
  - group: '*'
    kind: '*'
  roles:
  - name: tess-admin
    description: "Full administrative access for Tess agent"
    policies:
    - p, proj:tess-admin-project:tess-admin, applications, *, tess-admin-project/*, allow
    - p, proj:tess-admin-project:tess-admin, repositories, *, *, allow
    - p, proj:tess-admin-project:tess-admin, clusters, *, *, allow
    - p, proj:tess-admin-project:tess-admin, exec, *, *, allow
    groups:
    - tess-service-account
```

### Step 4: Tess Agent Pod Configuration

Update the Tess agent deployment with admin access:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tess-agent
  namespace: taskmaster
spec:
  template:
    spec:
      serviceAccountName: tess-admin-service-account
      initContainers:
      - name: secret-validation
        image: postgres:15-alpine
        command: ["/bin/sh", "-c"]
        args:
        - |
          # Validate PostgreSQL connection
          echo "Testing PostgreSQL connection..."
          PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USERNAME" -d "$POSTGRES_DATABASE" -c "SELECT version();"

          # Validate Redis connection
          echo "Testing Redis connection..."
          redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" -a "$REDIS_PASSWORD" ping

          # Validate Argo CD access
          echo "Testing Argo CD access..."
          argocd login "$ARGOCD_SERVER" --username "$ARGOCD_USERNAME" --password "$ARGOCD_TOKEN" --insecure
          argocd cluster list

          echo "All connections validated successfully"
        env:
        - name: POSTGRES_HOST
          valueFrom:
            secretKeyRef:
              name: tess-postgres-admin
              key: host
        - name: POSTGRES_PORT
          valueFrom:
            secretKeyRef:
              name: tess-postgres-admin
              key: port
        - name: POSTGRES_USERNAME
          valueFrom:
            secretKeyRef:
              name: tess-postgres-admin
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: tess-postgres-admin
              key: password
        - name: POSTGRES_DATABASE
          valueFrom:
            secretKeyRef:
              name: tess-postgres-admin
              key: database
        - name: REDIS_HOST
          valueFrom:
            secretKeyRef:
              name: tess-redis-admin
              key: host
        - name: REDIS_PORT
          valueFrom:
            secretKeyRef:
              name: tess-redis-admin
              key: port
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: tess-redis-admin
              key: password
        - name: ARGOCD_SERVER
          valueFrom:
            secretKeyRef:
              name: tess-argocd-admin
              key: server
        - name: ARGOCD_USERNAME
          valueFrom:
            secretKeyRef:
              name: tess-argocd-admin
              key: username
        - name: ARGOCD_TOKEN
          valueFrom:
            secretKeyRef:
              name: tess-argocd-admin
              key: token
      containers:
      - name: tess-agent
        image: taskmaster/tess-agent:latest
        env:
        - name: POSTGRES_ADMIN_URL
          valueFrom:
            secretKeyRef:
              name: tess-postgres-admin
              key: url
        - name: REDIS_ADMIN_URL
          valueFrom:
            secretKeyRef:
              name: tess-redis-admin
              key: url
        - name: ARGOCD_TOKEN
          valueFrom:
            secretKeyRef:
              name: tess-argocd-admin
              key: token
        - name: ARGOCD_SERVER
          valueFrom:
            secretKeyRef:
              name: tess-argocd-admin
              key: server
        volumeMounts:
        - name: tls-certs
          mountPath: /etc/ssl/certs/custom
          readOnly: true
        - name: config-volume
          mountPath: /etc/tess-config
          readOnly: true
      volumes:
      - name: tls-certs
        secret:
          secretName: tess-tls-certificates
      - name: config-volume
        configMap:
          name: tess-admin-config
```

### Step 5: ConfigMap for Non-Sensitive Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: tess-admin-config
  namespace: taskmaster
data:
  database.yaml: |
    postgres:
      max_connections: 100
      connection_timeout: 30s
      ssl_mode: require
      application_name: tess-admin
    redis:
      max_connections: 50
      connection_timeout: 5s
      read_timeout: 30s
      write_timeout: 30s
  argocd.yaml: |
    server:
      timeout: 60s
      retry_attempts: 3
      retry_delay: 5s
    operations:
      sync_timeout: 300s
      health_check_timeout: 60s
  monitoring.yaml: |
    audit_log: true
    metrics_enabled: true
    log_level: info
    performance_tracking: true
```

### Step 6: Service Account and RBAC

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: tess-admin-service-account
  namespace: taskmaster
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::ACCOUNT:role/TessAdminRole
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-admin-role
rules:
- apiGroups: [""]
  resources: ["secrets", "configmaps", "pods", "services"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["argoproj.io"]
  resources: ["workflows", "workflowtemplates", "applications"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["external-secrets.io"]
  resources: ["externalsecrets", "secretstores"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: tess-admin-binding
subjects:
- kind: ServiceAccount
  name: tess-admin-service-account
  namespace: taskmaster
roleRef:
  kind: ClusterRole
  name: tess-admin-role
  apiGroup: rbac.authorization.k8s.io
```

### Step 7: Database Admin Privileges

PostgreSQL admin setup:

```sql
-- Create admin user for Tess
CREATE USER tess_admin WITH SUPERUSER CREATEDB CREATEROLE LOGIN PASSWORD 'secure-password';

-- Grant additional privileges
GRANT ALL PRIVILEGES ON DATABASE taskmaster TO tess_admin;
GRANT ALL ON SCHEMA public TO tess_admin;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO tess_admin;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO tess_admin;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO tess_admin;

-- Create audit role for monitoring
CREATE ROLE tess_audit;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO tess_audit;
GRANT tess_audit TO tess_admin;
```

Redis admin setup:

```bash
# Configure Redis ACL for Tess admin
redis-cli ACL SETUSER tess_admin on >secure-password ~* &* +@all -flushall -flushdb -shutdown -debug

# Create admin ACL file
cat > /etc/redis/tess-admin-acl.conf <<EOF
user tess_admin on >secure-password ~* &* +@all -flushall -flushdb -shutdown -debug -eval -script
user tess_audit on >audit-password ~* &* +@read -@dangerous
EOF
```

### Step 8: TLS Certificate Management

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: tess-client-cert
  namespace: taskmaster
spec:
  secretName: tess-tls-certificates
  issuerRef:
    name: ca-issuer
    kind: ClusterIssuer
  commonName: tess-admin-client
  dnsNames:
  - tess-admin.taskmaster.local
  usages:
  - client auth
  - digital signature
  - key encipherment
---
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: tess-postgres-cert
  namespace: taskmaster
spec:
  secretName: tess-postgres-tls
  issuerRef:
    name: ca-issuer
    kind: ClusterIssuer
  commonName: postgres-client
  usages:
  - client auth
```

### Step 9: Audit Logging Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: tess-audit-config
  namespace: taskmaster
data:
  audit-policy.yaml: |
    apiVersion: audit.k8s.io/v1
    kind: Policy
    rules:
    - level: RequestResponse
      users: ["system:serviceaccount:taskmaster:tess-admin-service-account"]
      resources:
      - group: ""
        resources: ["secrets", "configmaps"]
      - group: "apps"
        resources: ["deployments"]
      - group: "argoproj.io"
        resources: ["applications", "workflows"]
  postgres-audit.yaml: |
    log_statement: 'all'
    log_min_duration_statement: 0
    log_line_prefix: '%t [%p]: user=%u,db=%d,app=%a,client=%h '
    log_connections: on
    log_disconnections: on
    log_checkpoints: on
    log_lock_waits: on
  redis-audit.yaml: |
    logfile: /var/log/redis/redis-audit.log
    loglevel: notice
    syslog-enabled: yes
    syslog-ident: redis-tess
```

## Security Considerations

### Access Control Matrix

| Resource | Tess Admin Access | Audit Requirements |
|----------|------------------|-------------------|
| PostgreSQL | SUPERUSER | All queries logged |
| Redis | +@all except dangerous | All commands logged |
| Argo CD | Admin project access | All operations logged |
| Kubernetes | ClusterRole binding | Audit policy enabled |
| Secrets | Full access to assigned secrets | Access logged |

### Break-Glass Access Procedures

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: tess-break-glass
  namespace: taskmaster
  annotations:
    emergency.taskmaster.io/procedure: "true"
    emergency.taskmaster.io/approvers: "platform-team"
type: Opaque
data:
  postgres-emergency: <base64-encoded-emergency-credentials>
  redis-emergency: <base64-encoded-emergency-credentials>
  argocd-emergency: <base64-encoded-emergency-token>
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-emergency-role
  annotations:
    emergency.taskmaster.io/ttl: "4h"
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]
```

## Monitoring and Observability

### Metrics Collection

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: tess-admin-monitor
  namespace: taskmaster
spec:
  selector:
    matchLabels:
      app: tess-agent
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
---
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: tess-admin-alerts
  namespace: taskmaster
spec:
  groups:
  - name: tess.admin.access
    rules:
    - alert: TessAdminConnectionFailure
      expr: tess_database_connection_failures_total > 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Tess admin database connection failing"
    - alert: TessSecretRotationRequired
      expr: time() - tess_secret_last_rotation_timestamp > 86400
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "Tess admin secrets require rotation"
```

## Integration Points

- **External Secrets Operator**: Dynamic secret management
- **Cert Manager**: TLS certificate lifecycle
- **Argo CD**: Administrative operations and deployments
- **Kubernetes RBAC**: Cluster-level permissions
- **Database Systems**: Admin-level database operations
- **Monitoring Stack**: Audit logging and alerting
