# Agent-Control Local Spike

Date: 2026-03-11

## Objective

Validate whether `agent-control` can be introduced as a local-only guardrail
layer without changing CTO runtime behavior.

## Environment

- Host: macOS (local workstation)
- Deployment mode: Docker Compose from upstream repository
- Endpoint: `http://localhost:8000`

## Commands

Start stack:

```bash
curl -L https://raw.githubusercontent.com/agentcontrol/agent-control/refs/heads/main/docker-compose.yml | docker compose -f - up -d
```

Health check:

```bash
curl http://localhost:8000/health
```

## Measurements

### 1) Health Endpoint Latency

20 samples against `/health`:

- p50: 1.86 ms
- avg: 2.41 ms
- max: 13.41 ms

### 2) Control Evaluation Latency + Accuracy

Setup:

- Registered test agent (`cto-market-sync-<timestamp>`)
- Added deny control with regex evaluator for SSN pattern
- Bound control directly to test agent
- Evaluated 20 total post-stage LLM outputs:
  - 10 benign outputs (expected safe)
  - 10 SSN-containing outputs (expected blocked)

Result:

- p50: 103.21 ms
- p95: 261.31 ms
- max: 485.65 ms
- false positive rate: 0.0
- false negative rate: 0.0

## Notes

- An initial run used an over-escaped regex and produced false negatives; after
  correcting the pattern to `\\b\\d{3}-\\d{2}-\\d{4}\\b`, detection matched
  expectations.
- We observed intermittent Docker CLI instability on this machine (occasional
  API/socket errors while inspecting/stopping containers). The guardrail path
  itself remained testable after stack restart.

## Recommendation

- Keep `agent-control` in local/dev validation only for now.
- Proceed with Phase 1 `mmmodels` catalog integration as primary rollout.
- For Phase 2 consideration, repeat this spike in non-prod cluster with:
  - 2-3 real CTO prompts/tools,
  - control set covering PII + prompt-injection + policy steer,
  - latency SLO guardrail and false-positive acceptance threshold.
