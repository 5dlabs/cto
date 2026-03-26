# Intake Agent

The `intake-agent` binary is now the thin runtime for the parts of intake that still need local code: JSON protocol handling and multi-source PRD research.

The intake pipeline itself has moved to Lobster workflows under `intake/workflows/`. Deliberation, task generation, voting, revision loops, and bridge-driven observability are no longer implemented inside this executable.

## What Changed Recently

- Intake orchestration is now Lobster-native instead of split between shell scripts and agent-local orchestration.
- Deliberation moved to [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/deliberation.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/deliberation.lobster.yaml).
- Top-level intake flow lives in [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml).
- Task expansion now uses a vote-gated revision loop in [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/task-refinement.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/task-refinement.lobster.yaml).
- Intake observability and approvals now flow through HTTP/webhook bridges instead of the old NATS-driven execution path inside this binary.
- Non-greenfield intake can add codebase analysis before PRD parsing.

## Current Responsibility

This executable currently supports only:

| Operation | Description |
|-----------|-------------|
| `ping` | Health check for the binary |
| `prd_research` | Multi-source PRD research via Exa, Perplexity, Tavily, and Firecrawl |

If you are looking for task generation, deliberation, or task refinement behavior, use the workflow docs instead of this binary README:

- [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/docs/intake-process.md`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/docs/intake-process.md)
- [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml)

## Usage

```bash
# Health check
echo '{"operation":"ping"}' | bun run src/index.ts

# PRD research
echo '{"operation":"prd_research","payload":{"prd_content":"..."}}' | bun run src/index.ts
```

## Building

```bash
bun install
bun run build
bun run typecheck
```

## JSON Protocol

### Request

```json
{
  "operation": "ping" | "prd_research",
  "payload": {}
}
```

### Success Response

```json
{
  "success": true,
  "data": {},
  "usage": { "input_tokens": 0, "output_tokens": 0, "total_tokens": 0 },
  "model": "...",
  "provider": "..."
}
```

### Error Response

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
| `EXA_API_KEY` | Optional | Exa search API |
| `PERPLEXITY_API_KEY` | Optional | Perplexity API |
| `TAVILY_API_KEY` | Optional | Tavily search API |
| `FIRECRAWL_API_KEY` | Optional | Firecrawl extraction API |

## Notes

- `prd_research` is the only operation that performs external research from this binary.
- Deliberation is no longer a valid binary operation.
- Some TypeScript types still include historical deliberation structures because they are shared with workflow-adjacent code, but the executable only accepts `ping` and `prd_research`.

## License

MIT
