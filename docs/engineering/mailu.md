# Mailu Kubernetes Implementation Plan with Argo CD

## Executive Summary

This implementation plan deploys Mailu mail server on Kubernetes using Argo CD for GitOps management. The architecture leverages your existing Postgres operator for database management, External DNS for automatic DNS record creation, and includes comprehensive monitoring and backup strategies. The deployment supports 10+ users with bot accounts, integrates with Cloudflare for DNS management, and provides a production-ready email infrastructure.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Internet                              │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   Cloudflare DNS   │
                    │   (DNS-only mode)  │
                    └─────────┬─────────┘
                              │
                    ┌─────────▼─────────┐
                    │   LoadBalancer     │
                    │  (Static IP)       │
                    └─────────┬─────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
   ┌────▼────┐         ┌─────▼─────┐        ┌─────▼─────┐
   │  SMTP   │         │   IMAP    │        │  Webmail  │
   │ :25,:587│         │   :993    │        │   :443    │
   └────┬────┘         └─────┬─────┘        └─────┬─────┘
        │                    │                     │
   ┌────▼──────────────────────────────────────────▼────┐
   │              Mailu StatefulSet                      │
   │  ┌──────────┐ ┌──────────┐ ┌──────────┐           │
   │  │  Front   │ │  Admin   │ │ Roundcube│           │
   │  ├──────────┤ ├──────────┤ ├──────────┤           │
   │  │  Postfix │ │  Dovecot │ │  Redis   │           │
   │  ├──────────┤ ├──────────┤ ├──────────┤           │
   │  │  Rspamd  │ │  ClamAV  │ │  Fetchmail│          │
   │  └──────────┘ └──────────┘ └──────────┘           │
   └─────────────────────┬───────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
   ┌────▼────┐    ┌─────▼─────┐   ┌─────▼─────┐
   │Postgres │    │    PVC     │   │   Redis   │
   │Operator │    │   Storage  │   │   Cache   │
   └─────────┘    └────────────┘   └───────────┘
```

## Phase 1: Prerequisites and Infrastructure Setup

### 1.1 Create Git Repository Structure

```bash
# Create repository structure
mkdir -p mailu-deployment/{base,overlays,charts,secrets}
cd mailu-deployment

# Initialize git repository
git init
git remote add origin https://github.com/yourorg/mailu-k8s-config.git
```

**Directory Structure:**
```
mailu-deployment/
├── README.md
├── .gitignore
├── argocd/
│   ├── application.yaml
│   └── appproject.yaml
├── base/
│   ├── namespace.yaml
│   ├── certificates.yaml
│   ├── configmaps.yaml
│   ├── networkpolicies.yaml
│   └── kustomization.yaml
├── overlays/
│   ├── development/
│   │   ├── kustomization.yaml
│   │   └── values.yaml
│   └── production/
│       ├── kustomization.yaml
│       ├── values.yaml
│       └── patches/
├── charts/
│   └── mailu/
│       ├── Chart.yaml
│       └── values.yaml
├── secrets/
│   └── sealed-secrets/
└── monitoring/
    ├── prometheus-rules.yaml
    └── grafana-dashboard.json
```

### 1.2 Namespace and RBAC Setup

**base/namespace.yaml:**
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: mailu
  labels:
    name: mailu
    monitoring: enabled
    backup: enabled
---
apiVersion: v1
kind: ResourceQuota
metadata:
  name: mailu-quota
  namespace: mailu
spec:
  hard:
    requests.cpu: "4"
    requests.memory: 8Gi
    limits.cpu: "8"
    limits.memory: 16Gi
    persistentvolumeclaims: "10"
```

### 1.3 Static IP Allocation

```bash
# For AWS EKS
aws ec2 allocate-address --domain vpc --tag-specifications 'ResourceType=elastic-ip,Tags=[{Key=Name,Value=mailu-mail-server}]'

# For GKE
gcloud compute addresses create mailu-static-ip --global

# For AKS
az network public-ip create \
  --resource-group myResourceGroup \
  --name mailu-static-ip \
  --sku Standard \
  --allocation-method static
```

### 1.4 Configure External DNS

**external-dns-config.yaml:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: external-dns-config
  namespace: external-dns
data:
  config.yaml: |
    sources:
    - service
    - ingress
    provider: cloudflare
    cloudflare:
      proxied: false  # Critical for email services
    domainFilters:
    - example.com
    annotationFilter: "external-dns.alpha.kubernetes.io/hostname"
    txtOwnerId: "mailu-cluster"
    policy: sync
    logLevel: info
```

## Phase 2: Database Configuration with Postgres Operator

### 2.1 Create Postgres Cluster

**postgres/mailu-db.yaml:**
```yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: mailu-postgres
  namespace: mailu
spec:
  instances: 3
  primaryUpdateStrategy: unsupervised
  
  postgresql:
    parameters:
      max_connections: "200"
      shared_buffers: "256MB"
      effective_cache_size: "1GB"
      work_mem: "16MB"
      maintenance_work_mem: "128MB"
      
  bootstrap:
    initdb:
      database: mailu
      owner: mailu
      secret:
        name: mailu-postgres-credentials
      dataChecksums: true
      encoding: UTF8
      
  storage:
    size: 10Gi
    storageClass: fast-ssd
    
  monitoring:
    enabled: true
    customQueries:
      - name: "mailu_stats"
        query: "SELECT COUNT(*) as user_count FROM users"
        metrics:
          - user_count:
              usage: "GAUGE"
              
  backup:
    retentionPolicy: "30d"
    barmanObjectStore:
      destinationPath: "s3://your-backup-bucket/mailu-postgres"
      s3Credentials:
        accessKeyId:
          name: backup-credentials
          key: ACCESS_KEY_ID
        secretAccessKey:
          name: backup-credentials
          key: SECRET_ACCESS_KEY
      wal:
        retention: "7d"
      data:
        jobs: 2
```

### 2.2 Database Credentials

**secrets/postgres-credentials.yaml:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: mailu-postgres-credentials
  namespace: mailu
type: Opaque
stringData:
  username: mailu
  password: $(openssl rand -base64 32)
```

## Phase 3: Mailu Helm Configuration

### 3.1 Add Helm Repository

```bash
helm repo add mailu https://mailu.github.io/helm-charts/
helm repo update
```

### 3.2 Production Values Configuration

**overlays/production/values.yaml:**
```yaml
# Mailu main configuration
domain: mail.example.com
hostnames:
  - mail.example.com
  - smtp.example.com
  - imap.example.com

# Initial admin configuration
initialAccount:
  enabled: true
  domain: example.com
  username: admin
  password: "ChangeMeImmediately123!"  # Change in sealed secret

# Database configuration
database:
  type: postgresql
  host: mailu-postgres-rw.mailu.svc.cluster.local
  port: 5432
  name: mailu
  existingSecret: mailu-postgres-credentials
  existingSecretPasswordKey: password
  existingSecretUsernameKey: username

# Subnet configuration for Kubernetes
subnet: 10.42.0.0/16  # Adjust to your cluster CIDR

# Mail settings
messageSizeLimitMb: 50
authRatelimit:
  ip: "60/hour"
  user: "100/hour"

# TLS configuration
tls:
  enabled: true
  certmanager:
    enabled: true
    issuerName: letsencrypt-prod
    issuerKind: ClusterIssuer

# Component configuration
front:
  enabled: true
  replicaCount: 2
  resources:
    requests:
      memory: "100Mi"
      cpu: "100m"
    limits:
      memory: "200Mi"
      cpu: "200m"
  service:
    type: LoadBalancer
    loadBalancerIP: "YOUR_STATIC_IP"  # From Phase 1.3
    annotations:
      external-dns.alpha.kubernetes.io/hostname: "mail.example.com"
      service.beta.kubernetes.io/aws-load-balancer-type: "nlb"  # For AWS

admin:
  enabled: true
  replicaCount: 1
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"

postfix:
  enabled: true
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"

dovecot:
  enabled: true
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"

rspamd:
  enabled: true
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"

clamav:
  enabled: true
  resources:
    requests:
      memory: "1Gi"
      cpu: "200m"
    limits:
      memory: "2Gi"
      cpu: "1000m"

roundcube:
  enabled: true
  resources:
    requests:
      memory: "128Mi"
      cpu: "100m"
    limits:
      memory: "256Mi"
      cpu: "200m"

redis:
  enabled: true
  persistence:
    enabled: true
    size: 1Gi
    storageClass: fast-ssd

# Storage configuration
persistence:
  enabled: true
  storageClass: fast-ssd
  annotations:
    volume.beta.kubernetes.io/storage-class: fast-ssd
  size:
    data: 100Gi
    mail: 100Gi
    
# Ingress for webmail
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/proxy-body-size: "50m"
  hosts:
    - host: webmail.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: webmail-tls
      hosts:
        - webmail.example.com

# Security settings
securityContext:
  runAsNonRoot: false  # Mailu requires root for some components
  fsGroup: 1000

# Network policies
networkPolicy:
  enabled: true
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: mailu
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
      ports:
        - protocol: TCP
          port: 80
        - protocol: TCP
          port: 443
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: mailu
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: TCP
          port: 53
        - protocol: UDP
          port: 53
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
            except:
              - 169.254.169.254/32  # Block metadata service
```

### 3.3 Kustomization Configuration

**overlays/production/kustomization.yaml:**
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: mailu

resources:
  - ../../base
  - postgres-cluster.yaml

helmCharts:
  - name: mailu
    repo: https://mailu.github.io/helm-charts/
    version: 2.2.2
    releaseName: mailu
    namespace: mailu
    valuesFile: values.yaml

patches:
  - target:
      kind: StatefulSet
      name: mailu
    patch: |-
      - op: add
        path: /spec/template/spec/affinity
        value:
          podAntiAffinity:
            preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchExpressions:
                  - key: app
                    operator: In
                    values:
                    - mailu
                topologyKey: kubernetes.io/hostname

configMapGenerator:
  - name: mailu-env
    literals:
      - KUBERNETES_CLUSTER=true
      - LOG_LEVEL=INFO
      - TZ=UTC

secretGenerator:
  - name: mailu-secrets
    literals:
      - SECRET_KEY=$(openssl rand -hex 32)
      - INITIAL_ADMIN_PASSWORD=$(openssl rand -base64 24)
```

## Phase 4: Argo CD Application Configuration

### 4.1 Create Argo CD Project

**argocd/appproject.yaml:**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: mailu
  namespace: argocd
spec:
  description: Mailu mail server project
  
  sourceRepos:
    - 'https://github.com/yourorg/mailu-k8s-config.git'
    - 'https://mailu.github.io/helm-charts/'
  
  destinations:
    - namespace: mailu
      server: https://kubernetes.default.svc
    - namespace: cert-manager
      server: https://kubernetes.default.svc
      
  clusterResourceWhitelist:
    - group: ''
      kind: Namespace
    - group: cert-manager.io
      kind: ClusterIssuer
      
  namespaceResourceWhitelist:
    - group: '*'
      kind: '*'
      
  roles:
    - name: admin
      policies:
        - p, proj:mailu:admin, applications, *, mailu/*, allow
        - p, proj:mailu:admin, repositories, *, *, allow
      groups:
        - platform-team
        
    - name: developer
      policies:
        - p, proj:mailu:developer, applications, get, mailu/*, allow
        - p, proj:mailu:developer, logs, get, mailu/*, allow
      groups:
        - dev-team
```

### 4.2 Create Argo CD Application

**argocd/application.yaml:**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: mailu
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: mailu
  
  source:
    repoURL: https://github.com/yourorg/mailu-k8s-config.git
    targetRevision: main
    path: overlays/production
    
  destination:
    server: https://kubernetes.default.svc
    namespace: mailu
    
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    syncOptions:
      - CreateNamespace=true
      - ServerSideApply=true
      - ApplyOutOfSyncOnly=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
        
  revisionHistoryLimit: 10
  
  ignoreDifferences:
    - group: apps
      kind: StatefulSet
      jsonPointers:
        - /spec/volumeClaimTemplates
    - group: ""
      kind: Service
      jsonPointers:
        - /spec/clusterIP
        
  info:
    - name: 'Documentation'
      value: 'https://mailu.io/docs'
    - name: 'Admin URL'
      value: 'https://admin.mail.example.com'
```

### 4.3 Deploy Application

```bash
# Apply Argo CD project
kubectl apply -f argocd/appproject.yaml

# Apply Argo CD application
kubectl apply -f argocd/application.yaml

# Trigger initial sync
argocd app sync mailu

# Watch deployment
argocd app wait mailu --health
```

## Phase 5: DNS Configuration

### 5.1 Required DNS Records

Create these DNS records in Cloudflare (with proxy disabled):

```yaml
# MX Records
Type: MX
Name: @
Content: mail.example.com
Priority: 10
Proxy: DNS only (gray cloud)

# A Records
Type: A
Name: mail
Content: YOUR_STATIC_IP
Proxy: DNS only (gray cloud)

# SPF Record
Type: TXT
Name: @
Content: "v=spf1 mx a:mail.example.com ~all"
Proxy: DNS only

# DMARC Record
Type: TXT
Name: _dmarc
Content: "v=DMARC1; p=quarantine; rua=mailto:dmarc@example.com; ruf=mailto:dmarc@example.com; sp=quarantine; adkim=s; aspf=s"
Proxy: DNS only

# Autodiscover Records
Type: CNAME
Name: autodiscover
Content: mail.example.com
Proxy: DNS only

Type: CNAME
Name: autoconfig
Content: mail.example.com
Proxy: DNS only

# SRV Records for mail clients
Type: SRV
Name: _imaps._tcp
Content: 0 1 993 mail.example.com
Proxy: DNS only

Type: SRV
Name: _submission._tcp
Content: 0 1 587 mail.example.com
Proxy: DNS only
```

### 5.2 DKIM Configuration

After Mailu is running:

```bash
# Access Mailu admin pod
kubectl exec -it -n mailu mailu-admin-0 -- /bin/bash

# Generate DKIM keys
mailu dkim generate example.com

# Get DKIM public key
mailu dkim show example.com

# Add to DNS as TXT record
# Name: dkim._domainkey
# Content: [output from above command]
```

### 5.3 PTR Record Configuration

Configure reverse DNS with your cloud provider:

```bash
# AWS
aws ec2 modify-address-attribute \
  --allocation-id eipalloc-xxxxx \
  --domain-name mail.example.com

# GCP
gcloud compute addresses update mailu-static-ip \
  --reverse-dns mail.example.com

# Azure
az network public-ip update \
  --resource-group myResourceGroup \
  --name mailu-static-ip \
  --reverse-fqdn mail.example.com
```

## Phase 6: User and Bot Account Management

### 6.1 Create User Management Scripts

**scripts/user-management.sh:**
```bash
#!/bin/bash

NAMESPACE="mailu"
ADMIN_POD=$(kubectl get pod -n $NAMESPACE -l app=mailu-admin -o jsonpath='{.items[0].metadata.name}')

create_user() {
    local email=$1
    local password=$2
    local quota=$3
    
    kubectl exec -n $NAMESPACE $ADMIN_POD -- \
        flask mailu user create $email "$password" --quota $quota
}

create_bot_account() {
    local email=$1
    local app_name=$2
    
    # Generate strong password
    password=$(openssl rand -base64 32)
    
    # Create account with limited privileges
    kubectl exec -n $NAMESPACE $ADMIN_POD -- \
        flask mailu user create $email "$password" --quota 100M
    
    # Store credentials in secret
    kubectl create secret generic bot-$app_name \
        -n $NAMESPACE \
        --from-literal=email=$email \
        --from-literal=password=$password
        
    echo "Bot account created: $email"
    echo "Credentials stored in secret: bot-$app_name"
}

# Usage examples
create_user "john@example.com" "TempPass123!" "1G"
create_bot_account "notifications@example.com" "notification-service"
```

### 6.2 Bulk User Creation

**scripts/bulk-users.yaml:**
```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: create-initial-users
  namespace: mailu
spec:
  template:
    spec:
      restartPolicy: OnFailure
      containers:
      - name: user-creator
        image: mailu/admin:2024.06
        command:
        - /bin/bash
        - -c
        - |
          # Create regular users
          flask mailu user create alice@example.com "$(openssl rand -base64 12)" --quota 5G
          flask mailu user create bob@example.com "$(openssl rand -base64 12)" --quota 5G
          
          # Create bot accounts
          flask mailu user create alerts@example.com "$(openssl rand -base64 32)" --quota 100M
          flask mailu user create noreply@example.com "$(openssl rand -base64 32)" --quota 100M
          
          # Create aliases
          flask mailu alias create postmaster@example.com admin@example.com
          flask mailu alias create abuse@example.com admin@example.com
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: mailu-database
              key: url
```

## Phase 7: Monitoring and Alerting

### 7.1 Prometheus ServiceMonitor

**monitoring/servicemonitor.yaml:**
```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: mailu-metrics
  namespace: mailu
spec:
  selector:
    matchLabels:
      app: mailu
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

### 7.2 Prometheus Rules

**monitoring/prometheus-rules.yaml:**
```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: mailu-alerts
  namespace: mailu
spec:
  groups:
  - name: mailu.rules
    interval: 30s
    rules:
    - alert: MailQueueHigh
      expr: postfix_queue_size > 100
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High mail queue size"
        description: "Mail queue has {{ $value }} messages"
        
    - alert: MailuPodDown
      expr: up{job="mailu"} == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Mailu component down"
        description: "{{ $labels.pod }} is down"
        
    - alert: DiskSpaceRunningOut
      expr: |
        (
          node_filesystem_avail_bytes{mountpoint="/data"} / 
          node_filesystem_size_bytes{mountpoint="/data"}
        ) * 100 < 10
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "Disk space running out"
        description: "Less than 10% disk space remaining"
        
    - alert: CertificateExpiringSoon
      expr: cert_expiry_timestamp - time() < 7 * 24 * 3600
      labels:
        severity: warning
      annotations:
        summary: "Certificate expiring soon"
        description: "Certificate expires in less than 7 days"
```

### 7.3 Grafana Dashboard

**monitoring/grafana-dashboard.json:**
```json
{
  "dashboard": {
    "title": "Mailu Mail Server",
    "panels": [
      {
        "title": "Mail Queue Size",
        "targets": [
          {
            "expr": "postfix_queue_size"
          }
        ]
      },
      {
        "title": "Messages Sent/Received",
        "targets": [
          {
            "expr": "rate(postfix_messages_sent_total[5m])"
          },
          {
            "expr": "rate(postfix_messages_received_total[5m])"
          }
        ]
      },
      {
        "title": "IMAP Connections",
        "targets": [
          {
            "expr": "dovecot_connections"
          }
        ]
      },
      {
        "title": "Spam Detection Rate",
        "targets": [
          {
            "expr": "rate(rspamd_spam_detected_total[5m])"
          }
        ]
      }
    ]
  }
}
```

## Phase 8: Backup and Disaster Recovery

### 8.1 Velero Backup Configuration

**backup/velero-schedule.yaml:**
```yaml
apiVersion: velero.io/v1
kind: Schedule
metadata:
  name: mailu-daily-backup
  namespace: velero
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  template:
    includedNamespaces:
    - mailu
    includedResources:
    - "*"
    labelSelector:
      matchLabels:
        app: mailu
    storageLocation: default
    ttl: 720h  # 30 days retention
    hooks:
      resources:
      - name: database-backup
        includedNamespaces:
        - mailu
        labelSelector:
          matchLabels:
            app: mailu-postgres
        pre:
        - exec:
            container: postgres
            command:
            - /bin/bash
            - -c
            - |
              pg_dump -U mailu mailu > /backup/mailu-$(date +%Y%m%d).sql
            onError: Fail
            timeout: 10m
```

### 8.2 Restore Procedure

**backup/restore-procedure.md:**
```markdown
# Mailu Restore Procedure

## Full Restore from Backup

1. List available backups:
   ```bash
   velero backup get
   ```

2. Restore specific backup:
   ```bash
   velero restore create --from-backup mailu-daily-backup-20240101
   ```

3. Verify pods are running:
   ```bash
   kubectl get pods -n mailu
   ```

4. Restore database if needed:
   ```bash
   kubectl exec -n mailu mailu-postgres-1 -- \
     psql -U mailu mailu < /backup/mailu-20240101.sql
   ```

## Partial Restore (Users Only)

1. Export users from backup:
   ```bash
   kubectl exec -n mailu mailu-admin-0 -- \
     flask mailu user export > users.yaml
   ```

2. Import users:
   ```bash
   kubectl exec -n mailu mailu-admin-0 -- \
     flask mailu user import < users.yaml
   ```
```

## Phase 9: Testing and Validation

### 9.1 Automated Testing Suite

**tests/mail-test.yaml:**
```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: mailu-test-suite
  namespace: mailu
spec:
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: test-runner
        image: alpine:latest
        command:
        - /bin/sh
        - -c
        - |
          # Install dependencies
          apk add --no-cache curl openssl swaks
          
          # Test SMTP
          echo "Testing SMTP..."
          swaks --to test@example.com \
                --from sender@example.com \
                --server mail.example.com:587 \
                --tls \
                --auth LOGIN \
                --auth-user test@example.com \
                --auth-password $TEST_PASSWORD
          
          # Test IMAP
          echo "Testing IMAP..."
          openssl s_client -connect mail.example.com:993 \
                  -quiet -crlf < /dev/null
          
          # Test Webmail
          echo "Testing Webmail..."
          curl -Is https://webmail.example.com | head -n 1
          
          # Test DNS
          echo "Testing DNS records..."
          nslookup -type=MX example.com
          nslookup -type=TXT _dmarc.example.com
          
          echo "All tests completed!"
        env:
        - name: TEST_PASSWORD
          valueFrom:
            secretKeyRef:
              name: test-credentials
              key: password
```

### 9.2 Mail Deliverability Test

```bash
# Test email deliverability
curl -X POST https://api.mail-tester.com/generate

# Send test email to provided address
swaks --to test-xxxxx@mail-tester.com \
      --from admin@example.com \
      --server mail.example.com:587 \
      --tls

# Check score at https://www.mail-tester.com/test-xxxxx
```

## Phase 10: Production Deployment Checklist

### Pre-deployment
- [ ] Static IP allocated and configured
- [ ] PTR record configured with ISP/Cloud provider
- [ ] Firewall rules allow ports 25, 587, 993, 443
- [ ] Postgres cluster healthy with 3 replicas
- [ ] Backup destination (S3/GCS/Azure) configured
- [ ] Monitoring stack (Prometheus/Grafana) operational
- [ ] Sealed Secrets controller installed

### DNS Configuration
- [ ] MX records pointing to mail.example.com
- [ ] A record for mail.example.com → static IP
- [ ] SPF record configured
- [ ] DKIM keys generated and published
- [ ] DMARC policy set to p=none initially
- [ ] Autodiscover/Autoconfig CNAME records
- [ ] PTR record verified with `dig -x YOUR_IP`

### Deployment
- [ ] Argo CD application created and synced
- [ ] All pods running and healthy
- [ ] PVCs bound and sufficient space
- [ ] TLS certificates issued by cert-manager
- [ ] Initial admin account accessible

### Post-deployment
- [ ] Send test email to Gmail/Outlook
- [ ] Verify no spam folder delivery
- [ ] DKIM signature validation passing
- [ ] SPF alignment passing
- [ ] Create user accounts
- [ ] Configure mail clients (Thunderbird/Outlook)
- [ ] Set up email aliases
- [ ] Enable 2FA for admin accounts
- [ ] Schedule backup verification
- [ ] Update DMARC policy to p=quarantine after 1 week
- [ ] Document credentials in password manager

### Monitoring
- [ ] Prometheus scraping metrics
- [ ] Grafana dashboard displaying data
- [ ] Alerts configured in AlertManager
- [ ] Log aggregation working
- [ ] Backup jobs completing successfully

## Troubleshooting Guide

### Common Issues and Solutions

**Issue: Emails going to spam**
```bash
# Check SPF
dig TXT example.com | grep spf

# Check DKIM
kubectl exec -n mailu mailu-admin-0 -- mailu dkim show example.com

# Check mail server reputation
curl https://www.senderscore.org/lookup.php?ip=YOUR_IP
```

**Issue: Cannot receive emails**
```bash
# Check MX records
dig MX example.com

# Test port 25 connectivity
telnet mail.example.com 25

# Check Postfix logs
kubectl logs -n mailu mailu-postfix-0 --tail=100
```

**Issue: High memory usage**
```bash
# Disable ClamAV if not needed
kubectl patch deployment mailu-clamav -n mailu \
  -p '{"spec":{"replicas":0}}'

# Adjust memory limits in values.yaml
```

**Issue: Slow webmail**
```bash
# Scale Redis
kubectl scale statefulset mailu-redis -n mailu --replicas=2

# Increase PHP memory limit
kubectl set env deployment/mailu-roundcube \
  PHP_MEMORY_LIMIT=256M -n mailu
```

## Maintenance Procedures

### Monthly Tasks
- Review mail logs for delivery issues
- Check disk usage trends
- Update spam filter rules
- Review user quotas
- Test backup restoration
- Update DMARC reports analysis

### Quarterly Tasks
- Update Mailu version
- Review security patches
- Audit user accounts
- Performance tuning based on metrics
- Disaster recovery drill
- SSL certificate renewal verification

### Annual Tasks
- Full security audit
- Capacity planning review
- Cost optimization analysis
- Documentation update
- Team training on procedures
- Compliance review (if applicable)

## Cost Optimization Tips

1. **Resource Rightsizing**: Start with minimal resources and scale based on actual usage
2. **Storage Optimization**: Use lifecycle policies to archive old emails to cheaper storage
3. **ClamAV Optional**: Disable if using cloud email security gateway
4. **Spot Instances**: Run non-critical components on spot/preemptible instances
5. **Regional Selection**: Choose regions with lower egress costs
6. **Reserved Instances**: Commit to 1-3 year terms for compute savings

## Security Hardening

1. **Network Policies**: Implement strict ingress/egress rules
2. **Pod Security Policies**: Enforce security standards
3. **Secrets Management**: Use Sealed Secrets or external secret managers
4. **Rate Limiting**: Configure fail2ban equivalent
5. **TLS Configuration**: Enforce TLS 1.2+ with strong ciphers
6. **Regular Updates**: Automate security patch deployment
7. **Audit Logging**: Enable and centralize audit logs
8. **RBAC**: Implement least-privilege access control