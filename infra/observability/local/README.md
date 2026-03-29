# Local Observability Stack

Grafana + Loki + Promtail stack for monitoring the CTO intake pipeline locally.

Collects JSONL log files from `.intake/logs/` and makes them queryable and visualizable in Grafana.

## Prerequisites

- Docker & Docker Compose

## Usage

**Start:**

```bash
intake/scripts/observability-up.sh
# or
docker compose -f infra/observability/local/docker-compose.yml up -d
```

**Stop:**

```bash
intake/scripts/observability-down.sh
# or
docker compose -f infra/observability/local/docker-compose.yml down
```

To remove persistent data volumes:

```bash
docker compose -f infra/observability/local/docker-compose.yml down -v
```

## Access

- **Grafana:** [http://localhost:3001](http://localhost:3001) — no login required (anonymous auth enabled)
- **Loki API:** [http://localhost:3100](http://localhost:3100)

## Log Files

The intake pipeline writes JSONL logs to `.intake/logs/`:

| File | Description |
|------|-------------|
| `llm-calls.jsonl` | LLM API call traces (provider, model, latency, tokens) |
| `pipeline.jsonl` | Pipeline step execution events |
| `runs.jsonl` | Top-level intake run lifecycle events |

## Dashboards

Drop Grafana dashboard JSON files into `dashboards/` — they are auto-loaded by Grafana's provisioning system. Changes are picked up on Grafana restart.

## Architecture

```
.intake/logs/*.jsonl  →  Promtail  →  Loki  →  Grafana
```

Promtail tails the JSONL files, parses JSON fields into Loki labels, and pushes to Loki. Grafana queries Loki via LogQL. Data is retained for 7 days.
