Implement subtask 9008: Deploy Loki for centralized log aggregation

## Objective
Deploy Grafana Loki and Promtail (or similar log shipper) to aggregate logs from all pods in the cluster. Configure log retention and label-based querying.

## Steps
1. Deploy Loki using the Helm chart (`grafana/loki` or `grafana/loki-stack`).
2. Deploy Promtail as a DaemonSet to ship container logs from all nodes to Loki.
3. Configure Promtail to add labels: namespace, pod name, container name, app label.
4. Configure Loki storage backend (filesystem with PVC for single-node, or S3-compatible for scalability).
5. Set log retention period to 14 days.
6. Configure Loki resource limits appropriate for the expected log volume.
7. Verify Loki is accessible as a data source endpoint for Grafana (ClusterIP service).

## Validation
Verify Loki and Promtail pods are running on all nodes. Query Loki via the API (`/loki/api/v1/query`) for recent logs from a known pod and confirm logs are returned. Verify logs from all namespaces are being collected. Check that label-based filtering works (e.g., filter by `{namespace="default"}`).