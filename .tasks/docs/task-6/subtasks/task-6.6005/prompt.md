Implement subtask 6005: Provision Grafana dashboard JSON files for Hermes observability

## Objective
Create four Grafana dashboard JSON files in `dashboards/hermes/` with LogQL queries against Loki for rollout health, deliberation pipeline metrics, migration progress, and rollback triggers. Configure for sidecar ConfigMap provisioning.

## Steps
Step-by-step:
1. Create `dashboards/hermes/` directory.
2. **Rollout Health Dashboard** (`rollout-health.json`):
   - Panel 1: Stat panel showing current `rollout_phase` — query: `{app="hermes"} | json | operation="rollout_phase_initialized" | line_format "{{.rollout_phase}}"` (latest).
   - Panel 2: Time series of error rate by rollout phase — query: `sum(count_over_time({app="hermes"} | json | error_code!="" [$__interval])) by (rollout_phase)`.
   - Panel 3: Log volume by phase — `sum(count_over_time({app="hermes"} | json [$__interval])) by (rollout_phase)`.
3. **Deliberation Pipeline Dashboard** (`deliberation-pipeline.json`):
   - Panel 1: Throughput — `count_over_time({app="hermes"} | json | operation=~"deliberation.*" [$__interval])`.
   - Panel 2: Latency percentiles — `quantile_over_time(0.50, {app="hermes"} | json | operation=~"deliberation.*" | unwrap duration_ms [$__interval])` (repeat for P95, P99).
   - Panel 3: Failure rate — ratio of error entries to total deliberation entries.
4. **Migration Progress Dashboard** (`migration-progress.json`):
   - Panel 1: Latest migration progress stats — query on `migration_step` entries.
   - Panel 2: MinIO health over time — `{app="hermes"} | json | minio_health!=""` with value mapping.
   - Panel 3: Artifact copy throughput — rate of `migration_step="copy"` entries.
5. **Rollback Triggers Dashboard** (`rollback-triggers.json`):
   - Panel 1: Table of rollback trigger events — `{app="hermes"} | json | rollback_trigger="true"`.
   - Panel 2: Time series count of triggers over time.
6. For each dashboard JSON: set `"editable": true`, appropriate `uid` and `title`, Loki datasource reference.
7. Create a `dashboards/hermes/configmap.yaml` Kubernetes ConfigMap with the `grafana_dashboard: "1"` label for sidecar auto-provisioning.

## Validation
Validate each dashboard JSON is syntactically valid (parse with `JSON.parse`). Import each dashboard into a Grafana instance via the API — verify no import errors. With the Hermes service running and emitting logs, verify each dashboard loads without 'No data' errors and at least one panel shows data points. Verify the ConfigMap YAML is valid Kubernetes manifest and includes the sidecar label.