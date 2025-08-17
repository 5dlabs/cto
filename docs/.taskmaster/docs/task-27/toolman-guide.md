# Toolman Guide: Comprehensive Admin Access for Tess Agent

## Overview

This guide provides detailed instructions for configuring, managing, and troubleshooting administrative access for the Tess agent. The system involves multiple components including external secret management, database administration, Argo CD integration, and comprehensive monitoring.

## Tool Categories

### 1. External Secret Management Tools

#### External Secrets Operator (`external_secrets_operator`)

**Purpose**: Manage dynamic secret injection from external secret stores

**Installation and Setup**:
```bash
# Install External Secrets Operator
helm repo add external-secrets https://charts.external-secrets.io
helm install external-secrets external-secrets/external-secrets \
  --namespace external-secrets-system \
  --create-namespace

# Verify installation
kubectl get pods -n external-secrets-system
kubectl get crd | grep external-secrets
```

**Key Operations**:
```bash
# Create AWS Secrets Manager SecretStore
kubectl apply -f - <<EOF
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
EOF

# Test SecretStore connectivity
kubectl get secretstore tess-admin-secrets -o yaml

# Create ExternalSecret for PostgreSQL
kubectl apply -f - <<EOF
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
  - secretKey: username
    remoteRef:
      key: /infrastructure/tess-admin/postgres  
      property: username
  - secretKey: password
    remoteRef:
      key: /infrastructure/tess-admin/postgres
      property: password
  refreshInterval: 24h
EOF
```

**Monitoring and Troubleshooting**:
```bash
# Check ExternalSecret status
kubectl get externalsecret tess-postgres-admin -o yaml

# View secret refresh logs
kubectl logs -f deployment/external-secrets -n external-secrets-system

# Force secret refresh
kubectl annotate externalsecret tess-postgres-admin force-sync=$(date +%s)

# Debug secret retrieval
kubectl describe externalsecret tess-postgres-admin
```

#### AWS Secrets Manager (`aws_secrets_manager`)

**Purpose**: Secure storage and retrieval of administrative credentials

**Setup and Configuration**:
```bash
# Install AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip && sudo ./aws/install

# Configure credentials
aws configure set aws_access_key_id YOUR_ACCESS_KEY
aws configure set aws_secret_access_key YOUR_SECRET_KEY
aws configure set region us-west-2

# Verify connectivity
aws sts get-caller-identity
```

**Secret Management Operations**:
```bash
# Create PostgreSQL admin secret
aws secretsmanager create-secret \
  --name "/infrastructure/tess-admin/postgres" \
  --description "PostgreSQL admin credentials for Tess agent" \
  --secret-string '{
    "host": "postgres-primary.taskmaster.local",
    "port": "5432",
    "username": "tess_admin",
    "password": "secure-random-password",
    "database": "taskmaster",
    "connection_url": "postgresql://tess_admin:secure-random-password@postgres-primary.taskmaster.local:5432/taskmaster"
  }'

# Create Redis admin secret
aws secretsmanager create-secret \
  --name "/infrastructure/tess-admin/redis" \
  --description "Redis admin credentials for Tess agent" \
  --secret-string '{
    "host": "redis-cluster.taskmaster.local",
    "port": "6379",
    "password": "redis-admin-password",
    "cluster_endpoints": "redis-node-1:6379,redis-node-2:6379,redis-node-3:6379",
    "connection_url": "redis://:redis-admin-password@redis-cluster.taskmaster.local:6379"
  }'

# Create Argo CD admin secret
aws secretsmanager create-secret \
  --name "/infrastructure/tess-admin/argocd" \
  --description "Argo CD admin credentials for Tess agent" \
  --secret-string '{
    "admin_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "server_url": "https://argocd.taskmaster.local",
    "admin_username": "admin"
  }'

# Update existing secret
aws secretsmanager update-secret \
  --secret-id "/infrastructure/tess-admin/postgres" \
  --secret-string '{"password": "new-secure-password"}'

# Retrieve secret value
aws secretsmanager get-secret-value \
  --secret-id "/infrastructure/tess-admin/postgres" \
  --query SecretString --output text | jq .

# List all Tess admin secrets
aws secretsmanager list-secrets \
  --filters Key=name,Values="/infrastructure/tess-admin"
```

**Secret Rotation**:
```bash
# Enable automatic rotation
aws secretsmanager rotate-secret \
  --secret-id "/infrastructure/tess-admin/postgres" \
  --rotation-rules AutomaticallyAfterDays=30

# Manual rotation trigger
aws secretsmanager rotate-secret \
  --secret-id "/infrastructure/tess-admin/postgres" \
  --force-rotate-immediately
```

### 2. Database Administration Tools

#### PostgreSQL Client (`postgres_client`)

**Purpose**: Administrative operations on PostgreSQL databases

**Installation**:
```bash
# Install PostgreSQL client
apt-get update && apt-get install -y postgresql-client

# Verify installation
psql --version
```

**Administrative Operations**:
```bash
# Connect as admin user
export PGPASSWORD="admin-password"
psql -h postgres-primary.taskmaster.local -p 5432 -U tess_admin -d taskmaster

# Create admin user
psql -h postgres-primary.taskmaster.local -c "
CREATE USER tess_admin WITH SUPERUSER CREATEDB CREATEROLE LOGIN PASSWORD 'secure-password';
GRANT ALL PRIVILEGES ON DATABASE taskmaster TO tess_admin;
"

# Test admin privileges
psql -h postgres-primary.taskmaster.local -U tess_admin -d taskmaster -c "
SELECT current_user, session_user, usesuper FROM pg_user WHERE usename = current_user;
"

# Create test database
psql -h postgres-primary.taskmaster.local -U tess_admin -c "
CREATE DATABASE tess_test_db;
DROP DATABASE tess_test_db;
"

# Administrative queries
psql -h postgres-primary.taskmaster.local -U tess_admin -d taskmaster <<EOF
-- Check database size
SELECT pg_size_pretty(pg_database_size('taskmaster'));

-- List all databases
\l

-- Show active connections
SELECT datname, usename, client_addr, state 
FROM pg_stat_activity 
WHERE state = 'active';

-- Vacuum and analyze
VACUUM ANALYZE;
EOF
```

**Connection Testing Script**:
```bash
#!/bin/bash
test_postgres_connection() {
  local host="$1"
  local port="$2" 
  local user="$3"
  local password="$4"
  local database="$5"
  
  export PGPASSWORD="$password"
  
  echo "Testing PostgreSQL connection..."
  if psql -h "$host" -p "$port" -U "$user" -d "$database" -c "SELECT version();" > /dev/null 2>&1; then
    echo "✓ PostgreSQL connection successful"
    return 0
  else
    echo "✗ PostgreSQL connection failed"
    return 1
  fi
}

# Usage
test_postgres_connection "postgres-primary.taskmaster.local" "5432" "tess_admin" "password" "taskmaster"
```

#### Redis Client (`redis_client`)

**Purpose**: Administrative operations on Redis instances and clusters

**Installation**:
```bash
# Install Redis client
apt-get update && apt-get install -y redis-tools

# Verify installation
redis-cli --version
```

**Administrative Operations**:
```bash
# Connect to Redis with authentication
redis-cli -h redis-cluster.taskmaster.local -p 6379 -a "admin-password"

# Test connection
redis-cli -h redis-cluster.taskmaster.local -p 6379 -a "admin-password" ping

# Administrative commands
redis-cli -h redis-cluster.taskmaster.local -p 6379 -a "admin-password" <<EOF
INFO server
INFO memory
INFO replication
CONFIG GET maxmemory
CONFIG GET save
DBSIZE
LASTSAVE
EOF

# Cluster operations
redis-cli --cluster info redis-node-1:6379
redis-cli --cluster check redis-node-1:6379

# Create Redis ACL for Tess
redis-cli -h redis-cluster.taskmaster.local -p 6379 -a "admin-password" \
  ACL SETUSER tess_admin on '>secure-password' ~* &* +@all -flushall -flushdb

# Test ACL permissions
redis-cli -h redis-cluster.taskmaster.local -p 6379 -a "secure-password" \
  --user tess_admin ACL WHOAMI
```

**Redis Health Check Script**:
```bash
#!/bin/bash
test_redis_connection() {
  local host="$1"
  local port="$2"
  local password="$3"
  
  echo "Testing Redis connection..."
  if redis-cli -h "$host" -p "$port" -a "$password" ping | grep -q PONG; then
    echo "✓ Redis connection successful"
    
    # Test admin operations
    if redis-cli -h "$host" -p "$port" -a "$password" INFO server > /dev/null 2>&1; then
      echo "✓ Redis admin operations available"
      return 0
    else
      echo "✗ Redis admin operations failed"
      return 1
    fi
  else
    echo "✗ Redis connection failed"
    return 1
  fi
}

# Usage
test_redis_connection "redis-cluster.taskmaster.local" "6379" "admin-password"
```

### 3. Argo CD Administration Tools

#### Argo CD Client (`argocd_client`)

**Purpose**: Administrative operations on Argo CD applications and projects

**Installation**:
```bash
# Download Argo CD CLI
VERSION=$(curl --silent "https://api.github.com/repos/argoproj/argo-cd/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
curl -sSL -o argocd-linux-amd64 https://github.com/argoproj/argo-cd/releases/download/$VERSION/argocd-linux-amd64
sudo install -m 555 argocd-linux-amd64 /usr/local/bin/argocd

# Verify installation
argocd version --client
```

**Authentication and Setup**:
```bash
# Login with admin credentials
argocd login argocd.taskmaster.local \
  --username admin \
  --password "$ARGOCD_TOKEN" \
  --insecure

# Create admin project for Tess
argocd proj create tess-admin-project \
  --description "Administrative project for Tess agent operations" \
  --src '*' \
  --dest '*,*' \
  --allow-cluster-resource '*/*' \
  --allow-namespaced-resource '*/*'

# Add role to project
argocd proj role create tess-admin-project tess-admin \
  --description "Full administrative access for Tess agent"

# Grant permissions
argocd proj role add-policy tess-admin-project tess-admin \
  --action '*' \
  --permission allow \
  --resource '*'
```

**Application Management**:
```bash
# Create test application
argocd app create tess-test-app \
  --project tess-admin-project \
  --repo https://github.com/5dlabs/cto \
  --path manifests/test-app \
  --dest-namespace default \
  --dest-server https://kubernetes.default.svc

# List applications
argocd app list --project tess-admin-project

# Sync application
argocd app sync tess-test-app

# Get application details
argocd app get tess-test-app

# Delete test application
argocd app delete tess-test-app --yes

# Manage repositories
argocd repo add https://github.com/5dlabs/cto \
  --username git \
  --password "$GITHUB_TOKEN"

# List clusters
argocd cluster list
```

**Health Check Script**:
```bash
#!/bin/bash
test_argocd_connection() {
  local server="$1"
  local username="$2"
  local password="$3"
  
  echo "Testing Argo CD connection..."
  if argocd login "$server" --username "$username" --password "$password" --insecure > /dev/null 2>&1; then
    echo "✓ Argo CD authentication successful"
    
    # Test administrative operations
    if argocd cluster list > /dev/null 2>&1; then
      echo "✓ Argo CD admin operations available"
      return 0
    else
      echo "✗ Argo CD admin operations failed"
      return 1
    fi
  else
    echo "✗ Argo CD authentication failed"
    return 1
  fi
}

# Usage
test_argocd_connection "argocd.taskmaster.local" "admin" "$ARGOCD_TOKEN"
```

### 4. Kubernetes Management Tools

#### Kubectl (`kubectl`)

**Purpose**: Kubernetes cluster resource management and administration

**Authentication Setup**:
```bash
# Configure kubectl context
kubectl config set-cluster taskmaster \
  --server=https://kubernetes.taskmaster.local:6443 \
  --certificate-authority=ca.crt

kubectl config set-credentials tess-admin \
  --token="$TESS_ADMIN_TOKEN"

kubectl config set-context tess-admin-context \
  --cluster=taskmaster \
  --user=tess-admin \
  --namespace=taskmaster

kubectl config use-context tess-admin-context
```

**Resource Management**:
```bash
# Create service account
kubectl create serviceaccount tess-admin-service-account -n taskmaster

# Create cluster role
kubectl apply -f - <<EOF
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
  resources: ["workflows", "applications"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
EOF

# Create cluster role binding
kubectl create clusterrolebinding tess-admin-binding \
  --clusterrole=tess-admin-role \
  --serviceaccount=taskmaster:tess-admin-service-account

# Verify permissions
kubectl auth can-i create secrets --as=system:serviceaccount:taskmaster:tess-admin-service-account
kubectl auth can-i delete deployments --as=system:serviceaccount:taskmaster:tess-admin-service-account
```

**Secret Management**:
```bash
# Create admin secrets manually (for testing)
kubectl create secret generic tess-postgres-admin \
  --from-literal=host="postgres-primary.taskmaster.local" \
  --from-literal=port="5432" \
  --from-literal=username="tess_admin" \
  --from-literal=password="secure-password" \
  --from-literal=database="taskmaster" \
  --namespace=taskmaster

# View secret (base64 encoded)
kubectl get secret tess-postgres-admin -o yaml

# Decode secret values
kubectl get secret tess-postgres-admin -o jsonpath='{.data.password}' | base64 -d

# Update secret
kubectl patch secret tess-postgres-admin \
  -p='{"data":{"password":"'$(echo -n "new-password" | base64)'"}}'
```

### 5. TLS Certificate Management Tools

#### OpenSSL (`openssl`)

**Purpose**: TLS certificate operations and validation

**Certificate Validation**:
```bash
# Check certificate details
openssl x509 -in tess-client.crt -text -noout

# Verify certificate chain
openssl verify -CAfile ca.crt tess-client.crt

# Check certificate expiration
openssl x509 -in tess-client.crt -noout -dates

# Test TLS connection
openssl s_client -connect postgres-primary.taskmaster.local:5432 \
  -cert tess-client.crt -key tess-client.key -CAfile ca.crt

# Generate CSR for certificate renewal
openssl req -new -key tess-client.key -out tess-client.csr \
  -subj "/CN=tess-admin-client/O=taskmaster"
```

**Certificate Management with cert-manager**:
```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.11.0/cert-manager.yaml

# Create certificate for Tess
kubectl apply -f - <<EOF
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
  usages:
  - client auth
  - digital signature
  - key encipherment
EOF

# Check certificate status
kubectl describe certificate tess-client-cert -n taskmaster

# View generated secret
kubectl get secret tess-tls-certificates -o yaml
```

### 6. Monitoring and Troubleshooting Tools

#### JSON Processor (`jq`)

**Purpose**: Parse API responses and configuration files

**Secret Processing**:
```bash
# Parse AWS Secrets Manager response
aws secretsmanager get-secret-value \
  --secret-id "/infrastructure/tess-admin/postgres" \
  --query SecretString --output text | \
  jq -r '.host'

# Process Kubernetes secret
kubectl get secret tess-postgres-admin -o json | \
  jq -r '.data.password' | base64 -d

# Extract connection details
kubectl get secret tess-postgres-admin -o json | \
  jq -r '.data | to_entries[] | "\(.key): \(.value | @base64d)"'
```

#### HTTP Client (`curl`)

**Purpose**: API operations and health checks

**Health Check Operations**:
```bash
# Check Argo CD API health
curl -k -H "Authorization: Bearer $ARGOCD_TOKEN" \
  https://argocd.taskmaster.local/api/v1/version

# Monitor External Secrets Operator
curl -s http://external-secrets-metrics:8080/metrics | \
  grep external_secrets

# Test database health endpoint
curl -f http://postgres-exporter:9187/metrics

# Check Redis metrics
curl -s http://redis-exporter:9121/metrics
```

## Best Practices

### 1. Security Best Practices

```bash
# Never log sensitive credentials
echo "Connecting to database..." # Good
echo "Password: $DB_PASSWORD"   # Never do this

# Use environment variables for secrets
export PGPASSWORD="$POSTGRES_PASSWORD"
psql -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DATABASE"

# Validate certificates
validate_cert() {
  local cert_file="$1"
  if openssl x509 -in "$cert_file" -noout -checkend 86400; then
    echo "Certificate valid for next 24 hours"
  else
    echo "Certificate expires soon or is invalid"
    return 1
  fi
}

# Secure temporary files
temp_file=$(mktemp)
trap "rm -f $temp_file" EXIT
echo "sensitive data" > "$temp_file"
```

### 2. Error Handling

```bash
# Robust error handling
set -euo pipefail

# Function with retry logic
retry_command() {
  local max_retries=3
  local delay=2
  local command="$1"
  
  for ((i=1; i<=max_retries; i++)); do
    if eval "$command"; then
      return 0
    else
      echo "Attempt $i failed, retrying in ${delay}s..."
      sleep "$delay"
      delay=$((delay * 2))
    fi
  done
  
  echo "Command failed after $max_retries attempts"
  return 1
}

# Usage
retry_command "psql -h postgres-primary.taskmaster.local -c 'SELECT 1'"
```

### 3. Monitoring Integration

```bash
# Health check with metrics
perform_health_check() {
  local service="$1"
  local start_time=$(date +%s)
  
  case "$service" in
    "postgres")
      test_postgres_connection && status="success" || status="failure"
      ;;
    "redis")
      test_redis_connection && status="success" || status="failure"
      ;;
    "argocd")
      test_argocd_connection && status="success" || status="failure"
      ;;
  esac
  
  local duration=$(($(date +%s) - start_time))
  
  # Push metrics to Prometheus
  cat <<EOF | curl -X POST --data-binary @- http://pushgateway:9091/metrics/job/tess-admin-health
tess_admin_health_check_duration_seconds{service="$service",status="$status"} $duration
tess_admin_health_check_total{service="$service",status="$status"} 1
EOF
}
```

### 4. Operational Procedures

```bash
# Complete admin access validation
validate_tess_admin_access() {
  echo "=== Validating Tess Admin Access ==="
  
  # Check Kubernetes access
  echo "1. Testing Kubernetes access..."
  if kubectl auth can-i create secrets --as=system:serviceaccount:taskmaster:tess-admin-service-account; then
    echo "✓ Kubernetes access validated"
  else
    echo "✗ Kubernetes access failed"
    return 1
  fi
  
  # Check database access
  echo "2. Testing database access..."
  test_postgres_connection || return 1
  test_redis_connection || return 1
  
  # Check Argo CD access
  echo "3. Testing Argo CD access..."
  test_argocd_connection || return 1
  
  # Check secret rotation
  echo "4. Testing secret rotation..."
  kubectl get externalsecret -n taskmaster | grep -q "True.*24h" || {
    echo "✗ Secret rotation not configured properly"
    return 1
  }
  
  echo "✓ All admin access validated successfully"
  return 0
}

# Emergency access procedure
enable_break_glass_access() {
  echo "=== EMERGENCY: Enabling Break-Glass Access ==="
  echo "This action is logged and monitored"
  
  # Create emergency role binding
  kubectl apply -f - <<EOF
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: tess-emergency-access
  annotations:
    emergency.taskmaster.io/enabled-at: "$(date -Iseconds)"
    emergency.taskmaster.io/expires-at: "$(date -d '+4 hours' -Iseconds)"
subjects:
- kind: ServiceAccount
  name: tess-admin-service-account
  namespace: taskmaster
roleRef:
  kind: ClusterRole
  name: cluster-admin
  apiGroup: rbac.authorization.k8s.io
EOF
  
  # Set expiration timer
  (sleep 14400 && kubectl delete clusterrolebinding tess-emergency-access) &
  
  echo "Break-glass access enabled for 4 hours"
}
```

This comprehensive guide provides all necessary tools and procedures for managing Tess agent administrative access. Follow these patterns and best practices to ensure secure, reliable, and well-monitored administrative operations.