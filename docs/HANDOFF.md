# Handoff: ACP Testing Branch — 2026-04-07

## Branch: `acp-testing`

## What Just Happened

We just rebuilt and pushed `ghcr.io/5dlabs/controller:latest` with all the changes below. The controller is currently **scaled to 0** on the cluster — ready to scale up and test.

## Images Ready to Deploy

| Image | Tag | Status |
|-------|-----|--------|
| `ghcr.io/5dlabs/controller:latest` | `sha256:1826ce0f9dd8...` | **Just pushed** — has unified ACP dispatch, telemetry, promtail sidecar, discord-agent-bots mount |
| `ghcr.io/5dlabs/agents:latest` | Built 2026-04-07 | Has acpx 0.5.2 (shell:true patch), droid 0.95.0, codex 0.118.0, claude 2.1.92, gemini 0.36.0, cursor-agent |

## Commits on `acp-testing` (chronological, oldest first)

1. **Clippy fixes** — pedantic lints across config, controller, acp-runtime, scm crates
2. **`fix: patch acpx terminal/create`** — `shell:true` for Linux ENOENT fix
3. **`fix: cursor-agent wrapper`** — correct node path and agent command in Dockerfile
4. **`fix(controller): serde + retry race`** — `#[serde(rename_all = "camelCase")]` on `CodeRunStatus` (was causing all status fields to deserialize as `None` → runaway job creation). Also fixed dedup retry check.
5. **`feat: mount discord-agent-bots`** — envFrom mount so CodeRun pods have per-agent Discord bot tokens
6. **`refactor: unify all CLIs through ACP dispatch`** — ALL `CLIType` variants now route through `openclaw.sh.hbs` instead of per-CLI templates. 8 legacy templates archived to `templates/clis/_archived/`.
7. **`feat: pluginToolsMcpBridge`** — enabled in `.acpxrc.json`, added agent command entries for claude/cursor/gemini
8. **`feat: telemetry + promtail sidecar`** — OTEL env vars in template, promtail sidecar container in pod spec, Datadog scaffolding

## Cluster State

- **cto-controller**: scaled to 0 replicas
- **CodeRuns**: 1 old one exists (can be cleaned up)
- **discord-agent-bots** secret: has tokens for Angie, Atlas, Cipher, Cleo, Rex, Spark, Stitch, Tap, Tess
- **Loki**: running at `openclaw-observability-loki-gateway.openclaw.svc.cluster.local`
- **OTel Collector**: running at `otel-collector-opentelemetry-collector.observability.svc.cluster.local:4317`

## What To Test

### Scale up and create a test CodeRun:

```bash
# Scale controller back up
kubectl scale deployment cto-controller -n cto --replicas=1

# Wait for it to be ready
kubectl rollout status deployment/cto-controller -n cto

# Create a test CodeRun (example: Rex on test-sandbox)
cat <<'EOF' | kubectl apply -f -
apiVersion: cto.5dlabs.ai/v1alpha1
kind: CodeRun
metadata:
  name: test-rex-sandbox-006
  namespace: cto
spec:
  agent: rex
  githubApp: 5DLabs-Rex
  repository: https://github.com/5dlabs/test-sandbox
  branch: main
  cliType: factory
  model: claude-opus-4-6
  prompt: "Create a simple hello world script at scripts/hello.py and open a PR"
  env:
    FACTORY_API_KEY: "fk-TUWFrDboAFn7GWtfAfk1-xvw_kvVdmdy5SeYPHRSVJor3HXFYf8kcMN5yI1X6CKg"
EOF
```

### Verify these things in the pod:

1. **ACP dispatch**: Logs should show "ACP AGENT DISPATCH (acpx)" NOT "FACTORY (DROID) CLI INVOCATION"
2. **Promtail sidecar**: `kubectl get pod <pod> -n cto -o jsonpath='{.spec.containers[*].name}'` should include `promtail`
3. **OTEL env vars**: Check the agent container has `OTEL_METRICS_EXPORTER`, `OTEL_EXPORTER_OTLP_ENDPOINT`
4. **Logs in Loki**: Query Grafana for `{namespace="cto", container="promtail"}` or `{cli_name=~".+"}`

## Key Files

| File | What it does |
|------|-------------|
| `templates/clis/openclaw.sh.hbs` | **THE** unified ACP dispatch template — CLI type mapping, auth, .acpxrc.json, telemetry, acpx invocation |
| `crates/controller/src/tasks/code/resources.rs` | Job/Pod spec builder — promtail sidecar, discord-agent-bots mount, volumes |
| `crates/controller/src/tasks/code/templates.rs` | Template rendering engine — context variables, helper methods, partial registration |
| `crates/controller/src/tasks/code/controller.rs` | Reconciliation loop — watches CodeRun CRDs, creates Jobs, handles status |
| `crates/controller/src/crds/coderun.rs` | CRD definition with serde fix |
| `templates/clis/_archived/` | 8 archived legacy CLI templates (kept for reference) |

## Outstanding Work

### Immediate (test the build)
- Scale controller to 1, create CodeRun, verify unified ACP dispatch + promtail works

### Short-term
- **Per-agent Discord bot notifications**: `crates/notify/src/channels/discord.rs` has generic webhook — needs a `DiscordBotChannel` using `DISCORD_TOKEN_{AGENT}` + `DISCORD_CHANNEL_{AGENT}` from the mounted `discord-agent-bots` secret
- **Controller OTEL exporter**: Wire `opentelemetry_otlp` in `agent_controller.rs` so adapter metrics actually export (currently all go to no-op sink)
- **PR to main**: Create PR `acp-testing` → `main` with all 8 commits

### Longer-term (from plan.md)
- Multi-provider agent-harness mapping (Phase 1)
- Plugin Tools MCP Bridge verification (Phase 2)
- Discord + Linear observability per Play task (Phase 3)
- Fallback cascade on harness failure (Phase 4)

## Build Notes

The controller cross-compile uses `controller-build.Dockerfile` at repo root (gitignored):

```bash
docker buildx build \
  --builder amd64-builder \
  --platform linux/amd64 \
  -t ghcr.io/5dlabs/controller:latest \
  -f controller-build.Dockerfile \
  --push .
```

⚠️ CI `build-and-push` job doesn't work (k8s-runner lacks Docker socket). Image builds are done locally via buildx.

## CLI Auth Keys

- **Droid (Factory)**: API key `fk-TUWFrDboAFn7GWtfAfk1-xvw_kvVdmdy5SeYPHRSVJor3HXFYf8kcMN5yI1X6CKg` — pass as `FACTORY_API_KEY` env
- **Cursor**: API key in OpenBao + cluster secret `crsr_c4ec...`
- **Claude**: OAuth (admin@5dlabs.ai Pro) — rate limited, Vertex disabled
- **Codex**: ChatGPT OAuth — working locally
- **OpenRouter**: Working API key on cluster for Claude models

## Session Context

This work was done on Copilot CLI. The session has 36 checkpoints with full history available at:
`~/.copilot/session-state/ca730d6b-25c0-4528-aebc-a413eefd53a2/`
