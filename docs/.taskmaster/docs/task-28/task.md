# Task 28: Implement Comprehensive Monitoring and Alerting for Long-Running Multi-Agent Workflows

## Overview

This task implements a comprehensive monitoring and alerting system specifically designed for long-running multi-agent workflows that may run for days or weeks. The system provides health checks, stuck workflow detection, resource tracking, and automated cleanup strategies to ensure operational excellence.

## Technical Requirements

### System Architecture

1. **Health Check Infrastructure**
   - Kubernetes liveness/readiness probes with custom health endpoints
   - Workflow heartbeat mechanism using Argo Workflows metrics
   - Synthetic health check workflows for system validation
   - Dead letter queue pattern for failed health checks

2. **Stuck Workflow Detection**
   - Prometheus alerts for workflows stuck at same stage >6 hours
   - Workflow age monitoring with graduated thresholds
   - AlertManager integration with PagerDuty/Slack
   - Automatic stuck reason analysis using controller logs

3. **Resource Consumption Monitoring**
   - Pod-level metrics collection (CPU, memory, disk I/O)
   - PVC usage monitoring with capacity alerts
   - GitHub API rate limit tracking per agent
   - Network egress monitoring for cost optimization

4. **Automated Cleanup Strategies**
   - Graduated cleanup policies based on workflow state
   - Workflow archival to S3/MinIO before deletion
   - PVC cleanup for orphaned volumes
   - Compliance-aware retention policies

## Implementation Guide

### Step 1: Health Check System Implementation

#### Custom Health Endpoints for Agents

Create health check endpoints for each agent type:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: agent-health-config
  namespace: taskmaster
data:
  rex-health.py: |
    #!/usr/bin/env python3
    import json
    import time
    from datetime import datetime, timedelta
    from flask import Flask, jsonify
    
    app = Flask(__name__)
    
    @app.route('/health')
    def health_check():
        checks = {
            'mcp_server': check_mcp_connection(),
            'git_access': check_git_access(),
            'documentation_query': check_documentation_query(),
            'workspace_access': check_workspace_access()
        }
        
        overall_status = all(checks.values())
        
        return jsonify({
            'status': 'healthy' if overall_status else 'unhealthy',
            'timestamp': datetime.utcnow().isoformat(),
            'checks': checks,
            'agent_type': 'rex'
        }), 200 if overall_status else 503
    
    def check_mcp_connection():
        # Implement MCP server connectivity check
        try:
            # Mock MCP connection test
            return True
        except:
            return False
    
    def check_git_access():
        # Implement git access check
        import subprocess
        try:
            result = subprocess.run(['git', 'status'], capture_output=True, timeout=5)
            return result.returncode == 0
        except:
            return False
    
    if __name__ == '__main__':
        app.run(host='0.0.0.0', port=8080)

  cleo-health.py: |
    #!/usr/bin/env python3
    import json
    from datetime import datetime
    from flask import Flask, jsonify
    
    app = Flask(__name__)
    
    @app.route('/health')
    def health_check():
        checks = {
            'cargo_fmt': check_cargo_fmt(),
            'cargo_clippy': check_cargo_clippy(),
            'git_access': check_git_access(),
            'workspace_access': check_workspace_access()
        }
        
        overall_status = all(checks.values())
        
        return jsonify({
            'status': 'healthy' if overall_status else 'unhealthy',
            'timestamp': datetime.utcnow().isoformat(),
            'checks': checks,
            'agent_type': 'cleo'
        }), 200 if overall_status else 503
    
    def check_cargo_fmt():
        import subprocess
        try:
            result = subprocess.run(['cargo', 'fmt', '--version'], capture_output=True, timeout=5)
            return result.returncode == 0
        except:
            return False
    
    if __name__ == '__main__':
        app.run(host='0.0.0.0', port=8080)

  tess-health.py: |
    #!/usr/bin/env python3
    import json
    from datetime import datetime
    from flask import Flask, jsonify
    
    app = Flask(__name__)
    
    @app.route('/health')
    def health_check():
        checks = {
            'cargo_test': check_cargo_test(),
            'coverage_tools': check_coverage_tools(),
            'database_access': check_database_access(),
            'deployment_validation': check_deployment_validation()
        }
        
        overall_status = all(checks.values())
        
        return jsonify({
            'status': 'healthy' if overall_status else 'unhealthy',
            'timestamp': datetime.utcnow().isoformat(),
            'checks': checks,
            'agent_type': 'tess'
        }), 200 if overall_status else 503
    
    def check_cargo_test():
        import subprocess
        try:
            result = subprocess.run(['cargo', 'test', '--version'], capture_output=True, timeout=5)
            return result.returncode == 0
        except:
            return False
    
    if __name__ == '__main__':
        app.run(host='0.0.0.0', port=8080)
```

#### Kubernetes Health Probes Configuration

Update agent deployments with health probes:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rex-agent
  namespace: taskmaster
spec:
  template:
    spec:
      containers:
      - name: rex-agent
        image: taskmaster/rex-agent:latest
        ports:
        - containerPort: 8080
          name: http-health
        livenessProbe:
          httpGet:
            path: /health
            port: http-health
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: http-health
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        volumeMounts:
        - name: health-config
          mountPath: /app/health
      volumes:
      - name: health-config
        configMap:
          name: agent-health-config
```

#### Synthetic Health Check Workflows

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: synthetic-health-check
  namespace: taskmaster
spec:
  entrypoint: health-check-pipeline
  templates:
  - name: health-check-pipeline
    steps:
    - - name: test-workflow-creation
        template: test-workflow-creation
    - - name: test-agent-health
        template: test-agent-health
        withItems:
        - rex
        - cleo
        - tess
    - - name: test-resource-allocation
        template: test-resource-allocation
    - - name: report-health-status
        template: report-health-status
        arguments:
          parameters:
          - name: results
            value: "{{steps.test-agent-health.outputs.result}}"

  - name: test-workflow-creation
    container:
      image: argoproj/argocd:latest
      command: [sh, -c]
      args:
      - |
        echo "Testing workflow creation capability"
        kubectl apply -f - <<EOF
        apiVersion: argoproj.io/v1alpha1
        kind: Workflow
        metadata:
          generateName: health-test-
        spec:
          entrypoint: dummy-test
          templates:
          - name: dummy-test
            container:
              image: alpine:latest
              command: [echo, "health test successful"]
        EOF
        sleep 10
        kubectl delete workflow -l workflows.argoproj.io/workflow-template=health-test

  - name: test-agent-health
    inputs:
      parameters:
      - name: agent-type
    container:
      image: curlimages/curl:latest
      command: [sh, -c]
      args:
      - |
        AGENT_TYPE="{{inputs.parameters.agent-type}}"
        HEALTH_URL="http://${AGENT_TYPE}-agent:8080/health"
        
        echo "Testing $AGENT_TYPE agent health"
        
        RESPONSE=$(curl -s -w "%{http_code}" -o /tmp/response.json "$HEALTH_URL")
        HTTP_CODE=${RESPONSE: -3}
        
        if [ "$HTTP_CODE" = "200" ]; then
          echo "Agent $AGENT_TYPE: HEALTHY"
          echo "healthy" > /tmp/result
        else
          echo "Agent $AGENT_TYPE: UNHEALTHY (HTTP $HTTP_CODE)"
          cat /tmp/response.json
          echo "unhealthy" > /tmp/result
        fi
    outputs:
      parameters:
      - name: result
        valueFrom:
          path: /tmp/result
```

### Step 2: Stuck Workflow Detection System

#### Prometheus Rules for Stuck Detection

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: workflow-stuck-detection
  namespace: taskmaster
spec:
  groups:
  - name: workflow.stuck.detection
    interval: 1m
    rules:
    - alert: WorkflowStuckAtSameStage
      expr: |
        (
          time() - argo_workflow_status_phase_start_time{phase!="Succeeded",phase!="Failed",phase!="Error"}
        ) > 21600
      for: 1m
      labels:
        severity: warning
        component: workflow
      annotations:
        summary: "Workflow {{ $labels.name }} stuck at {{ $labels.phase }} for over 6 hours"
        description: "Workflow {{ $labels.name }} in namespace {{ $labels.namespace }} has been in phase {{ $labels.phase }} for {{ humanizeDuration $value }}. This may indicate a stuck workflow requiring intervention."

    - alert: WorkflowExcessiveAge
      expr: |
        (
          time() - argo_workflow_start_time{phase!="Succeeded",phase!="Failed",phase!="Error"}
        ) > 604800
      for: 5m
      labels:
        severity: warning
        component: workflow
      annotations:
        summary: "Workflow {{ $labels.name }} running for over 7 days"
        description: "Long-running workflow {{ $labels.name }} has been active for {{ humanizeDuration $value }}. Consider investigation for potential issues."

    - alert: WorkflowCriticalAge
      expr: |
        (
          time() - argo_workflow_start_time{phase!="Succeeded",phase!="Failed",phase!="Error"}
        ) > 864000
      for: 2m
      labels:
        severity: critical
        component: workflow
        escalate: "true"
      annotations:
        summary: "Critical: Workflow {{ $labels.name }} running for over 10 days"
        description: "Workflow {{ $labels.name }} has been running for {{ humanizeDuration $value }}. Immediate investigation required."

    - alert: MultipleWorkflowsStuck
      expr: |
        count by (namespace) (
          (time() - argo_workflow_status_phase_start_time{phase!="Succeeded",phase!="Failed",phase!="Error"}) > 21600
        ) >= 3
      for: 5m
      labels:
        severity: critical
        component: workflow-system
      annotations:
        summary: "Multiple workflows stuck in namespace {{ $labels.namespace }}"
        description: "{{ $value }} workflows are stuck in namespace {{ $labels.namespace }}. System-wide issue possible."
```

#### Stuck Reason Analysis Script

```python
#!/usr/bin/env python3
import kubernetes
import json
import re
from datetime import datetime, timedelta

class StuckWorkflowAnalyzer:
    def __init__(self):
        kubernetes.config.load_incluster_config()
        self.v1 = kubernetes.client.CoreV1Api()
        self.custom_api = kubernetes.client.CustomObjectsApi()
    
    def analyze_stuck_workflow(self, workflow_name, namespace):
        """Analyze why a workflow might be stuck"""
        try:
            # Get workflow object
            workflow = self.custom_api.get_namespaced_custom_object(
                group="argoproj.io",
                version="v1alpha1", 
                namespace=namespace,
                plural="workflows",
                name=workflow_name
            )
            
            analysis = {
                'workflow_name': workflow_name,
                'namespace': namespace,
                'analysis_time': datetime.utcnow().isoformat(),
                'stuck_reasons': [],
                'recommendations': []
            }
            
            # Check workflow status
            status = workflow.get('status', {})
            phase = status.get('phase', 'Unknown')
            
            # Analyze different stuck scenarios
            if phase == 'Running':
                self._analyze_running_stuck(workflow, analysis)
            elif phase == 'Pending':
                self._analyze_pending_stuck(workflow, analysis)
            elif phase in ['Failed', 'Error']:
                self._analyze_failed_stuck(workflow, analysis)
            
            # Check node statuses
            self._analyze_node_statuses(workflow, analysis)
            
            # Check resource constraints
            self._check_resource_constraints(workflow, analysis)
            
            return analysis
            
        except Exception as e:
            return {
                'workflow_name': workflow_name,
                'error': str(e),
                'analysis_time': datetime.utcnow().isoformat()
            }
    
    def _analyze_running_stuck(self, workflow, analysis):
        """Analyze stuck running workflows"""
        nodes = workflow.get('status', {}).get('nodes', {})
        
        running_nodes = [n for n in nodes.values() if n.get('phase') == 'Running']
        pending_nodes = [n for n in nodes.values() if n.get('phase') == 'Pending']
        
        if pending_nodes and not running_nodes:
            analysis['stuck_reasons'].append('No active running nodes despite Running phase')
            analysis['recommendations'].append('Check resource availability and scheduling constraints')
        
        # Check for long-running nodes
        current_time = datetime.utcnow()
        for node in running_nodes:
            start_time_str = node.get('startedAt')
            if start_time_str:
                start_time = datetime.fromisoformat(start_time_str.replace('Z', '+00:00'))
                runtime = current_time - start_time.replace(tzinfo=None)
                
                if runtime > timedelta(hours=12):
                    analysis['stuck_reasons'].append(f'Node {node.get("displayName")} running for {runtime}')
                    analysis['recommendations'].append(f'Investigate long-running node {node.get("displayName")}')
    
    def _analyze_pending_stuck(self, workflow, analysis):
        """Analyze stuck pending workflows"""
        analysis['stuck_reasons'].append('Workflow stuck in Pending phase')
        analysis['recommendations'].extend([
            'Check resource quotas and limits',
            'Verify node capacity and scheduling constraints',
            'Check for PVC binding issues'
        ])
    
    def _check_resource_constraints(self, workflow, analysis):
        """Check for resource-related constraints"""
        try:
            # Check PVCs
            pvcs = self.v1.list_namespaced_persistent_volume_claim(
                namespace=workflow['metadata']['namespace']
            )
            
            for pvc in pvcs.items:
                if pvc.status.phase == 'Pending':
                    analysis['stuck_reasons'].append(f'PVC {pvc.metadata.name} in Pending state')
                    analysis['recommendations'].append('Check storage class and available storage')
                    
        except Exception as e:
            analysis['stuck_reasons'].append(f'Could not check resources: {str(e)}')
```

### Step 3: Resource Consumption Tracking

#### Enhanced Metrics Collection

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-metrics-config
  namespace: taskmaster
data:
  custom-metrics.py: |
    #!/usr/bin/env python3
    import time
    import json
    import requests
    import psutil
    import kubernetes
    from prometheus_client import CollectorRegistry, Gauge, Counter, push_to_gateway
    
    class WorkflowMetricsCollector:
        def __init__(self):
            self.registry = CollectorRegistry()
            self.setup_metrics()
            
        def setup_metrics(self):
            # Resource metrics
            self.cpu_usage = Gauge('workflow_cpu_usage_percent', 'CPU usage percentage', 
                                 ['workflow', 'agent', 'namespace'], registry=self.registry)
            self.memory_usage = Gauge('workflow_memory_usage_bytes', 'Memory usage in bytes',
                                    ['workflow', 'agent', 'namespace'], registry=self.registry)
            self.disk_io = Gauge('workflow_disk_io_bytes', 'Disk I/O in bytes',
                               ['workflow', 'agent', 'namespace', 'direction'], registry=self.registry)
            
            # PVC metrics
            self.pvc_usage = Gauge('workflow_pvc_usage_percent', 'PVC usage percentage',
                                 ['pvc_name', 'namespace'], registry=self.registry)
            
            # GitHub API metrics
            self.github_rate_limit = Gauge('workflow_github_rate_limit_remaining', 'Remaining GitHub API calls',
                                         ['agent'], registry=self.registry)
            
            # Workflow age metrics
            self.workflow_age = Gauge('workflow_age_seconds', 'Workflow age in seconds',
                                    ['workflow', 'phase', 'namespace'], registry=self.registry)
        
        def collect_metrics(self):
            # Collect resource metrics
            self._collect_resource_metrics()
            
            # Collect PVC metrics  
            self._collect_pvc_metrics()
            
            # Collect GitHub API metrics
            self._collect_github_metrics()
            
            # Collect workflow age metrics
            self._collect_workflow_metrics()
            
            # Push to Prometheus
            push_to_gateway('pushgateway:9091', job='workflow-metrics', registry=self.registry)
        
        def _collect_resource_metrics(self):
            # Get pod metrics using Kubernetes metrics API
            kubernetes.config.load_incluster_config()
            custom_api = kubernetes.client.CustomObjectsApi()
            
            try:
                metrics = custom_api.list_namespaced_custom_object(
                    group="metrics.k8s.io",
                    version="v1beta1",
                    namespace="taskmaster", 
                    plural="pods"
                )
                
                for pod_metrics in metrics['items']:
                    pod_name = pod_metrics['metadata']['name']
                    
                    for container in pod_metrics['containers']:
                        # Extract agent type from pod name
                        agent = 'unknown'
                        if 'rex' in pod_name:
                            agent = 'rex'
                        elif 'cleo' in pod_name:
                            agent = 'cleo'
                        elif 'tess' in pod_name:
                            agent = 'tess'
                        
                        # Parse CPU usage (e.g., "250m" -> 0.25)
                        cpu_str = container['usage']['cpu']
                        cpu_cores = self._parse_cpu(cpu_str)
                        cpu_percent = cpu_cores * 100  # Convert to percentage
                        
                        # Parse memory usage (e.g., "128Mi" -> bytes)
                        memory_str = container['usage']['memory']
                        memory_bytes = self._parse_memory(memory_str)
                        
                        self.cpu_usage.labels(
                            workflow=pod_name,
                            agent=agent,
                            namespace='taskmaster'
                        ).set(cpu_percent)
                        
                        self.memory_usage.labels(
                            workflow=pod_name,
                            agent=agent, 
                            namespace='taskmaster'
                        ).set(memory_bytes)
                        
            except Exception as e:
                print(f"Error collecting resource metrics: {e}")
        
        def _parse_cpu(self, cpu_str):
            """Parse CPU string like '250m' to cores"""
            if cpu_str.endswith('m'):
                return float(cpu_str[:-1]) / 1000
            return float(cpu_str)
        
        def _parse_memory(self, memory_str):
            """Parse memory string like '128Mi' to bytes"""
            units = {'Ki': 1024, 'Mi': 1024**2, 'Gi': 1024**3}
            for unit, multiplier in units.items():
                if memory_str.endswith(unit):
                    return float(memory_str[:-len(unit)]) * multiplier
            return float(memory_str)

---
apiVersion: batch/v1
kind: CronJob
metadata:
  name: workflow-metrics-collector
  namespace: taskmaster
spec:
  schedule: "*/1 * * * *"  # Every minute
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: metrics-collector
            image: python:3.9-slim
            command: ["python3", "/scripts/custom-metrics.py"]
            volumeMounts:
            - name: metrics-config
              mountPath: /scripts
          restartPolicy: OnFailure
          volumes:
          - name: metrics-config
            configMap:
              name: workflow-metrics-config
```

### Step 4: Automated Cleanup System

#### Workflow Cleanup CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: workflow-cleanup
  namespace: taskmaster
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: workflow-cleanup-sa
          containers:
          - name: cleanup
            image: python:3.9-slim
            command: ["python3", "/scripts/workflow-cleanup.py"]
            volumeMounts:
            - name: cleanup-script
              mountPath: /scripts
            - name: s3-config
              mountPath: /etc/s3
            env:
            - name: RETENTION_POLICIES
              valueFrom:
                configMapKeyRef:
                  name: cleanup-config
                  key: retention-policies.json
          restartPolicy: OnFailure
          volumes:
          - name: cleanup-script
            configMap:
              name: workflow-cleanup-config
          - name: s3-config
            secret:
              secretName: s3-credentials
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-cleanup-config
  namespace: taskmaster
data:
  workflow-cleanup.py: |
    #!/usr/bin/env python3
    import os
    import json
    import boto3
    import kubernetes
    from datetime import datetime, timedelta
    import gzip
    import base64
    
    class WorkflowCleanupManager:
        def __init__(self):
            kubernetes.config.load_incluster_config()
            self.custom_api = kubernetes.client.CustomObjectsApi()
            self.v1 = kubernetes.client.CoreV1Api()
            
            # Initialize S3 client
            self.s3_client = boto3.client(
                's3',
                aws_access_key_id=os.getenv('AWS_ACCESS_KEY_ID'),
                aws_secret_access_key=os.getenv('AWS_SECRET_ACCESS_KEY'),
                endpoint_url=os.getenv('S3_ENDPOINT', 'https://s3.amazonaws.com')
            )
            
            # Load retention policies
            self.policies = json.loads(os.getenv('RETENTION_POLICIES', '{}'))
        
        def run_cleanup(self):
            """Execute cleanup based on retention policies"""
            workflows = self.get_all_workflows()
            
            for workflow in workflows:
                self.process_workflow(workflow)
        
        def get_all_workflows(self):
            """Get all workflows across all namespaces"""
            try:
                workflows = self.custom_api.list_cluster_custom_object(
                    group="argoproj.io",
                    version="v1alpha1",
                    plural="workflows"
                )
                return workflows.get('items', [])
            except Exception as e:
                print(f"Error fetching workflows: {e}")
                return []
        
        def process_workflow(self, workflow):
            """Process individual workflow for cleanup"""
            name = workflow['metadata']['name']
            namespace = workflow['metadata']['namespace']
            phase = workflow.get('status', {}).get('phase', 'Unknown')
            
            # Skip if workflow has cleanup exemption
            if self.has_cleanup_exemption(workflow):
                print(f"Skipping {name}: cleanup exemption")
                return
            
            # Determine age and cleanup eligibility
            age_days = self.calculate_workflow_age(workflow)
            policy = self.get_retention_policy(phase)
            
            if age_days >= policy['retention_days']:
                print(f"Cleaning up {name} (age: {age_days} days, policy: {policy['retention_days']} days)")
                
                # Archive before deletion
                if policy.get('archive', True):
                    self.archive_workflow(workflow)
                
                # Delete workflow
                self.delete_workflow(name, namespace)
                
                # Cleanup associated PVCs
                self.cleanup_workflow_pvcs(workflow)
        
        def has_cleanup_exemption(self, workflow):
            """Check if workflow has cleanup exemption annotation"""
            annotations = workflow.get('metadata', {}).get('annotations', {})
            return annotations.get('taskmaster.io/cleanup-exempt', 'false').lower() == 'true'
        
        def calculate_workflow_age(self, workflow):
            """Calculate workflow age in days"""
            created_time_str = workflow['metadata']['creationTimestamp']
            created_time = datetime.fromisoformat(created_time_str.replace('Z', '+00:00'))
            age = datetime.now(created_time.tzinfo) - created_time
            return age.days
        
        def get_retention_policy(self, phase):
            """Get retention policy based on workflow phase"""
            default_policies = {
                'Succeeded': {'retention_days': 7, 'archive': True},
                'Failed': {'retention_days': 3, 'archive': True}, 
                'Error': {'retention_days': 3, 'archive': True},
                'Running': {'retention_days': 14, 'archive': False}
            }
            
            return self.policies.get(phase, default_policies.get(phase, default_policies['Running']))
        
        def archive_workflow(self, workflow):
            """Archive workflow to S3 before deletion"""
            try:
                name = workflow['metadata']['name']
                namespace = workflow['metadata']['namespace'] 
                timestamp = datetime.utcnow().strftime('%Y%m%d_%H%M%S')
                
                # Compress workflow definition
                workflow_json = json.dumps(workflow, indent=2)
                compressed_data = gzip.compress(workflow_json.encode('utf-8'))
                
                # Upload to S3
                s3_key = f"workflow-archives/{namespace}/{name}/{timestamp}.json.gz"
                
                self.s3_client.put_object(
                    Bucket=os.getenv('ARCHIVE_BUCKET', 'taskmaster-workflow-archives'),
                    Key=s3_key,
                    Body=compressed_data,
                    ContentType='application/gzip',
                    Metadata={
                        'workflow-name': name,
                        'namespace': namespace,
                        'archive-date': timestamp,
                        'phase': workflow.get('status', {}).get('phase', 'Unknown')
                    }
                )
                
                print(f"Archived {name} to s3://{os.getenv('ARCHIVE_BUCKET')}/{s3_key}")
                
            except Exception as e:
                print(f"Error archiving workflow {name}: {e}")
        
        def delete_workflow(self, name, namespace):
            """Delete workflow from cluster"""
            try:
                self.custom_api.delete_namespaced_custom_object(
                    group="argoproj.io",
                    version="v1alpha1",
                    namespace=namespace,
                    plural="workflows",
                    name=name
                )
                print(f"Deleted workflow {name} from namespace {namespace}")
                
            except Exception as e:
                print(f"Error deleting workflow {name}: {e}")
    
    if __name__ == "__main__":
        cleanup_manager = WorkflowCleanupManager()
        cleanup_manager.run_cleanup()

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: cleanup-config
  namespace: taskmaster
data:
  retention-policies.json: |
    {
      "Succeeded": {"retention_days": 7, "archive": true},
      "Failed": {"retention_days": 3, "archive": true},
      "Error": {"retention_days": 3, "archive": true},
      "Running": {"retention_days": 14, "archive": false}
    }
```

### Step 5: Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "title": "Long-Running Workflow Monitoring",
    "tags": ["taskmaster", "workflows", "monitoring"],
    "panels": [
      {
        "title": "Workflow Age Distribution",
        "type": "histogram",
        "targets": [
          {
            "expr": "argo_workflow_age_seconds / 86400",
            "legendFormat": "Workflow Age (days)"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "title": "Stuck Workflows by Phase",
        "type": "stat",
        "targets": [
          {
            "expr": "count by (phase) (argo_workflow_status_phase{phase!=\"Succeeded\",phase!=\"Failed\"})",
            "legendFormat": "{{phase}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "title": "Resource Usage by Agent",
        "type": "graph",
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
      },
      {
        "title": "PVC Usage Alerts",
        "type": "table",
        "targets": [
          {
            "expr": "workflow_pvc_usage_percent > 80",
            "format": "table"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16}
      },
      {
        "title": "GitHub API Rate Limits",
        "type": "gauge",
        "targets": [
          {
            "expr": "workflow_github_rate_limit_remaining",
            "legendFormat": "{{agent}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16}
      }
    ],
    "time": {
      "from": "now-24h",
      "to": "now"
    },
    "refresh": "1m"
  }
}
```

## Integration Points

- **Argo Workflows**: Custom metrics and health check integration
- **Prometheus**: Metrics collection and alerting rules
- **Grafana**: Visualization dashboards and operational insights
- **AlertManager**: Notification routing and escalation policies
- **PagerDuty/Slack**: Incident notification and response
- **AWS S3/MinIO**: Workflow archival and compliance storage
- **Kubernetes**: Resource monitoring and cleanup operations