# Research Create System

## Overview

Automated research + PRD generator that:
1. Searches X via Grok every 5 minutes
2. Filters for CTO-relevant developer content
3. Creates PRDs for worthwhile projects
4. Posts findings to Discord

## Quick Start

### Run Manually
```bash
bun run src/utils/research-create.ts --test
```

### Install Cron (every 5 minutes)
```bash
crontab -e
# Add line:
*/5 * * * * /Users/jonathonfritz/agents/research/scripts/research-create.sh >> /tmp/research-create.log 2>&1
```

## Configuration

Edit `src/utils/research-create.ts`:

```typescript
const CONFIG = {
  MIN_ENGAGEMENT: 50,       // Posts with 50+ likes
  MIN_LIKES_FOR_PRD: 100,   // Creates PRD for 100+ likes
  MAX_POSTS_PER_RUN: 5,     // Limit per run
};
```

## Keywords

30 developer-focused categories:

| Category | Focus |
|----------|-------|
| agent-development | Orchestration, patterns, protocols |
| claude-sdk | Claude API, function calling |
| openai-sdk | OpenAI API, streaming, batch |
| mcp | Model Context Protocol |
| llm-inference | vLLM, TensorRT, quantization |
| vector-databases | Pinecone, Weaviate, RAG |
| ai-security | Prompt injection, jailbreak |
| coding-agents | SWE-bench, autonomous coding |
| developer-tools | Cursor, Copilot, Claude Code |
| kubernetes | k8s, Helm, operators |
| rust-ai | llama.cpp, Candle, MCP servers |
| go-ai | Go MCP, concurrency |
| And 19 more... |

See `src/utils/keywords.ts` for full list.

## Output

- **Posts**: Posted to Discord channel
- **PRDs**: Saved to `docs/prds/{post-id}.md`
- **Logs**: `/tmp/research-create.log`

## Example PRD

```markdown
---
id: "2018768974872449100"
author: "research-agent"
source_url: "https://x.com/i/status/2018768974872449100"
likes: 1073
---

# Research: 2018768974872449100

## Summary
...

## Implementation Ideas
- Build with MCP integration
- Create skill for OpenClaw
- Evaluate using SWE-bench
```

## Dependencies

- Bun runtime
- 1Password CLI (for Grok API key)
- Grok API access
- Discord webhook (optional)
