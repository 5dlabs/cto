# Intake Agent

PRD research and deliberation orchestrator. Reads JSON from stdin, writes JSON to stdout.

All LLM-based operations (parse_prd, expand_task, analyze_complexity, etc.) have been
migrated to Lobster `llm-task` steps. This binary retains only the operations that
require stateful orchestration or multi-source API calls.

## Operations

| Operation | Description |
|-----------|-------------|
| `ping` | Health check |
| `prd_research` | Multi-source research via Exa, Perplexity, Tavily, Firecrawl |
| `deliberate` | Stateful NATS debate loop with committee voting |

## Usage

```bash
# Health check
echo '{"operation":"ping"}' | bun run src/index.ts

# PRD research (requires API keys)
echo '{"operation":"prd_research","payload":{"prd_content":"..."}}' | bun run src/index.ts

# Deliberation (requires NATS)
echo '{"operation":"deliberate","payload":{"session_id":"...","prd_content":"..."}}' | bun run src/index.ts
```

## Building

```bash
bun install
bun run build        # outputs dist/intake-agent
bun run typecheck    # type-check only
```

## JSON Protocol

### Request

```json
{
  "operation": "ping" | "deliberate" | "prd_research",
  "payload": { }
}
```

### Response (Success)

```json
{
  "success": true,
  "data": { },
  "usage": { "input_tokens": 0, "output_tokens": 0, "total_tokens": 0 },
  "model": "...",
  "provider": "..."
}
```

### Response (Error)

```json
{
  "success": false,
  "error": "message",
  "error_type": "api_error" | "parse_error" | "validation_error" | "unknown"
}
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `NATS_URL` | For deliberate | NATS server URL |
| `EXA_API_KEY` | Optional | Exa search API |
| `PERPLEXITY_API_KEY` | Optional | Perplexity API |
| `TAVILY_API_KEY` | Optional | Tavily search API |
| `FIRECRAWL_API_KEY` | Optional | Firecrawl extraction API |
| `DISCORD_WEBHOOK_URL` | Optional | Discord webhook for deliberation updates |

## License

MIT
