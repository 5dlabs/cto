# Repository Guidelines

## Context7 for Rust Best Practices

Before implementing significant Rust code, use Context7 to get current documentation.

### Two-Step Workflow

1. **Resolve library:** `resolve_library_id({ libraryName: "tokio rust" })`
2. **Get docs:** `get_library_docs({ context7CompatibleLibraryID: "/websites/rs_tokio_tokio", topic: "error handling async" })`

### Key Rust Library IDs

- **Tokio:** `/websites/rs_tokio_tokio` (async runtime, 93.8 score)
- **Serde:** `/websites/serde_rs` (serialization)
- **Clippy:** `/rust-lang/rust-clippy` (lints)
- **Anyhow:** `/dtolnay/anyhow` (app errors, 89.3 score)
- **Thiserror:** `/dtolnay/thiserror` (custom errors, 83.1 score)
- **Tracing:** `/tokio-rs/tracing` (logging)

### When to Query

Always consult Context7 when:
- Setting up async code with Tokio
- Implementing error handling (anyhow context patterns, thiserror enums)
- Using serde attributes or custom serialization
- Configuring Clippy pedantic lints
- Writing HTTP handlers or database queries

### Best Practices

- **Resolve first** - Always resolve library names to get current IDs
- **Be specific** - Query focused topics: "error handling context" not "documentation"
- **High scores win** - Prefer libraries with higher benchmark scores
- **Single topic** - One focused topic per query for best results

## Project Structure & Module Organization
- `mcp/` — Rust MCP server (`cto-mcp`).
- `controller/` — Rust controllers and binaries (e.g., `agent-controller`).
- `sidecar/` — Rust sidecar utilities.
- `infra/` — Helm charts, GitOps manifests, images, and scripts.
- `scripts/` — Bash helpers and validation utilities.
- `docs/` — Architecture, examples, and references.

## Build, Test, and Development Commands
- Build MCP server: `cd mcp && cargo build --release`.
- Build controller: `cd controller && cargo build --release --bin agent-controller`.
- Run tests (Rust): `cargo test -p cto-mcp` and `cargo test -p controller`.
- Lint/format (Rust): `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`.
- GitOps validation: `make -C infra/gitops validate` (or `lint`, `test`).
- Pre-commit checks: `pre-commit install && pre-commit run --all-files`.

## Coding Style & Naming Conventions
- Rust: rustfmt (Edition 2021, `max_width=100`); prefer `tracing::*` over `println!` (enforced by Clippy). Binary names kebab-case (e.g., `agent-controller`); files/modules snake_case (e.g., `src/bin/agent_controller.rs`).
- YAML: 2-space indent; begin docs with `---`; follow `.yamllint.yaml`.
- Markdown: follow `.markdownlint.yaml` (incremental headings, no trailing spaces).
- Shell: keep `bash -euo pipefail`; validate with ShellCheck where applicable.

## Testing Guidelines
- Unit tests live alongside code (`mod tests { ... }`) and in Rust integration tests when needed. Run `cargo test` in crate roots.
- For controllers and workflows, add small, deterministic tests; prefer fixtures under `controller/src/**/tests/`.
- Validate infrastructure changes with `make -C infra/gitops validate` and attach output to PRs when material.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `refactor:`, etc. Keep commits focused and descriptive.
- Before pushing: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `pre-commit run --all-files`.
- PRs must include: summary, rationale, scope of impact, and verification steps; link issues (e.g., `Closes #123`). Include logs/screenshots for infra or CLI-facing changes.

## Security & Configuration Tips
- Never commit secrets. Use Kubernetes secrets/Helm values under `infra/secret-store/` and local env vars.
- Use `cto-config.json` (see `cto-config-example.json`) and `.cursor/mcp.json` to configure local runs; avoid committing user-specific tokens.
- Large files and generated artifacts should not be committed unless explicitly required.

## MCP Tools for Observability & GitOps

The platform provides MCP (Model Context Protocol) tools for interacting with observability and GitOps systems. These tools require port-forwards to be active.

### Port-Forward Requirements

```bash
# Prometheus (metrics)
kubectl port-forward svc/prometheus-server -n observability 9090:80

# Loki (logs)
kubectl port-forward svc/loki-gateway -n observability 3100:80

# Grafana (dashboards)
kubectl port-forward svc/grafana -n observability 3000:80

# ArgoCD (GitOps)
kubectl port-forward svc/argocd-server -n argocd 8080:80

# Argo Workflows
kubectl port-forward svc/argo-workflows-server -n automation 2746:2746
```

### Prometheus (Metrics)

Query metrics for performance analysis:

```bash
# Check service health
prometheus_execute_query({ query: "up{job=\"cto-controller\"}" })

# Error rates
prometheus_execute_query({ query: "rate(http_requests_total{status=~\"5..\"}[5m])" })

# Range queries for trends
prometheus_execute_range_query({
  query: "rate(http_requests_total[5m])",
  start: "now-1h", end: "now", step: "1m"
})

# List available metrics
prometheus_list_metrics({ filter_pattern: "cto_" })
```

### Loki (Logs)

Search and analyze application logs:

```bash
# Query logs with filters
loki_query({
  query: "{namespace=\"cto\"} |~ \"error|ERROR\"",
  limit: 100
})

# Get available labels
loki_label_names()
loki_label_values({ label: "pod" })
```

**LogQL Patterns:**
- `{namespace="cto"}` - Label selector
- `|~ "pattern"` - Regex match
- `|= "exact"` - Exact match
- `| json | level="error"` - Parse JSON and filter

### Grafana (Dashboards & Alerts)

Access dashboards, alerts, and query datasources:

```bash
# Search dashboards
grafana_search_dashboards({ query: "CTO" })

# Query metrics via Grafana
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "up",
  queryType: "instant"
})

# Query logs via Grafana
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"}",
  limit: 50
})

# List alert rules
grafana_list_alert_rules()
```

### ArgoCD (GitOps)

Manage GitOps deployments:

```bash
# List applications
argocd_list_applications()

# Get application status (sync, health)
argocd_get_application({ applicationName: "cto-controller" })

# Check resources and events
argocd_get_application_resource_tree({ applicationName: "app" })
argocd_get_application_events({ applicationName: "app" })

# Trigger sync
argocd_sync_application({ applicationName: "app" })
```

### Argo Workflows

Monitor and manage workflow executions:

```bash
# List workflows
argo_workflows_list_workflows({ namespace: "cto", limit: 20 })

# Get workflow details and logs
argo_workflows_get_workflow({ namespace: "cto", name: "workflow-name" })
argo_workflows_get_workflow_logs({ namespace: "cto", workflow_name: "name" })

# List templates and cron workflows
argo_workflows_list_workflow_templates({ namespace: "cto" })
argo_workflows_list_cron_workflows({ namespace: "automation" })

# Retry failed workflow
argo_workflows_retry_workflow({ namespace: "cto", name: "failed-workflow" })
```

### Debugging Workflow

When investigating issues:

1. **Metrics** - `prometheus_execute_query({ query: "up{job=\"...\"}" })`
2. **Logs** - `loki_query({ query: "{pod=~\"...\"}|~\"error\"" })`
3. **Deployment** - `argocd_get_application({ applicationName: "..." })`
4. **Workflows** - `argo_workflows_list_workflows({ status: "Failed" })`

