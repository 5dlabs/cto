# Intake Agent Status

## ✅ Working Operations

| Operation | Status | Notes |
|-----------|--------|-------|
| `ping` | ✅ | Health check |
| `provider_status` | ✅ | Shows claude=available, minimax/codex need API keys |
| `research_capabilities` | ✅ | context7 + exa always available |
| `parse_prd` | ✅ | Generates tasks with agent mapping in titles |
| `expand_task` | ✅ | Generates subtasks with `subagentType` + `parallelizable` |
| `analyze_complexity` | ✅ | Returns complexity scores and subtask recommendations |
| `generate` | ✅ | Basic generation |
| `generate_with_critic` | ✅ | Dual-agent with refinement loop |
| `validate_content` | ✅ | Critic-only validation |
| `research` | ⚠️ | Runs but MCP tools not fully connected |

## Agent Hints (subagentType)

Supported values for subtask expansion:
- `implementer` - Writing code, creating features
- `reviewer` - Code review, architecture review  
- `tester` - Writing tests, QA validation
- `researcher` - Research, spikes, investigations
- `documenter` - Documentation, comments, READMEs

## Dual-Agent Pattern

**Generator** (optimistic): Creates initial content
- Default: Claude (claude-sonnet-4)
- Role: Generate based on prompts

**Critic** (pessimistic): Evaluates and finds issues
- Default: Minimax (needs MINIMAX_API_KEY)
- Fallback: Claude as critic
- Role: Identify issues, suggest improvements

**Flow:**
1. Generator creates content
2. Critic evaluates (approved/rejected + issues)
3. If rejected → refine and re-evaluate
4. Max refinements configurable (default: 2)

## MCP Servers

| Server | Status | Env Var |
|--------|--------|---------|
| context7 | ✅ Always | - |
| exa | ✅ Always | - |
| firecrawl | ❌ | `FIRECRAWL_API_KEY` |
| octocode | ❌ | `GITHUB_TOKEN` |
| tavily | ❌ | `TAVILY_API_KEY` |
| perplexity | ❌ | `PERPLEXITY_API_KEY` |

## Provider Configuration

| Provider | Env Var | Default Model |
|----------|---------|---------------|
| Claude | (uses Claude Code auth) | claude-sonnet-4-20250514 |
| Minimax | `MINIMAX_API_KEY` | MiniMax-M2.1-lightning |
| Codex | `OPENAI_API_KEY` | gpt-4o |

## Usage Examples

```bash
# Ping
echo '{"operation":"ping"}' | ./dist/intake-agent

# Parse PRD
echo '{"operation":"parse_prd","payload":{"prd_content":"..."}}' | ./dist/intake-agent

# Expand task with agent hints
echo '{"operation":"expand_task","payload":{"task":{"id":1,"title":"...","description":"..."}}}' | ./dist/intake-agent

# Dual-agent generation
echo '{"operation":"generate_with_critic","payload":{"system_prompt":"...","user_prompt":"...","config":{"generator":"claude","critic":"claude"}}}' | ./dist/intake-agent
```

## Debug Mode

Enable granular logging with environment variables:

```bash
# Standard debug logging
LOG_LEVEL=debug ./dist/intake-agent

# Trace level (most granular - message by message)
LOG_LEVEL=trace ./dist/intake-agent

# Shorthand for debug mode
DEBUG=1 ./dist/intake-agent

# Disable colors
NO_COLOR=1 LOG_LEVEL=debug ./dist/intake-agent
```

**Log Levels:**
- `trace` - Message-by-message, MCP tool calls, prompts
- `debug` - Operation steps, timing, configurations
- `info` - Key events (default)
- `warn` - Potential issues
- `error` - Failures

**Output:**
- Logs go to stderr (colored)
- JSON response goes to stdout
- Safe for piping: `cat input.json | ./dist/intake-agent 2>debug.log`

## Known Issues

1. **Research operation**: MCP tools (context7, exa) connected but not returning structured findings
2. **Minimax unavailable**: Need `MINIMAX_API_KEY` for true dual-model critic
3. **Tool server mode**: `TOOLS_SERVER_URL` not tested yet

## Next Steps

- [ ] Fix research operation to use MCP tools properly
- [ ] Test with Minimax API key for true dual-model
- [ ] Add tool server integration tests
- [ ] Add streaming support for long operations
