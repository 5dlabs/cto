# Viewing CLI Container Logs in Grafana

This guide explains how to view logs from all CLI containers (agents) in Grafana using Victoria Logs.

## Access Grafana

1. Port-forward Grafana to your local machine:
   ```bash
   kubectl port-forward -n telemetry svc/grafana 3000:80
   ```

2. Open http://localhost:3000 in your browser

3. Login with credentials:
   - Username: `admin`
   - Password: `admin` (or check the Grafana secret)

## Using the Victoria Logs Data Source

The Victoria Logs datasource should be automatically configured. If not, add it manually:

1. Go to Configuration → Data Sources
2. Add data source → Prometheus
3. Configure:
   - **Name**: Victoria Logs
   - **URL**: `http://victoria-logs-victoria-logs-single-server.telemetry.svc.cluster.local:9428/select/logsql`
   - **HTTP Method**: POST

## Querying CLI Container Logs

### LogsQL Query Examples

Victoria Logs uses LogsQL for querying. Here are examples for viewing CLI container logs:

#### All logs from agent-platform namespace:
```logsql
{kubernetes.namespace_name="agent-platform"}
```

#### Logs from specific CLI containers:
```logsql
{kubernetes.namespace_name="agent-platform", kubernetes.container_name=~".*-cli"}
```

#### Logs from specific agents (Cleo, Atlas, Cipher, etc.):
```logsql
{kubernetes.pod_name=~"cleo.*"}
{kubernetes.pod_name=~"atlas.*"}
{kubernetes.pod_name=~"cipher.*"}
{kubernetes.pod_name=~"tess.*"}
{kubernetes.pod_name=~"rex.*"}
```

#### Filter by log level:
```logsql
{kubernetes.namespace_name="agent-platform"} | json | level="error"
```

#### Search for specific text in logs:
```logsql
{kubernetes.namespace_name="agent-platform"} | "error" or "failed"
```

#### Logs from a specific time range with text search:
```logsql
{kubernetes.namespace_name="agent-platform", _time>"2025-11-24T00:00:00Z"} | "task"
```

## Available Log Fields

All logs include the following Kubernetes metadata fields:
- `kubernetes.namespace_name` - Kubernetes namespace
- `kubernetes.pod_name` - Pod name
- `kubernetes.container_name` - Container name
- `kubernetes.container_image` - Container image
- `kubernetes.host` - Node name
- `kubernetes.labels.*` - Pod labels
- `cluster` - Cluster name (currently: telemetry-dev)
- `message` - The actual log message
- `stream` - stdout or stderr
- `_time` - Log timestamp

## Creating a Dashboard

1. In Grafana, go to Dashboards → New Dashboard
2. Add a new panel
3. Select "Victoria Logs" as the data source
4. Enter a LogsQL query (e.g., `{kubernetes.namespace_name="agent-platform"}`)
5. Choose visualization type (usually "Logs")
6. Configure panel settings:
   - Time range
   - Refresh interval
   - Display options

### Pre-built Dashboard Template

```json
{
  "title": "CLI Container Logs",
  "panels": [
    {
      "title": "All Agent Platform Logs",
      "targets": [
        {
          "expr": "{kubernetes.namespace_name=\"agent-platform\"}"
        }
      ]
    },
    {
      "title": "Error Logs",
      "targets": [
        {
          "expr": "{kubernetes.namespace_name=\"agent-platform\"} | \"error\" or \"ERROR\" or \"failed\""
        }
      ]
    }
  ]
}
```

## Troubleshooting

### No logs appearing

1. Check that Fluent-Bit is running:
   ```bash
   kubectl get pods -n telemetry -l app.kubernetes.io/name=fluent-bit
   ```

2. Verify Fluent-Bit is sending logs to Victoria Logs:
   ```bash
   kubectl logs -n telemetry -l app.kubernetes.io/name=fluent-bit --tail=50 | grep victoria
   ```

3. Query Victoria Logs directly:
   ```bash
   kubectl port-forward -n telemetry victoria-logs-victoria-logs-single-server-0 9428:9428
   curl 'http://localhost:9428/select/logsql/query' -d 'query={kubernetes.namespace_name=~".*"}' -d 'limit=10'
   ```

### Data source connection error

1. Verify Victoria Logs service is accessible:
   ```bash
   kubectl get svc -n telemetry victoria-logs-victoria-logs-single-server
   ```

2. Test connectivity from Grafana pod:
   ```bash
   kubectl exec -n telemetry deployment/grafana -- wget -qO- http://victoria-logs-victoria-logs-single-server.telemetry.svc.cluster.local:9428/
   ```

## Log Retention

- Current retention: 31 days
- Logs are stored in a persistent volume
- Storage path: `/storage` in the Victoria Logs pod

## Performance Tips

1. Always include a namespace filter to reduce query scope
2. Use time range filters for faster queries  
3. Limit results with `| limit N` for initial exploration
4. Use field filters before text searches for better performance

## Links

- [Victoria Logs Documentation](https://docs.victoriametrics.com/victorialogs/)
- [LogsQL Query Language](https://docs.victoriametrics.com/victorialogs/logsql/)
