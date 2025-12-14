# Monitoring Stack

This directory contains the configuration for the monitoring and observability stack.

## Components

### Log Ingestion Pipeline

**Fluent-Bit** → **OTEL Collector** → **Victoria Logs** → **Grafana**

1. **Fluent-Bit** (DaemonSet)
   - Collects logs from all containers in the cluster
   - Parses Kubernetes metadata
   - Sends logs to OTEL Collector via OTLP HTTP

2. **OpenTelemetry Collector** (Deployment)
   - Receives logs from Fluent-Bit
   - Processes and enriches logs with additional metadata
   - Forwards logs to Victoria Logs
   - Also handles metrics and traces

3. **Victoria Logs** (StatefulSet)
   - High-performance log storage
   - Retention: 31 days
   - Query endpoint: `http://victoria-logs-victoria-logs-single-server.observability.svc.cluster.local:9428`
   - Storage: Local-path PVC

4. **Grafana** (Deployment)
   - Log visualization and dashboards
   - Data source: Victoria Logs
   - Access: Port 80 (ClusterIP)

## Configuration Files

- `otel-collector/values.yaml` - OTEL Collector Helm values with Victoria Logs integration
- `grafana/datasources.yaml` - Grafana data source configuration for Victoria Logs

## Applying Changes

Update the OTEL Collector ArgoCD application:
```bash
kubectl patch application otel-collector -n argocd --type=json \
  -p='[{"op": "replace", "path": "/spec/source/helm/values", "value": "'"$(cat otel-collector/values.yaml)"'"}]'
```

## Viewing Logs in Grafana

1. Access Grafana (port-forward if needed):
   ```bash
   kubectl port-forward -n telemetry svc/grafana 3000:80
   ```

2. Login with admin credentials

3. Add Victoria Logs data source (if not already configured):
   - Type: Prometheus
   - URL: `http://victoria-logs-victoria-logs-single-server.observability.svc.cluster.local:9428/select/logsql/query`

4. Query logs using LogsQL:
   ```
   {namespace="cto"} | json
   ```

## Testing Log Flow

Check if logs are flowing through the pipeline:

```bash
# Check Fluent-Bit is collecting logs
kubectl logs -n telemetry -l app.kubernetes.io/name=fluent-bit --tail=50

# Check OTEL Collector is forwarding logs
kubectl logs -n telemetry -l app.kubernetes.io/name=opentelemetry-collector --tail=50

# Query Victoria Logs directly
kubectl port-forward -n telemetry victoria-logs-victoria-logs-single-server-0 9428:9428
curl 'http://localhost:9428/select/logsql/query' -d 'query={namespace=~".*"}' -d 'limit=10'
```

## CLI Container Logs

To view logs from CLI containers (agents) in Grafana, use queries like:

```
{namespace="cto", container=~".*-cli"} | json
{pod=~".*cleo.*"} | json
{pod=~".*atlas.*"} | json
{pod=~".*cipher.*"} | json
```

## Troubleshooting

### Logs not appearing in Victoria Logs

1. Check Fluent-Bit is running: `kubectl get pods -n telemetry -l app.kubernetes.io/name=fluent-bit`
2. Check OTEL Collector logs for errors
3. Verify Victoria Logs is healthy: `kubectl get pods -n telemetry -l app=server`
4. Test direct ingestion to Victoria Logs

### Grafana can't connect to Victoria Logs

1. Verify service endpoints: `kubectl get svc -n telemetry`
2. Check network policies
3. Verify data source configuration in Grafana
