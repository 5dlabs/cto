# CLI Telemetry Implementation Plan

This document provides the exact implementation steps to enable telemetry collection from all supported CLI backends.

## Current State

| CLI | Status | Data Types | Protocol |
|-----|--------|------------|----------|
| **Claude Code** | ✅ Configured | Metrics, Logs | OTLP gRPC |
| **Codex CLI** | ❌ Not configured | Logs only | OTLP gRPC/HTTP |
| **Gemini CLI** | ❌ Not configured | Metrics, Logs, Traces | OTLP gRPC/HTTP |
| **Factory (Droid)** | ❌ Not configured | Metrics, Logs, Traces | OTLP gRPC/HTTP |
| **OpenCode** | N/A | None | N/A |
| **Cursor** | N/A | None | N/A |

## OTEL Collector Compatibility

The current OTEL collector (`infra/telemetry/values/otel-collector.yaml`) already supports:
- ✅ OTLP gRPC receiver on port 4317
- ✅ OTLP HTTP receiver on port 4318
- ✅ Metrics export to Victoria Metrics
- ✅ Logs export to Victoria Logs
- ✅ Traces (debug only currently)

**No collector changes required** - all CLIs use standard OTLP protocol.

---

## 1. Codex CLI Configuration

### Data Exported
**Logs only** (no metrics). Events:
- `codex.conversation_starts` - session start with config
- `codex.api_request` - API calls with timing/status
- `codex.sse_event` - streaming events with token counts
- `codex.user_prompt` - user prompts (redactable)
- `codex.tool_decision` - approval decisions
- `codex.tool_result` - tool execution results

### Implementation

Add `[otel]` section to `infra/charts/controller/agent-templates/code/codex/config.toml.hbs`:

```toml
{{#if telemetry.enabled}}
[otel]
environment = "production"
log_user_prompt = false

[otel.exporter.otlp-grpc]
endpoint = "{{telemetry.otlpEndpoint}}"
{{/if}}
```

### Files to Modify
- `infra/charts/controller/agent-templates/code/codex/config.toml.hbs`

---

## 2. Gemini CLI Configuration

### Data Exported
**Metrics, Logs, and Traces**.

Key Metrics:
- `gemini_cli.session.count` - sessions started
- `gemini_cli.tool.call.count` - tool calls
- `gemini_cli.tool.call.latency` - tool latency
- `gemini_cli.api.request.count` - API requests
- `gemini_cli.api.request.latency` - API latency
- `gemini_cli.token.usage` - token counts
- `gemini_cli.file.operation.count` - file operations
- `gen_ai.client.token.usage` - GenAI semantic convention
- `gen_ai.client.operation.duration` - GenAI semantic convention

Key Events:
- `gemini_cli.config` - startup configuration
- `gemini_cli.user_prompt` - user prompts
- `gemini_cli.tool_call` - tool executions
- `gemini_cli.api_request/response/error` - API lifecycle

### Implementation

Add environment variables to container scripts. Modify `infra/charts/controller/agent-templates/code/gemini/container-base.sh.hbs`:

Add after the environment setup section:

```bash
{{#if telemetry.enabled}}
# Gemini CLI Telemetry Configuration
export GEMINI_TELEMETRY_ENABLED=true
export GEMINI_TELEMETRY_TARGET=local
export GEMINI_TELEMETRY_OTLP_ENDPOINT="{{telemetry.otlpEndpoint}}"
export GEMINI_TELEMETRY_OTLP_PROTOCOL="{{telemetry.otlpProtocol}}"
export GEMINI_TELEMETRY_LOG_PROMPTS=false
echo "✓ Gemini telemetry enabled: {{telemetry.otlpEndpoint}}"
{{/if}}
```

### Files to Modify
- `infra/charts/controller/agent-templates/code/gemini/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/gemini/container-blaze.sh.hbs`
- `infra/charts/controller/agent-templates/code/gemini/container-cipher.sh.hbs`
- `infra/charts/controller/agent-templates/code/gemini/container-cleo.sh.hbs`
- `infra/charts/controller/agent-templates/code/gemini/container-rex.sh.hbs`
- `infra/charts/controller/agent-templates/code/gemini/container-rex-remediation.sh.hbs`
- `infra/charts/controller/agent-templates/code/gemini/container-tess.sh.hbs`

---

## 3. Factory (Droid) CLI Configuration

### Data Exported
**Metrics, Logs, and Traces** via standard OTEL SDK.

Key Metrics:
- Session counts (interactive/headless)
- LLM token usage (in/out by model)
- API request counts and latencies
- Tool invocations and execution time
- File operations (modified/created/deleted)

### Implementation

Add standard OTEL environment variables to container scripts. Modify `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`:

```bash
{{#if telemetry.enabled}}
# Factory Droid Telemetry Configuration (standard OTEL SDK)
export OTEL_EXPORTER_OTLP_ENDPOINT="{{telemetry.otlpEndpoint}}"
export OTEL_EXPORTER_OTLP_PROTOCOL="{{telemetry.otlpProtocol}}"
export OTEL_SERVICE_NAME="factory-droid"
export OTEL_SERVICE_VERSION="1.0.0"
export OTEL_RESOURCE_ATTRIBUTES="deployment.environment=production,service.namespace=agent-platform"
echo "✓ Factory telemetry enabled: {{telemetry.otlpEndpoint}}"
{{/if}}
```

### Files to Modify
- `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-blaze.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-cipher.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-cleo.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-rex.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-rex-remediation.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-tess.sh.hbs`

---

## Telemetry Context Configuration

The telemetry settings are already defined in `infra/charts/controller/templates/task-controller-config.yaml`:

```yaml
telemetry:
  enabled: true
  otlpEndpoint: "otel-collector-opentelemetry-collector.telemetry.svc.cluster.local:4317"
  otlpProtocol: "grpc"
  logsEndpoint: "otel-collector-opentelemetry-collector.telemetry.svc.cluster.local:4317"
  logsProtocol: "grpc"
```

These values are already available in all Handlebars templates via the `telemetry` context.

---

## Implementation Order

1. **Codex CLI** (simplest - just TOML addition)
   - Estimated time: 15 minutes
   - Risk: Low

2. **Gemini CLI** (env vars in shell scripts)
   - Estimated time: 30 minutes
   - Risk: Low

3. **Factory (Droid)** (env vars in shell scripts)
   - Estimated time: 30 minutes
   - Risk: Low

---

## Validation

After deployment, verify telemetry is flowing:

```bash
# Check Victoria Metrics for new metrics
curl -s "http://victoria-metrics:8428/api/v1/label/__name__/values" | jq '.data | map(select(startswith("gemini_cli") or startswith("codex") or startswith("factory")))'

# Check Victoria Logs for new log streams
curl -s "http://victoria-logs:9428/select/logsql/query" -d 'query=service.name:("codex_cli_rs" OR "gemini-cli" OR "factory-droid")'

# Check OTEL collector debug logs
kubectl logs -n telemetry deployment/otel-collector-opentelemetry-collector | grep -E "(gemini|codex|factory)"
```

---

## Summary

| CLI | Change Type | Files Modified | Effort |
|-----|-------------|----------------|--------|
| **Codex** | TOML section | 1 file | 15 min |
| **Gemini** | Env vars | 7 files | 30 min |
| **Factory** | Env vars | 7 files | 30 min |
| **OTEL Collector** | None | 0 files | 0 min |

**Total estimated effort: ~1.5 hours**
