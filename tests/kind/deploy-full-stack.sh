#!/bin/bash
# Deploy full CTO stack to Kind cluster
# Includes: Observability (Prometheus, Loki, Grafana), Argo Events, tweakcn
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHARTS_DIR="${SCRIPT_DIR}/../../infra/charts"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     CTO Full Stack Deployment for Kind                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Add Helm repos
echo ""
echo "📦 Adding Helm repositories..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts 2>/dev/null || true
helm repo add grafana https://grafana.github.io/helm-charts 2>/dev/null || true
helm repo add argo https://argoproj.github.io/argo-helm 2>/dev/null || true
helm repo update

# Create namespaces
echo ""
echo "📁 Creating namespaces..."
kubectl create namespace observability --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace automation --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace cto --dry-run=client -o yaml | kubectl apply -f -

# ============================================================================
# 1. Deploy Prometheus
# ============================================================================
echo ""
echo "1️⃣  Deploying Prometheus..."
helm upgrade --install prometheus prometheus-community/prometheus \
  --namespace observability \
  --version 25.27.0 \
  --set server.persistentVolume.enabled=true \
  --set server.persistentVolume.storageClass=standard \
  --set server.persistentVolume.size=10Gi \
  --set server.resources.limits.cpu=1000m \
  --set server.resources.limits.memory=2Gi \
  --set server.resources.requests.cpu=200m \
  --set server.resources.requests.memory=512Mi \
  --set server.retention=7d \
  --set 'server.extraFlags[0]=web.enable-remote-write-receiver' \
  --set alertmanager.enabled=false \
  --set kube-state-metrics.enabled=true \
  --set prometheus-node-exporter.enabled=false \
  --set prometheus-pushgateway.enabled=false \
  --wait --timeout 5m
echo "   ✅ Prometheus deployed"

# ============================================================================
# 2. Deploy Loki
# ============================================================================
echo ""
echo "2️⃣  Deploying Loki..."

# Create Loki values file for complex config
cat > /tmp/loki-values.yaml <<'EOF'
deploymentMode: SingleBinary

singleBinary:
  replicas: 1
  persistence:
    enabled: true
    storageClass: standard
    size: 10Gi
  resources:
    limits:
      cpu: 1000m
      memory: 2Gi
    requests:
      cpu: 200m
      memory: 512Mi

loki:
  auth_enabled: false
  commonConfig:
    replication_factor: 1
  storage:
    type: filesystem
  schemaConfig:
    configs:
      - from: "2024-01-01"
        store: tsdb
        object_store: filesystem
        schema: v13
        index:
          prefix: index_
          period: 24h

gateway:
  enabled: true
  replicas: 1

backend:
  replicas: 0
read:
  replicas: 0
write:
  replicas: 0

test:
  enabled: false
monitoring:
  selfMonitoring:
    enabled: false
lokiCanary:
  enabled: false
chunksCache:
  enabled: false
resultsCache:
  enabled: false
EOF

helm upgrade --install loki grafana/loki \
  --namespace observability \
  --version 6.16.0 \
  -f /tmp/loki-values.yaml \
  --wait --timeout 5m
echo "   ✅ Loki deployed"

# ============================================================================
# 3. Deploy Grafana
# ============================================================================
echo ""
echo "3️⃣  Deploying Grafana..."
helm upgrade --install grafana grafana/grafana \
  --namespace observability \
  --version 9.2.9 \
  --set adminUser=admin \
  --set adminPassword=admin \
  --set persistence.enabled=true \
  --set persistence.size=5Gi \
  --set persistence.storageClassName=standard \
  --set resources.limits.cpu=500m \
  --set resources.limits.memory=512Mi \
  --set resources.requests.cpu=100m \
  --set resources.requests.memory=128Mi \
  --set service.type=ClusterIP \
  --set service.port=80 \
  --set initChownData.enabled=false \
  --set 'env.GF_EXPLORE_ENABLED=true' \
  --set 'env.GF_ALERTING_ENABLED=true' \
  --set-json 'datasources.datasources\.yaml.apiVersion=1' \
  --set-json 'datasources.datasources\.yaml.datasources[0]={"name":"Prometheus","type":"prometheus","access":"proxy","url":"http://prometheus-server.observability.svc:80","isDefault":true}' \
  --set-json 'datasources.datasources\.yaml.datasources[1]={"name":"Loki","type":"loki","access":"proxy","url":"http://loki-gateway.observability.svc:80"}' \
  --wait --timeout 5m
echo "   ✅ Grafana deployed (admin/admin)"

# ============================================================================
# 4. Deploy OpenTelemetry Collector
# ============================================================================
echo ""
echo "4️⃣  Deploying OpenTelemetry Collector..."

helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts 2>/dev/null || true

cat > /tmp/otel-collector-values.yaml <<'EOF'
mode: deployment
replicaCount: 1

image:
  repository: otel/opentelemetry-collector-contrib
  pullPolicy: IfNotPresent
  tag: "0.105.0"

resources:
  limits:
    cpu: 500m
    memory: 1Gi
  requests:
    cpu: 100m
    memory: 256Mi

ports:
  otlp:
    enabled: true
    containerPort: 4317
    servicePort: 4317
    protocol: TCP
  otlp-http:
    enabled: true
    containerPort: 4318
    servicePort: 4318
    protocol: TCP
  prometheus:
    enabled: true
    containerPort: 8889
    servicePort: 8889
    protocol: TCP

config:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: 0.0.0.0:4317
        http:
          endpoint: 0.0.0.0:4318

  processors:
    batch:
      timeout: 10s
      send_batch_size: 1024
    memory_limiter:
      check_interval: 1s
      limit_mib: 800
      spike_limit_mib: 256
    resource:
      attributes:
        - key: cluster.name
          value: kind-local
          action: insert

  exporters:
    prometheus:
      endpoint: "0.0.0.0:8889"
      send_timestamps: true
    prometheusremotewrite:
      endpoint: http://prometheus-server.observability.svc.cluster.local:80/api/v1/write
      tls:
        insecure: true
    otlphttp/loki:
      endpoint: http://loki-gateway.observability.svc.cluster.local:80/otlp
      tls:
        insecure: true
    debug:
      verbosity: basic

  extensions:
    health_check:
      endpoint: 0.0.0.0:13133

  service:
    extensions: [health_check]
    pipelines:
      metrics:
        receivers: [otlp]
        processors: [memory_limiter, batch, resource]
        exporters: [prometheusremotewrite, prometheus]
      logs:
        receivers: [otlp]
        processors: [memory_limiter, batch, resource]
        exporters: [otlphttp/loki]
      traces:
        receivers: [otlp]
        processors: [memory_limiter, batch]
        exporters: [debug]
    telemetry:
      metrics:
        address: 0.0.0.0:8890

ingress:
  enabled: false
EOF

helm upgrade --install otel-collector open-telemetry/opentelemetry-collector \
  --namespace observability \
  --version 0.127.2 \
  -f /tmp/otel-collector-values.yaml \
  --wait --timeout 5m
echo "   ✅ OpenTelemetry Collector deployed"

# ============================================================================
# 5. Deploy Fluent Bit
# ============================================================================
echo ""
echo "5️⃣  Deploying Fluent Bit..."

helm repo add fluent https://fluent.github.io/helm-charts 2>/dev/null || true

cat > /tmp/fluent-bit-values.yaml <<'EOF'
fullnameOverride: fluent-bit
serviceAccount:
  create: true
tolerations: []
nodeSelector: {}

config:
  service: |
    [SERVICE]
        Flush           1
        Daemon          Off
        Log_Level       info
        Parsers_File    parsers.conf
        HTTP_Server     On
        HTTP_Listen     0.0.0.0
        HTTP_Port       2020

  inputs: |
    [INPUT]
        Name              tail
        Tag               kube.*
        Path              /var/log/containers/*.log
        Exclude_Path      /var/log/containers/fluent-bit-*.log
        Parser            cri
        Mem_Buf_Limit     50MB
        Skip_Long_Lines   On
        Refresh_Interval  5

  filters: |
    [FILTER]
        Name                kubernetes
        Match               kube.*
        Kube_URL            https://kubernetes.default.svc:443
        Kube_CA_File        /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
        Kube_Token_File     /var/run/secrets/kubernetes.io/serviceaccount/token
        Kube_Tag_Prefix     kube.var.log.containers.
        Merge_Log           On
        Keep_Log            Off
        Labels              On
        Annotations         On
        K8S-Logging.Parser  On
        K8S-Logging.Exclude Off

    [FILTER]
        Name                modify
        Match               kube.*
        Add                 cluster.name        kind-local
        Add                 platform.name       cto

    [FILTER]
        Name                modify
        Match               kube.var.log.containers.controller-*_cto_*
        Add                 service.name        agent-controller

    [FILTER]
        Name                modify
        Match               kube.var.log.containers.*tools*_cto_*
        Add                 service.name        tools-mcp

    [FILTER]
        Name                modify
        Match               kube.var.log.containers.*openmemory*_cto_*
        Add                 service.name        openmemory

    [FILTER]
        Name                modify
        Match               kube.var.log.containers.*healer*_cto_*
        Add                 service.name        healer

  outputs: |
    [OUTPUT]
        Name                opentelemetry
        Match               kube.*
        Host                otel-collector-opentelemetry-collector.observability.svc.cluster.local
        Port                4318
        TLS                 Off
        Logs_uri            /v1/logs
        logs_body_key       log
        logs_resource_metadata_key  kubernetes

  customParsers: |
    [PARSER]
        Name        cri
        Format      regex
        Regex       ^(?<time>[^ ]+) (?<stream>stdout|stderr) (?<log>.*)$
        Time_Key    time
        Time_Format %Y-%m-%dT%H:%M:%S.%L%z

hostTail:
  enabled: true
  path: /var/log/containers
EOF

helm upgrade --install fluent-bit fluent/fluent-bit \
  --namespace observability \
  --version 0.47.7 \
  -f /tmp/fluent-bit-values.yaml \
  --wait --timeout 5m
echo "   ✅ Fluent Bit deployed"

# ============================================================================
# 6. Deploy Argo Workflows
# ============================================================================
echo ""
echo "6️⃣  Deploying Argo Workflows..."

# Adopt existing CRDs for Helm (may exist from argo-events or previous installs)
echo "   Adopting existing Argo CRDs..."
kubectl get crds -o name | grep argoproj.io | while read crd; do
  kubectl label "$crd" app.kubernetes.io/managed-by=Helm --overwrite 2>/dev/null || true
  kubectl annotate "$crd" meta.helm.sh/release-name=argo-workflows meta.helm.sh/release-namespace=automation --overwrite 2>/dev/null || true
done

helm upgrade --install argo-workflows argo/argo-workflows \
  --namespace automation \
  --version 0.45.21 \
  --set controller.image.tag=v3.7.1 \
  --set server.image.tag=v3.7.1 \
  --set executor.image.tag=v3.7.1 \
  --set 'server.authModes[0]=server' \
  --set server.secure=false \
  --set 'server.extraArgs[0]=--secure=false' \
  --set server.serviceType=NodePort \
  --set server.servicePort=2746 \
  --set server.serviceNodePort=30081 \
  --set workflow.serviceAccount.create=true \
  --set workflow.serviceAccount.name=argo-workflow \
  --set workflow.rbac.create=true \
  --set 'controller.workflowDefaults.spec.ttlStrategy.secondsAfterCompletion=1800' \
  --set 'controller.workflowDefaults.spec.ttlStrategy.secondsAfterSuccess=1800' \
  --set 'controller.workflowDefaults.spec.ttlStrategy.secondsAfterFailure=7200' \
  --set 'controller.workflowDefaults.spec.podGC.strategy=OnPodCompletion' \
  --set artifactRepository.archiveLogs=true \
  --wait --timeout 5m
echo "   ✅ Argo Workflows deployed"

# ============================================================================
# 7. Deploy Argo Events
# ============================================================================
echo ""
echo "7️⃣  Deploying Argo Events..."
helm upgrade --install argo-events argo/argo-events \
  --namespace automation \
  --version 2.4.16 \
  --set serviceAccount.create=true \
  --set serviceAccount.name=argo-events-controller-sa \
  --set singleNamespace=false \
  --set controller.replicas=1 \
  --set gateway.enabled=true \
  --set sensor.enabled=true \
  --wait --timeout 5m
echo "   ✅ Argo Events deployed"

# ============================================================================
# 8. Deploy EventBus (required for sensors)
# ============================================================================
echo ""
echo "8️⃣  Deploying EventBus..."
cat <<EOF | kubectl apply -f -
apiVersion: argoproj.io/v1alpha1
kind: EventBus
metadata:
  name: default
  namespace: automation
spec:
  nats:
    native:
      replicas: 1
      auth: none
EOF
echo "   ✅ EventBus deployed"

# ============================================================================
# 9. Deploy Argo Events RBAC & ServiceAccounts
# ============================================================================
echo ""
echo "9️⃣  Deploying Argo Events RBAC..."
cat <<'EOF' | kubectl apply -f -
---
# ServiceAccount for Argo Events sensors
apiVersion: v1
kind: ServiceAccount
metadata:
  name: argo-events-sa
  namespace: automation
  labels:
    app.kubernetes.io/name: argo-events-sa
    app.kubernetes.io/part-of: platform
---
# Role for sensors in automation namespace
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: argo-events-executor
  namespace: automation
rules:
  - apiGroups: [""]
    resources: ["pods", "pods/log", "configmaps", "secrets"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["batch"]
    resources: ["jobs"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["argoproj.io"]
    resources: ["workflows", "workflowtemplates", "cronworkflows"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: argo-events-executor-binding
  namespace: automation
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: argo-events-executor
subjects:
  - kind: ServiceAccount
    name: argo-events-sa
    namespace: automation
---
# Role for sensors to create workflows/coderuns in cto namespace
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: argo-events-workflows-cto
  namespace: cto
rules:
  - apiGroups: ["argoproj.io"]
    resources: ["workflows"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: ["agents.platform"]
    resources: ["coderuns"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: ["agents.platform"]
    resources: ["coderuns/status"]
    verbs: ["get", "update", "patch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: argo-events-workflows-binding
  namespace: cto
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: argo-events-workflows-cto
subjects:
  - kind: ServiceAccount
    name: argo-events-sa
    namespace: automation
EOF
echo "   ✅ Argo Events RBAC deployed"

# ============================================================================
# 10. Deploy GitHub Webhook Secret (placeholder - update with real secret)
# ============================================================================
echo ""
echo "🔟  Deploying GitHub Webhook Secret..."
kubectl create secret generic github-webhook-secret \
  --namespace automation \
  --from-literal=secret="kind-local-webhook-secret" \
  --dry-run=client -o yaml | kubectl apply -f -
echo "   ✅ GitHub Webhook Secret deployed (placeholder)"

# ============================================================================
# 11. Deploy GitHub EventSource
# ============================================================================
echo ""
echo "1️⃣1️⃣  Deploying GitHub EventSource..."
cat <<'EOF' | kubectl apply -f -
---
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: github
  namespace: automation
  labels:
    app.kubernetes.io/component: webhooks
spec:
  template:
    metadata:
      labels:
        app.kubernetes.io/component: webhooks
  github:
    org:
      events:
        - "*"
      organizations:
        - "5dlabs"
      webhook:
        endpoint: /github/webhook
        port: "12000"
      secret:
        name: github-webhook-secret
        key: secret
EOF
echo "   ✅ GitHub EventSource deployed"

# ============================================================================
# 12. Deploy CI Failure Remediation Sensor
# ============================================================================
echo ""
echo "1️⃣2️⃣  Deploying CI Failure Remediation Sensor..."
cat <<'EOF' | kubectl apply -f -
---
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: ci-failure-remediation
  namespace: automation
  labels:
    app: ci-remediation
    component: failure-sensor
spec:
  replicas: 1
  template:
    serviceAccountName: argo-events-sa
    container:
      resources:
        requests:
          memory: "128Mi"
          cpu: "100m"
        limits:
          memory: "256Mi"
          cpu: "300m"
  dependencies:
    - name: workflow-failure
      eventSourceName: github
      eventName: org
      filters:
        data:
          - path: body.X-GitHub-Event
            type: string
            value: ["workflow_run"]
          - path: body.action
            type: string
            value: ["completed"]
          - path: body.workflow_run.conclusion
            type: string
            value: ["failure"]
          - path: body.repository.full_name
            type: string
            value: ["5dlabs/cto"]
    - name: job-failure
      eventSourceName: github
      eventName: org
      filters:
        data:
          - path: body.X-GitHub-Event
            type: string
            value: ["workflow_job"]
          - path: body.action
            type: string
            value: ["completed"]
          - path: body.workflow_job.conclusion
            type: string
            value: ["failure"]
          - path: body.repository.full_name
            type: string
            value: ["5dlabs/cto"]
    - name: security-alert
      eventSourceName: github
      eventName: org
      filters:
        data:
          - path: body.X-GitHub-Event
            type: string
            value: ["dependabot_alert", "code_scanning_alert", "secret_scanning_alert"]
          - path: body.action
            type: string
            value: ["created", "reopened"]
          - path: body.repository.full_name
            type: string
            value: ["5dlabs/cto"]
  triggers:
    - template:
        name: healer-workflow-failure
        conditions: "workflow-failure"
        http:
          url: http://healer.cto.svc.cluster.local:8080/api/remediate/ci-failure
          method: POST
          headers:
            Content-Type: application/json
          payload:
            - src:
                dependencyName: workflow-failure
                dataKey: body
              dest: ""
          timeout: 30s
        retryStrategy:
          steps: 3
          duration: "10s"
    - template:
        name: healer-job-failure
        conditions: "job-failure"
        http:
          url: http://healer.cto.svc.cluster.local:8080/api/remediate/ci-failure
          method: POST
          headers:
            Content-Type: application/json
          payload:
            - src:
                dependencyName: job-failure
                dataKey: body
              dest: ""
          timeout: 30s
    - template:
        name: healer-security-alert
        conditions: "security-alert"
        http:
          url: http://healer.cto.svc.cluster.local:8080/api/remediate/security-alert
          method: POST
          headers:
            Content-Type: application/json
          payload:
            - src:
                dependencyName: security-alert
                dataKey: body
              dest: ""
          timeout: 30s
EOF
echo "   ✅ CI Failure Remediation Sensor deployed"

# ============================================================================
# 13. Deploy PR Merged to Main Sensor
# ============================================================================
echo ""
echo "1️⃣3️⃣  Deploying PR Merged to Main Sensor..."
kubectl apply -f "${REPO_ROOT}/infra/gitops/resources/github-webhooks/merge-to-main-sensor.yaml"
echo "   ✅ PR Merged to Main Sensor deployed"

# ============================================================================
# 15. Deploy Workflow Templates
# ============================================================================
echo ""
echo "1️⃣5️⃣  Deploying Workflow Templates..."
helm upgrade --install workflow-templates "${CHARTS_DIR}/workflow-templates" \
  --namespace automation \
  --wait --timeout 3m 2>/dev/null || echo "   ⚠️  workflow-templates chart not found or failed"
echo "   ✅ Workflow Templates deployed"

# ============================================================================
# 16. Deploy tweakcn
# ============================================================================
echo ""
echo "1️⃣6️⃣  Deploying tweakcn..."

# Check if image exists
if docker images | grep -q "ghcr.io/5dlabs/tweakcn.*kind-local"; then
  helm upgrade --install tweakcn "${CHARTS_DIR}/universal-app" \
    --namespace cto \
    --set fullnameOverride=tweakcn \
    --set image.repository=ghcr.io/5dlabs/tweakcn \
    --set image.tag=kind-local \
    --set image.pullPolicy=IfNotPresent \
    --set imagePullSecrets=null \
    --set service.type=ClusterIP \
    --set service.port=3000 \
    --set ingress.enabled=false \
    --set persistence.enabled=false \
    --set 'env[0].name=NODE_ENV' \
    --set 'env[0].value=production' \
    --set 'env[1].name=PORT' \
    --set 'env[1].value=3000' \
    --set resources.requests.cpu=50m \
    --set resources.requests.memory=128Mi \
    --set resources.limits.cpu=500m \
    --set resources.limits.memory=512Mi \
    --set securityContext.readOnlyRootFilesystem=false \
    --set securityContext.runAsNonRoot=true \
    --set securityContext.runAsUser=1001 \
    --wait --timeout 3m
  echo "   ✅ tweakcn deployed"
else
  echo "   ⚠️  tweakcn image not found (ghcr.io/5dlabs/tweakcn:kind-local)"
  echo "      Build and load with: docker build -t ghcr.io/5dlabs/tweakcn:kind-local -f infra/images/tweakcn/Dockerfile . && kind load docker-image ghcr.io/5dlabs/tweakcn:kind-local"
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     Deployment Complete!                                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Observability Stack:"
kubectl get pods -n observability
echo ""
echo "⚡ Automation Stack:"
kubectl get pods -n automation
echo ""
echo "🔧 CTO Stack:"
kubectl get pods -n cto
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Port Forward Commands:"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "# Grafana (admin/admin)"
echo "kubectl port-forward svc/grafana -n observability 3000:80"
echo ""
echo "# Prometheus"
echo "kubectl port-forward svc/prometheus-server -n observability 9090:80"
echo ""
echo "# Loki"
echo "kubectl port-forward svc/loki-gateway -n observability 3100:80"
echo ""
echo "# Tools MCP Server"
echo "kubectl port-forward svc/tools -n cto 3001:3000"
echo ""
echo "# Healer"
echo "kubectl port-forward svc/healer -n cto 8080:8080"
echo ""
echo "# OpenMemory"
echo "kubectl port-forward svc/openmemory -n cto 8081:8080"
echo ""
echo "# tweakcn"
echo "kubectl port-forward svc/tweakcn -n cto 3002:3000"
echo ""

