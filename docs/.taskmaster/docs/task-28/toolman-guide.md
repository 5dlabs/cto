# Toolman Guide: Comprehensive Monitoring and Alerting for Long-Running Multi-Agent Workflows

## Overview

This guide provides comprehensive instructions for implementing, configuring, and managing monitoring and alerting systems for long-running multi-agent workflows. The system includes health checking, stuck workflow detection, resource monitoring, automated cleanup, and operational dashboards.

## Tool Categories

### 1. Prometheus Monitoring Stack

#### Prometheus (`prometheus`)

**Purpose**: Core metrics collection and alerting for workflow monitoring

**Installation and Configuration**:
```bash
# Install Prometheus Operator
kubectl apply -f https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/bundle.yaml

# Create Prometheus instance for workflow monitoring
kubectl apply -f - <<EOF
apiVersion: monitoring.coreos.com/v1
kind: Prometheus
metadata:
  name: workflow-prometheus
  namespace: taskmaster
spec:
  replicas: 2
  serviceAccountName: prometheus
  serviceMonitorSelector:
    matchLabels:
      app: workflow-monitoring
  ruleSelector:
    matchLabels:
      prometheus: workflow-prometheus
  retention: 30d
  storage:
    volumeClaimTemplate:
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 100Gi
  resources:
    requests:
      memory: 2Gi
      cpu: 1000m
    limits:
      memory: 4Gi
      cpu: 2000m
EOF
```

**Custom Metrics Collection**:
```bash
# Create ServiceMonitor for agent health endpoints
kubectl apply -f - <<EOF
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: agent-health-monitor
  namespace: taskmaster
  labels:
    app: workflow-monitoring
spec:
  selector:
    matchLabels:
      app: taskmaster-agent
  endpoints:
  - port: http-health
    path: /health
    interval: 30s
    scrapeTimeout: 10s
    metricRelabelings:
    - sourceLabels: [__name__]
      regex: 'agent_health_.*'
      targetLabel: __name__
      replacement: 'workflow_${1}'
EOF

# Create PrometheusRule for workflow alerts
kubectl apply -f - <<EOF
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: workflow-monitoring-rules
  namespace: taskmaster
  labels:
    prometheus: workflow-prometheus
spec:
  groups:
  - name: workflow.health
    interval: 1m
    rules:
    - alert: AgentHealthCheckFailing
      expr: agent_health_status{status!="healthy"} == 1
      for: 2m
      labels:
        severity: warning
        component: agent-health
      annotations:
        summary: "Agent {{ \$labels.agent_type }} health check failing"
        description: "Health check for {{ \$labels.agent_type }} agent has been failing for 2 minutes"

    - alert: WorkflowStuckDetection
      expr: |
        (time() - argo_workflow_status_phase_start_time{phase!="Succeeded",phase!="Failed"}) > 21600
      for: 1m
      labels:
        severity: critical
        component: workflow-execution
      annotations:
        summary: "Workflow {{ \$labels.name }} stuck for over 6 hours"
        description: "Workflow {{ \$labels.name }} has been in {{ \$labels.phase }} phase for {{ humanizeDuration \$value }}"
EOF
```

**Query Examples**:
```bash
# Check workflow age distribution
curl -G 'http://prometheus:9090/api/v1/query' \
  --data-urlencode 'query=argo_workflow_start_time'

# Monitor resource usage by agent
curl -G 'http://prometheus:9090/api/v1/query' \
  --data-urlencode 'query=avg by (agent) (workflow_cpu_usage_percent)'

# Find stuck workflows
curl -G 'http://prometheus:9090/api/v1/query' \
  --data-urlencode 'query=(time() - argo_workflow_status_phase_start_time{phase!="Succeeded"}) > 21600'

# Check PVC usage alerts
curl -G 'http://prometheus:9090/api/v1/query' \
  --data-urlencode 'query=workflow_pvc_usage_percent > 80'
```

#### Prometheus CLI Tools (`prometheus_client`)

**Purpose**: Rule validation and query testing

**Rule Validation**:
```bash
# Install promtool
wget https://github.com/prometheus/prometheus/releases/download/v2.40.0/prometheus-2.40.0.linux-amd64.tar.gz
tar xzf prometheus-2.40.0.linux-amd64.tar.gz
sudo cp prometheus-2.40.0.linux-amd64/promtool /usr/local/bin/

# Validate PrometheusRule files
promtool check rules workflow-monitoring-rules.yaml

# Test queries
promtool query instant 'http://prometheus:9090' 'up{job="workflow-monitoring"}'

# Check rule syntax
promtool check rules - <<EOF
groups:
- name: test
  rules:
  - alert: TestAlert
    expr: up == 0
    for: 1m
EOF
```

### 2. Grafana Visualization

#### Grafana (`grafana`)

**Purpose**: Operational dashboards and workflow visualization

**Dashboard Creation**:
```bash
# Install Grafana
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: grafana
  namespace: taskmaster
spec:
  replicas: 1
  selector:
    matchLabels:
      app: grafana
  template:
    metadata:
      labels:
        app: grafana
    spec:
      containers:
      - name: grafana
        image: grafana/grafana:9.3.0
        ports:
        - containerPort: 3000
        env:
        - name: GF_SECURITY_ADMIN_PASSWORD
          value: "admin123"
        volumeMounts:
        - name: grafana-storage
          mountPath: /var/lib/grafana
        - name: dashboard-config
          mountPath: /etc/grafana/provisioning/dashboards
        - name: datasource-config
          mountPath: /etc/grafana/provisioning/datasources
      volumes:
      - name: grafana-storage
        emptyDir: {}
      - name: dashboard-config
        configMap:
          name: grafana-dashboard-config
      - name: datasource-config
        configMap:
          name: grafana-datasource-config
EOF

# Create datasource configuration
kubectl create configmap grafana-datasource-config --from-literal=prometheus.yaml="
apiVersion: 1
datasources:
- name: Prometheus
  type: prometheus
  url: http://prometheus:9090
  access: proxy
  isDefault: true
"
```

**Long-Running Workflow Dashboard**:
```bash
# Create dashboard ConfigMap
kubectl create configmap workflow-dashboard --from-literal=dashboard.json='
{
  "dashboard": {
    "title": "Long-Running Workflow Monitoring",
    "tags": ["taskmaster", "workflows"],
    "panels": [
      {
        "id": 1,
        "title": "Active Workflows by Age",
        "type": "histogram",
        "targets": [
          {
            "expr": "(time() - argo_workflow_start_time{phase!=\"Succeeded\",phase!=\"Failed\"}) / 86400",
            "legendFormat": "Days Active"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Stuck Workflows",
        "type": "stat",
        "targets": [
          {
            "expr": "count((time() - argo_workflow_status_phase_start_time{phase!=\"Succeeded\",phase!=\"Failed\"}) > 21600)",
            "legendFormat": "Stuck Workflows"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "id": 3,
        "title": "Resource Usage by Agent",
        "type": "timeseries",
        "targets": [
          {
            "expr": "avg by (agent) (workflow_cpu_usage_percent)",
            "legendFormat": "{{agent}} CPU %"
          },
          {
            "expr": "avg by (agent) (workflow_memory_usage_bytes / 1024 / 1024 / 1024)",
            "legendFormat": "{{agent}} Memory GB"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 8}
      }
    ]
  }
}'

# Import dashboard
curl -X POST http://admin:admin123@grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @dashboard.json
```

### 3. AlertManager Configuration

#### AlertManager (`alertmanager`)

**Purpose**: Alert routing and notification management

**Configuration Setup**:
```bash
# Create AlertManager configuration
kubectl apply -f - <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: alertmanager-config
  namespace: taskmaster
stringData:
  alertmanager.yml: |
    global:
      smtp_smarthost: 'smtp.company.com:587'
      smtp_from: 'alerts@company.com'
      slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
      pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'

    route:
      group_by: ['alertname', 'severity']
      group_wait: 30s
      group_interval: 5m
      repeat_interval: 4h
      receiver: 'default-receiver'
      routes:
      - match:
          severity: critical
        receiver: 'pagerduty-critical'
      - match:
          severity: warning
        receiver: 'slack-warnings'

    receivers:
    - name: 'default-receiver'
      email_configs:
      - to: 'team@company.com'
        subject: 'TaskMaster Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}

    - name: 'pagerduty-critical'
      pagerduty_configs:
      - routing_key: 'YOUR-PAGERDUTY-ROUTING-KEY'
        description: '{{ .CommonAnnotations.summary }}'
        details:
          workflow: '{{ .CommonLabels.workflow_name }}'
          namespace: '{{ .CommonLabels.namespace }}'

    - name: 'slack-warnings'
      slack_configs:
      - channel: '#taskmaster-alerts'
        title: 'TaskMaster Warning'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Annotations.summary }}
          *Description:* {{ .Annotations.description }}
          *Workflow:* {{ .Labels.workflow_name }}
          {{ end }}

    inhibit_rules:
    - source_match:
        severity: 'critical'
      target_match:
        severity: 'warning'
      equal: ['alertname', 'workflow_name']
EOF
```

**Alert Testing**:
```bash
# Test alert firing
curl -X POST http://alertmanager:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '[
    {
      "labels": {
        "alertname": "TestAlert",
        "severity": "warning",
        "workflow_name": "test-workflow"
      },
      "annotations": {
        "summary": "Test alert for workflow monitoring",
        "description": "This is a test alert to verify notification routing"
      }
    }
  ]'

# Check alert status
curl http://alertmanager:9093/api/v1/alerts

# Silence alert
curl -X POST http://alertmanager:9093/api/v1/silences \
  -H "Content-Type: application/json" \
  -d '{
    "matchers": [
      {
        "name": "alertname",
        "value": "TestAlert"
      }
    ],
    "startsAt": "2024-01-01T00:00:00Z",
    "endsAt": "2024-01-01T01:00:00Z",
    "createdBy": "admin",
    "comment": "Testing silence functionality"
  }'
```

### 4. Health Check Implementation

#### Python Health Endpoints (`python_metrics`)

**Purpose**: Custom health checks and metrics collection

**Health Check Server Implementation**:
```python
#!/usr/bin/env python3
import json
import time
import subprocess
import requests
from datetime import datetime
from flask import Flask, jsonify
from prometheus_client import Counter, Histogram, Gauge, generate_latest

app = Flask(__name__)

# Metrics
health_check_requests = Counter('agent_health_check_requests_total', 'Health check requests', ['agent_type', 'status'])
health_check_duration = Histogram('agent_health_check_duration_seconds', 'Health check duration', ['agent_type'])
health_status = Gauge('agent_health_status', 'Agent health status', ['agent_type', 'check_name'])

class HealthChecker:
    def __init__(self, agent_type):
        self.agent_type = agent_type
        
    def check_git_access(self):
        """Check git repository access"""
        try:
            result = subprocess.run(['git', 'status'], 
                                  capture_output=True, timeout=5)
            return result.returncode == 0
        except Exception:
            return False
    
    def check_workspace_access(self):
        """Check workspace file system access"""
        try:
            import os
            workspace_path = os.getenv('WORKSPACE_PATH', '/workspace')
            return os.path.exists(workspace_path) and os.access(workspace_path, os.W_OK)
        except Exception:
            return False
    
    def check_mcp_connection(self):
        """Check MCP server connectivity (Rex-specific)"""
        if self.agent_type != 'rex':
            return True
            
        try:
            mcp_endpoint = os.getenv('MCP_SERVER_URL', 'http://mcp-server:8080')
            response = requests.get(f"{mcp_endpoint}/health", timeout=5)
            return response.status_code == 200
        except Exception:
            return False
    
    def check_cargo_tools(self):
        """Check cargo tool availability (Cleo/Tess-specific)"""
        if self.agent_type == 'rex':
            return True
            
        tools = ['cargo']
        if self.agent_type == 'cleo':
            tools.extend(['cargo-fmt', 'cargo-clippy'])
        elif self.agent_type == 'tess':
            tools.extend(['cargo-test'])
            
        for tool in tools:
            try:
                result = subprocess.run([tool, '--version'], 
                                      capture_output=True, timeout=5)
                if result.returncode != 0:
                    return False
            except Exception:
                return False
        return True

# Health check endpoint
@app.route('/health')
def health_check():
    agent_type = os.getenv('AGENT_TYPE', 'unknown')
    checker = HealthChecker(agent_type)
    
    with health_check_duration.labels(agent_type=agent_type).time():
        checks = {
            'git_access': checker.check_git_access(),
            'workspace_access': checker.check_workspace_access(),
            'mcp_connection': checker.check_mcp_connection(),
            'cargo_tools': checker.check_cargo_tools()
        }
        
        # Update metrics
        for check_name, status in checks.items():
            health_status.labels(agent_type=agent_type, check_name=check_name).set(1 if status else 0)
        
        overall_status = all(checks.values())
        status_label = 'healthy' if overall_status else 'unhealthy'
        
        health_check_requests.labels(agent_type=agent_type, status=status_label).inc()
        
        return jsonify({
            'status': status_label,
            'timestamp': datetime.utcnow().isoformat(),
            'checks': checks,
            'agent_type': agent_type
        }), 200 if overall_status else 503

# Metrics endpoint
@app.route('/metrics')
def metrics():
    return generate_latest()

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

**Synthetic Health Check Workflow**:
```bash
# Create synthetic health check script
cat > /tmp/synthetic-health-check.py <<'EOF'
#!/usr/bin/env python3
import requests
import time
import json
import sys
from prometheus_client import push_to_gateway, CollectorRegistry, Gauge

def run_synthetic_checks():
    registry = CollectorRegistry()
    synthetic_check_status = Gauge('synthetic_health_check_status', 
                                 'Synthetic health check status', 
                                 ['agent_type'], registry=registry)
    
    agents = ['rex', 'cleo', 'tess']
    all_healthy = True
    
    for agent in agents:
        try:
            response = requests.get(f'http://{agent}-agent:8080/health', timeout=10)
            healthy = response.status_code == 200
            
            synthetic_check_status.labels(agent_type=agent).set(1 if healthy else 0)
            
            if not healthy:
                all_healthy = False
                print(f"WARN: {agent} agent health check failed")
            else:
                print(f"OK: {agent} agent health check passed")
                
        except Exception as e:
            synthetic_check_status.labels(agent_type=agent).set(0)
            all_healthy = False
            print(f"ERROR: {agent} agent health check exception: {e}")
    
    # Push metrics to Prometheus
    try:
        push_to_gateway('pushgateway:9091', job='synthetic-health-checks', 
                       registry=registry)
    except Exception as e:
        print(f"ERROR: Failed to push metrics: {e}")
    
    return 0 if all_healthy else 1

if __name__ == '__main__':
    sys.exit(run_synthetic_checks())
EOF

# Create CronJob for synthetic checks
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: CronJob
metadata:
  name: synthetic-health-checks
  namespace: taskmaster
spec:
  schedule: "0 */6 * * *"  # Every 6 hours
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: health-check
            image: python:3.9-slim
            command: ["python3", "/scripts/synthetic-health-check.py"]
            volumeMounts:
            - name: health-script
              mountPath: /scripts
          restartPolicy: OnFailure
          volumes:
          - name: health-script
            configMap:
              name: synthetic-health-script
EOF
```

### 5. Resource Monitoring Tools

#### System Resource Monitoring (`psutil`)

**Purpose**: Detailed system resource collection and analysis

**Resource Metrics Collector**:
```python
#!/usr/bin/env python3
import psutil
import time
import json
import kubernetes
from prometheus_client import CollectorRegistry, Gauge, push_to_gateway

class ResourceMonitor:
    def __init__(self):
        self.registry = CollectorRegistry()
        self.setup_metrics()
        
    def setup_metrics(self):
        self.cpu_usage = Gauge('workflow_detailed_cpu_usage_percent', 
                             'Detailed CPU usage', 
                             ['workflow', 'agent'], registry=self.registry)
        self.memory_usage = Gauge('workflow_detailed_memory_usage_bytes',
                                'Detailed memory usage',
                                ['workflow', 'agent', 'type'], registry=self.registry)
        self.disk_io = Gauge('workflow_disk_io_bytes_total',
                           'Disk I/O bytes',
                           ['workflow', 'agent', 'type'], registry=self.registry)
        self.network_io = Gauge('workflow_network_io_bytes_total',
                              'Network I/O bytes', 
                              ['workflow', 'agent', 'type'], registry=self.registry)
    
    def collect_system_metrics(self):
        """Collect detailed system metrics"""
        
        # CPU metrics
        cpu_percent = psutil.cpu_percent(interval=1, percpu=False)
        
        # Memory metrics
        memory = psutil.virtual_memory()
        
        # Disk I/O metrics
        disk_io = psutil.disk_io_counters()
        
        # Network I/O metrics  
        network_io = psutil.net_io_counters()
        
        # Update metrics (simplified for example)
        workflow_name = os.getenv('WORKFLOW_NAME', 'unknown')
        agent_type = os.getenv('AGENT_TYPE', 'unknown')
        
        self.cpu_usage.labels(workflow=workflow_name, agent=agent_type).set(cpu_percent)
        
        self.memory_usage.labels(workflow=workflow_name, agent=agent_type, type='used').set(memory.used)
        self.memory_usage.labels(workflow=workflow_name, agent=agent_type, type='available').set(memory.available)
        
        if disk_io:
            self.disk_io.labels(workflow=workflow_name, agent=agent_type, type='read').set(disk_io.read_bytes)
            self.disk_io.labels(workflow=workflow_name, agent=agent_type, type='write').set(disk_io.write_bytes)
        
        if network_io:
            self.network_io.labels(workflow=workflow_name, agent=agent_type, type='received').set(network_io.bytes_recv)
            self.network_io.labels(workflow=workflow_name, agent=agent_type, type='sent').set(network_io.bytes_sent)
    
    def collect_kubernetes_metrics(self):
        """Collect Kubernetes-specific metrics"""
        try:
            kubernetes.config.load_incluster_config()
            v1 = kubernetes.client.CoreV1Api()
            
            # Get PVC usage
            pvcs = v1.list_namespaced_persistent_volume_claim('taskmaster')
            
            pvc_usage = Gauge('workflow_pvc_usage_percent',
                            'PVC usage percentage',
                            ['pvc_name', 'namespace'], registry=self.registry)
            
            for pvc in pvcs.items:
                # This would require additional logic to get actual usage
                # For now, using placeholder
                pvc_usage.labels(pvc_name=pvc.metadata.name, 
                               namespace=pvc.metadata.namespace).set(50)
                
        except Exception as e:
            print(f"Error collecting Kubernetes metrics: {e}")
    
    def push_metrics(self):
        """Push metrics to Prometheus Pushgateway"""
        try:
            push_to_gateway('pushgateway:9091', job='resource-monitoring',
                           registry=self.registry)
        except Exception as e:
            print(f"Error pushing metrics: {e}")
    
    def run_collection(self):
        """Main collection loop"""
        self.collect_system_metrics()
        self.collect_kubernetes_metrics()
        self.push_metrics()

if __name__ == '__main__':
    monitor = ResourceMonitor()
    monitor.run_collection()
```

### 6. Workflow Cleanup and Archival

#### AWS CLI (`aws_cli`)

**Purpose**: S3 archival operations and lifecycle management

**Archive Management**:
```bash
# Configure AWS CLI
aws configure set aws_access_key_id "$AWS_ACCESS_KEY_ID"
aws configure set aws_secret_access_key "$AWS_SECRET_ACCESS_KEY"
aws configure set region us-west-2

# Create S3 bucket for archives
aws s3 mb s3://taskmaster-workflow-archives

# Set up lifecycle policy
cat > /tmp/lifecycle-policy.json <<EOF
{
  "Rules": [
    {
      "ID": "WorkflowArchiveLifecycle",
      "Status": "Enabled",
      "Filter": {"Prefix": "workflow-archives/"},
      "Transitions": [
        {
          "Days": 30,
          "StorageClass": "STANDARD_IA"
        },
        {
          "Days": 90,
          "StorageClass": "GLACIER"
        },
        {
          "Days": 365,
          "StorageClass": "DEEP_ARCHIVE"
        }
      ],
      "Expiration": {
        "Days": 2555  # 7 years for compliance
      }
    }
  ]
}
EOF

aws s3api put-bucket-lifecycle-configuration \
  --bucket taskmaster-workflow-archives \
  --lifecycle-configuration file:///tmp/lifecycle-policy.json

# Archive workflow function
archive_workflow() {
  local workflow_name="$1"
  local namespace="$2"
  local timestamp=$(date +%Y%m%d_%H%M%S)
  
  echo "Archiving workflow: $workflow_name"
  
  # Get workflow definition
  kubectl get workflow "$workflow_name" -n "$namespace" -o json > "/tmp/${workflow_name}.json"
  
  # Compress and upload
  gzip "/tmp/${workflow_name}.json"
  
  aws s3 cp "/tmp/${workflow_name}.json.gz" \
    "s3://taskmaster-workflow-archives/workflow-archives/$namespace/$workflow_name/${timestamp}.json.gz" \
    --metadata workflow-name="$workflow_name",namespace="$namespace",archive-date="$timestamp"
  
  echo "Archived to s3://taskmaster-workflow-archives/workflow-archives/$namespace/$workflow_name/${timestamp}.json.gz"
  
  # Cleanup local files
  rm -f "/tmp/${workflow_name}.json.gz"
}

# Retrieve archived workflow
retrieve_workflow() {
  local workflow_name="$1"
  local namespace="$2"
  local archive_date="$3"
  
  aws s3 cp \
    "s3://taskmaster-workflow-archives/workflow-archives/$namespace/$workflow_name/${archive_date}.json.gz" \
    "/tmp/${workflow_name}-${archive_date}.json.gz"
  
  gunzip "/tmp/${workflow_name}-${archive_date}.json.gz"
  
  echo "Retrieved workflow definition to /tmp/${workflow_name}-${archive_date}.json"
}

# List archived workflows
list_archived_workflows() {
  local namespace="$1"
  
  aws s3 ls --recursive "s3://taskmaster-workflow-archives/workflow-archives/$namespace/" \
    --human-readable --summarize
}
```

## Best Practices

### 1. Health Check Best Practices

```bash
# Comprehensive health check implementation
implement_health_checks() {
  echo "=== Implementing Health Checks ==="
  
  # 1. Create health check endpoints with proper timeouts
  # 2. Use appropriate HTTP status codes
  # 3. Include detailed check results in response
  # 4. Implement proper error handling
  # 5. Use structured logging
  
  # Example health check validation
  validate_health_endpoint() {
    local agent="$1"
    local endpoint="http://${agent}-agent:8080/health"
    
    echo "Testing $agent health endpoint..."
    
    response=$(curl -s -w "%{http_code}" -o /tmp/health_response.json "$endpoint")
    http_code=${response: -3}
    
    if [ "$http_code" = "200" ]; then
      echo "✓ $agent health check responding correctly"
      
      # Validate response structure
      if jq -e '.status and .timestamp and .checks and .agent_type' /tmp/health_response.json > /dev/null; then
        echo "✓ $agent health response structure valid"
      else
        echo "✗ $agent health response structure invalid"
      fi
    else
      echo "✗ $agent health check failed with HTTP $http_code"
    fi
  }
  
  for agent in rex cleo tess; do
    validate_health_endpoint "$agent"
  done
}
```

### 2. Monitoring Performance Optimization

```bash
# Optimize monitoring performance
optimize_monitoring() {
  echo "=== Optimizing Monitoring Performance ==="
  
  # 1. Configure appropriate scrape intervals
  # 2. Use metric relabeling to reduce cardinality
  # 3. Implement efficient queries
  # 4. Configure proper retention policies
  
  # Check Prometheus performance
  check_prometheus_performance() {
    echo "Checking Prometheus performance..."
    
    # Query Prometheus self-metrics
    curl -G 'http://prometheus:9090/api/v1/query' \
      --data-urlencode 'query=prometheus_tsdb_head_samples_appended_total'
    
    # Check ingestion rate
    curl -G 'http://prometheus:9090/api/v1/query' \
      --data-urlencode 'query=rate(prometheus_tsdb_head_samples_appended_total[5m])'
    
    # Monitor memory usage
    curl -G 'http://prometheus:9090/api/v1/query' \
      --data-urlencode 'query=process_resident_memory_bytes{job="prometheus"}'
  }
  
  # Optimize dashboard queries
  optimize_dashboard_queries() {
    echo "Optimizing dashboard queries..."
    
    # Use recording rules for expensive queries
    cat > /tmp/recording-rules.yaml <<EOF
groups:
- name: workflow.recording
  interval: 1m
  rules:
  - record: workflow:cpu_usage_avg
    expr: avg by (agent) (workflow_cpu_usage_percent)
  - record: workflow:memory_usage_avg
    expr: avg by (agent) (workflow_memory_usage_bytes)
  - record: workflow:stuck_count
    expr: count((time() - argo_workflow_status_phase_start_time{phase!="Succeeded",phase!="Failed"}) > 21600)
EOF
    
    kubectl apply -f /tmp/recording-rules.yaml
  }
  
  check_prometheus_performance
  optimize_dashboard_queries
}
```

### 3. Alert Management

```bash
# Comprehensive alert management
manage_alerts() {
  echo "=== Managing Alerts ==="
  
  # Test alert firing
  test_alert_firing() {
    echo "Testing alert firing..."
    
    # Create test metric to trigger alert
    curl -X POST http://pushgateway:9091/metrics/job/test-alert <<EOF
test_metric{severity="critical"} 1
EOF
    
    # Wait for alert to fire
    sleep 60
    
    # Check if alert is active
    alerts=$(curl -s http://alertmanager:9093/api/v1/alerts | jq '.data[] | select(.labels.alertname=="TestAlert")')
    
    if [ -n "$alerts" ]; then
      echo "✓ Alert firing correctly"
    else
      echo "✗ Alert not firing"
    fi
  }
  
  # Test alert routing
  test_alert_routing() {
    echo "Testing alert routing..."
    
    # Send test alert to AlertManager
    curl -X POST http://alertmanager:9093/api/v1/alerts \
      -H "Content-Type: application/json" \
      -d '[{
        "labels": {
          "alertname": "RoutingTest",
          "severity": "warning"
        },
        "annotations": {
          "summary": "Test alert routing"
        }
      }]'
    
    echo "Test alert sent to AlertManager"
  }
  
  # Check alert manager status
  check_alertmanager_status() {
    echo "Checking AlertManager status..."
    
    # Check AlertManager health
    curl -f http://alertmanager:9093/-/healthy
    
    # List active alerts
    curl -s http://alertmanager:9093/api/v1/alerts | jq '.data[].labels.alertname' | sort | uniq -c
    
    # Check silences
    curl -s http://alertmanager:9093/api/v1/silences | jq '.data[].comment'
  }
  
  test_alert_firing
  test_alert_routing  
  check_alertmanager_status
}
```

### 4. Operational Procedures

```bash
# Complete operational workflow
operational_workflow() {
  echo "=== Running Operational Workflow ==="
  
  # Daily health check
  daily_health_check() {
    echo "Running daily health check..."
    
    # Check all agents
    for agent in rex cleo tess; do
      echo "Checking $agent agent health..."
      curl -f "http://${agent}-agent:8080/health" > /dev/null
      if [ $? -eq 0 ]; then
        echo "✓ $agent healthy"
      else
        echo "✗ $agent unhealthy"
      fi
    done
    
    # Check stuck workflows
    stuck_workflows=$(curl -s -G 'http://prometheus:9090/api/v1/query' \
      --data-urlencode 'query=count((time() - argo_workflow_status_phase_start_time{phase!="Succeeded",phase!="Failed"}) > 21600)' | \
      jq -r '.data.result[0].value[1]')
    
    echo "Stuck workflows: $stuck_workflows"
    
    # Check resource usage
    high_cpu_pods=$(curl -s -G 'http://prometheus:9090/api/v1/query' \
      --data-urlencode 'query=workflow_cpu_usage_percent > 90' | \
      jq -r '.data.result[] | .metric.workflow')
    
    if [ -n "$high_cpu_pods" ]; then
      echo "High CPU usage pods: $high_cpu_pods"
    fi
  }
  
  # Weekly cleanup verification
  weekly_cleanup_verification() {
    echo "Running weekly cleanup verification..."
    
    # Check cleanup job status
    kubectl get cronjobs workflow-cleanup -n taskmaster
    
    # Verify archive upload
    aws s3 ls s3://taskmaster-workflow-archives/workflow-archives/ --recursive | tail -10
    
    # Check PVC usage
    kubectl get pvc -n taskmaster --sort-by=.status.capacity.storage
  }
  
  daily_health_check
  weekly_cleanup_verification
}
```

This comprehensive guide provides all the tools and procedures needed to implement, monitor, and maintain a robust monitoring system for long-running multi-agent workflows. Follow these patterns and best practices to ensure operational excellence and proactive issue detection.