# Agent Metrics Reference

This document provides a comprehensive reference for token usage and metrics collection
across all CLI backends used by the CTO platform agents.

## CLI Capabilities Matrix

| CLI | Native OTEL | Token Extraction | Output Format | Current Storage | Persistence |
|-----|-------------|------------------|---------------|-----------------|-------------|
| Gemini | Yes (full) | Via OTEL | stream-json | OTEL collector | Persistent (if enabled) |
| Claude Code | Yes | Via OTEL | stream-json | OTEL collector | Persistent (if enabled) |
| Factory | No | Custom JSON parse | json/debug | `/tmp/factory-metrics.jsonl` | Lost on pod death |
| Cursor | No | Custom JSON parse | json | `/tmp/cursor-metrics.jsonl` | Lost on pod death |
| Codex | No | Text + JSON | text/json | `/tmp/codex-metrics.jsonl` | Lost on pod death |
| OpenCode | No | Limit check only | text | None | N/A |

## Detailed CLI Analysis

### Gemini CLI

**Telemetry Support**: Full OpenTelemetry integration with comprehensive metrics.

**Configuration** (in `container-base.sh.hbs`):
```bash
{{#if telemetry.enabled}}
export GEMINI_TELEMETRY_ENABLED=true
export GEMINI_TELEMETRY_TARGET=local
export GEMINI_TELEMETRY_OTLP_ENDPOINT="{{telemetry.otlpEndpoint}}"
export GEMINI_TELEMETRY_OTLP_PROTOCOL="{{telemetry.otlpProtocol}}"
export GEMINI_TELEMETRY_LOG_PROMPTS=false
{{/if}}
```

**Available Metrics via OTEL**:
- `gemini_cli.token.usage` (Counter) - with `type` attribute: input/output/thought/cache/tool
- `gen_ai.client.token.usage` (Histogram) - OpenTelemetry semantic conventions
- `gemini_cli.api.request.count` - API request counts
- `gemini_cli.api.request.latency` - Latency histogram
- `gemini_cli.tool.call.count` - Tool usage tracking
- `gemini_cli.agent.duration` - Agent run durations

**Current Status**: Configured but telemetry block not present in values.yaml.

### Claude Code

**Telemetry Support**: OpenTelemetry via environment variables.

**Configuration** (in `settings.json.hbs`):
```json
{
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "{{#if telemetry.enabled}}1{{else}}0{{/if}}",
    "OTEL_METRICS_EXPORTER": "otlp",
    "OTEL_LOGS_EXPORTER": "otlp",
    "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT": "otel-collector-opentelemetry-collector.observability.svc.cluster.local:4317",
    "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL": "grpc"
  }
}
```

**Current Status**: Configured but telemetry block not present in values.yaml.

### Factory CLI

**Token Extraction** (in `container-base.sh.hbs` ~line 2288):
```bash
# Extract token usage for cost tracking (JSON format only)
if [ "$OUTPUT_FORMAT" = "json" ] || [ "$OUTPUT_FORMAT" = "debug" ]; then
  TOKENS_IN=$(jq -r 'select(.usage.input_tokens != null) | .usage.input_tokens' "$RUN_LOG" 2>/dev/null | tail -n1)
  TOKENS_OUT=$(jq -r 'select(.usage.output_tokens != null) | .usage.output_tokens' "$RUN_LOG" 2>/dev/null | tail -n1)
  
  if [[ "$TOKENS_IN" =~ ^[0-9]+$ ]] && [[ "$TOKENS_OUT" =~ ^[0-9]+$ ]]; then
    echo "{...}" >> /tmp/factory-metrics.jsonl
  fi
fi
```

**JSON Format**: `.usage.input_tokens`, `.usage.output_tokens`

**Issue**: Writes to `/tmp/` which is lost when pod terminates.

### Cursor CLI

**Token Extraction** (in `container-base.sh.hbs` ~line 1957):
```bash
TOKENS_IN=$(jq -r 'select(.metadata.usage.inputTokens != null) | .metadata.usage.inputTokens' "$RUN_LOG" 2>/dev/null | tail -n1)
TOKENS_OUT=$(jq -r 'select(.metadata.usage.outputTokens != null) | .metadata.usage.outputTokens' "$RUN_LOG" 2>/dev/null | tail -n1)

echo "{...}" >> /tmp/cursor-metrics.jsonl
```

**JSON Format**: `.metadata.usage.inputTokens`, `.metadata.usage.outputTokens`

**Issue**: Writes to `/tmp/` which is lost when pod terminates.

### Codex CLI

**Token Extraction** (in `container-base.sh.hbs` ~line 2213):
```bash
# Try text format first
LAST_TOKENS=$(grep -Eo 'tokens used: [0-9,]+' "$RUN_LOG" 2>/dev/null | tail -n1 | awk '{print $3}' | tr -d ',' || true)

# Fallback to JSON format
if [ -z "$LAST_TOKENS" ]; then
  TOKENS_IN=$(jq -r 'select(.tokens.input != null) | .tokens.input' "$RUN_LOG" 2>/dev/null | tail -n1)
  TOKENS_OUT=$(jq -r 'select(.tokens.output != null) | .tokens.output' "$RUN_LOG" 2>/dev/null | tail -n1)
fi

echo "{...}" >> /tmp/codex-metrics.jsonl
```

**Text Format**: `tokens used: [0-9,]+`
**JSON Format**: `.tokens.input`, `.tokens.output`

**Issue**: Writes to `/tmp/` which is lost when pod terminates.

### OpenCode CLI

**Token Extraction** (in `container-base.sh.hbs` ~line 1105):
```bash
if [ "${TOKEN_LIMIT:-0}" -gt 0 ] && [ -f "$RUN_LOG" ]; then
  LAST_TOKENS=$(grep -Eo 'tokens used: [0-9,]+' "$RUN_LOG" 2>/dev/null | tail -n1 | awk '{print $3}' | tr -d ',' || true)
  if [ "$LAST_TOKENS" -gt "$TOKEN_LIMIT" ]; then
    # Force clean session
  fi
fi
```

**Text Format**: `tokens used: [0-9,]+` (total only)

**Issues**:
- Only extracts for limit checking, does not persist
- No input/output breakdown
- No metrics file written

## Telemetry Configuration Status

### Current values.yaml State

The `values.yaml` file does **NOT** currently have a telemetry section at the top level.
This means `{{#if telemetry.enabled}}` blocks in templates evaluate to false.

### Required Configuration

To enable OTEL telemetry, add to `values.yaml`:

```yaml
telemetry:
  enabled: true
  otlpEndpoint: "otel-collector.telemetry.svc.cluster.local:4317"
  otlpProtocol: "grpc"
```

## Provider API Integration

### Anthropic Usage API

**Endpoint**: `GET /v1/organizations/usage_report/messages`
**Auth**: Admin API Key (`sk-ant-admin-...`)
**Capabilities**:
- Token breakdown by model, workspace, service tier
- Cache statistics (cached vs uncached tokens)
- Cost data via `/v1/organizations/cost_report`

**Example**:
```bash
curl "https://api.anthropic.com/v1/organizations/usage_report/messages?\
starting_at=2025-01-01T00:00:00Z&\
ending_at=2025-01-08T00:00:00Z&\
group_by[]=model&bucket_width=1d" \
  --header "anthropic-version: 2023-06-01" \
  --header "x-api-key: $ANTHROPIC_ADMIN_KEY"
```

**Data Freshness**: ~5 minutes
**Rate Limit**: Once per minute sustained

### OpenAI Usage API

**Endpoint**: `GET /v1/organization/usage/completions`
**Auth**: Admin API Key
**Capabilities**:
- Input/output/cached tokens
- Groupable by model, project, user, api_key
- Cost endpoint: `/v1/organization/costs`

**Example**:
```bash
curl "https://api.openai.com/v1/organization/usage/completions?\
start_time=$(date -d 'yesterday' +%s)&group_by[]=model" \
  -H "Authorization: Bearer $OPENAI_ADMIN_KEY"
```

## Metrics Schema

### Unified JSON Schema for `/workspace/.metrics/`

```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "task_id": "42",
  "service": "my-service",
  "agent": "rex",
  "cli": "cursor",
  "model": "claude-sonnet-4-5-20250929",
  "tokens": {
    "input": 45000,
    "output": 8500,
    "total": 53500
  },
  "context_pct": 22.5,
  "attempts": 1,
  "success": 1,
  "duration_ms": 45000
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| timestamp | ISO8601 | UTC timestamp of metric collection |
| task_id | string | TaskMaster task ID |
| service | string | Target service name |
| agent | string | Agent name (rex, cleo, cipher, tess) |
| cli | string | CLI backend (cursor, claude, codex, etc.) |
| model | string | Model identifier |
| tokens.input | int | Input tokens consumed |
| tokens.output | int | Output tokens generated |
| tokens.total | int | Total tokens (input + output) |
| context_pct | float | Percentage of context window used |
| attempts | int | Retry attempt number |
| success | int | 1 if successful, 0 if failed |
| duration_ms | int | Execution duration in milliseconds |

## Cost Estimation

### Model Pricing (as of 2025)

| Provider | Model | Input (per 1M) | Output (per 1M) |
|----------|-------|----------------|-----------------|
| Anthropic | Claude Sonnet 4 | $3.00 | $15.00 |
| Anthropic | Claude Opus 4.5 | $15.00 | $75.00 |
| OpenAI | GPT-4o | $2.50 | $10.00 |
| OpenAI | Codex | $0.03 | $0.06 |
| Google | Gemini 2.5 Pro | $1.25 | $5.00 |

### Cost Calculation Formula

```
cost = (input_tokens * input_price / 1_000_000) + (output_tokens * output_price / 1_000_000)
```

## Context Window Limits

| Model | Context Window | 70% Threshold | 90% Threshold |
|-------|---------------|---------------|---------------|
| Claude Sonnet 4 | 200K | 140K | 180K |
| Claude Opus 4.5 | 200K | 140K | 180K |
| GPT-4o | 128K | 89.6K | 115.2K |
| Gemini 2.5 Pro | 2M | 1.4M | 1.8M |

## Implementation Recommendations

### Priority 1: Fix Persistence

Replace `/tmp/` writes with `/workspace/.metrics/`:
- Factory: line ~2299
- Cursor: line ~1967
- Codex: line ~2228

### Priority 2: Enable OTEL

Add telemetry block to values.yaml to enable native telemetry for Gemini and Claude Code.

### Priority 3: Add OpenCode Metrics

OpenCode currently only checks limits. Need to add metrics extraction and persistence.

### Priority 4: Provider API Polling

Create daily job to poll Anthropic and OpenAI usage APIs for authoritative billing data.

---

*Last updated: 2025-11-29*
*Author: CTO Platform Team*

