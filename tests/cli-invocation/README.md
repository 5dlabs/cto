# CLI Invocation Tests

End-to-end testing for CTO agents with Linear integration.

## Quick Start

```bash
# 1. Setup environment
cp .env.example .env
# Edit .env with your credentials (see "OAuth Token Setup" below)

# 2. Generate per-agent configurations
./scaffold-skills.sh     # Creates config/skills-{agent}/ for all agents
./scaffold-agents.sh     # Creates config/client-config-{agent}.json

# 3. Build Docker images (first time only)
./build-images.sh all

# 4. Run an agent test
./run-agent.sh bolt      # Run Bolt agent test
./run-agent.sh rex       # Run Rex agent test
./run-agent.sh blaze     # Run Blaze agent test
```

## Directory Structure

```
tests/cli-invocation/
├── config/
│   ├── skills-{agent}/           # Per-agent skills (from scaffold-skills.sh)
│   │   └── {skill-name}/
│   │       └── SKILL.md
│   ├── client-config-{agent}.json  # Per-agent MCP tool filtering
│   ├── task-{agent}/             # Per-agent task prompts
│   │   └── prompt.md
│   ├── claude-settings.json      # Claude CLI settings
│   └── agents/                   # Sub-agent definitions
├── scripts/
│   ├── run-claude.sh             # Container entrypoint for Claude CLI
│   ├── run-bolt.sh               # Bolt-specific entrypoint
│   └── run-factory.sh            # Factory CLI entrypoint
├── workspaces/
│   └── {agent}/                  # Agent workspace (stream.jsonl output)
├── docs/
│   └── GITHUB-TOKENS.md          # GitHub App token documentation
├── docker-compose.yml            # Multi-agent service definitions
├── .env.example                  # Environment template
├── scaffold-skills.sh            # Generate per-agent skills
├── scaffold-agents.sh            # Generate per-agent configs
├── verify-github-tokens.sh       # Verify GitHub App tokens
├── run-agent.sh                  # Easy agent test runner
└── build-images.sh               # Build Docker images
```

## Available Agents

All agents use the Claude CLI:

| Agent | Specialty | Skills | Tools |
|-------|-----------|--------|-------|
| **bolt** | Infrastructure & DevOps | 20 | 53 |
| **rex** | Backend Rust Development | 22 | 53 |
| **blaze** | Frontend Development | 26 | 30 |
| **morgan** | Project Management | 20 | 25 |
| **cleo** | Code Review & Quality | 17 | 25 |
| **tess** | Testing & QA | 19 | 25 |
| **atlas** | Integration & Coordination | 15 | 20 |
| **cipher** | Security Analysis | 20 | 25 |
| **grizz** | Database & Backend | 21 | 30 |
| **nova** | Data Science | 23 | 30 |
| **tap** | Mobile (Expo) | 26 | 30 |
| **spark** | Electron Desktop | 20 | 25 |
| **stitch** | PR Review | 9 | 15 |
| **vex** | VR/AR Development | 18 | 20 |

## OAuth Token Setup

### 1. Linear OAuth Token

Required for posting agent sessions to Linear issues.

```bash
# Read from 1Password
op read "op://Automation/Linear Morgan OAuth/developer_token" --reveal
```

Add to `.env`:
```
LINEAR_OAUTH_TOKEN=your_token_here
```

### 2. Anthropic API Key

Required for Claude CLI execution.

```bash
# Read from 1Password
op read "op://Automation/cto-secrets/anthropic_api_key" --reveal
```

Add to `.env`:
```
ANTHROPIC_API_KEY=your_key_here
```

### 3. Linear Issue Identifier

The Linear issue where agent sessions will be posted.

```
LINEAR_ISSUE_IDENTIFIER=CTOPA-123
```

### 4. GitHub App Tokens (Optional)

Each agent has a dedicated GitHub App for repository access. Tokens are auto-generated from private keys.

See `docs/GITHUB-TOKENS.md` for details.

```bash
# Verify all tokens
./verify-github-tokens.sh
```

## Running Tests

### Single Agent

```bash
# Using the helper script (recommended)
./run-agent.sh bolt
./run-agent.sh rex coder

# Using docker compose directly
docker compose --profile bolt up
```

### Multiple Agents

```bash
# Run all agents (parallel)
docker compose --profile all up
```

### Viewing Output

Agent output is written to `workspaces/{agent}/stream.jsonl`:

```bash
# View stream types
cat workspaces/bolt/stream.jsonl | jq -r '.type' | sort | uniq -c

# View init message (skills, tools, model)
head -1 workspaces/bolt/stream.jsonl | jq .

# View tool calls
grep '"tool_use"' workspaces/bolt/stream.jsonl | jq .
```

## Troubleshooting

### "Skills showing empty []"

**Cause:** Skills not mounted or wrong directory structure.

**Fix:**
```bash
./scaffold-skills.sh {agent}
ls config/skills-{agent}/  # Should show directories, not flat files
```

### "MCP tools 0/X used"

**Cause:** Tool filtering not working or CLI didn't use MCP tools.

**Check:**
```bash
jq '.remoteTools | length' config/client-config-{agent}.json
grep mcp workspaces/{agent}/stream.jsonl
```

### "Container exits immediately"

**Cause:** Missing environment variables or API keys.

**Check:**
```bash
docker compose --profile bolt config  # Validate compose file
grep -E "^(ANTHROPIC|LINEAR)" .env    # Verify secrets
```

### "Linear post failed"

**Cause:** Invalid OAuth token or issue not accessible.

**Check:**
```bash
# Test token
curl -H "Authorization: Bearer $LINEAR_OAUTH_TOKEN" \
  https://api.linear.app/graphql \
  -d '{"query":"{ viewer { id } }"}'
```

### Docker Image Issues

```bash
# Rebuild images
./build-images.sh all

# Check image exists
docker images | grep cto-claude

# Pull base images if slow
docker pull debian:bookworm-slim
```

## Configuration Sources

Skills and tools are configured in `cto-config.json`:

```json
{
  "agents": {
    "bolt": {
      "skills": {
        "default": ["context-fundamentals", "github-mcp", ...],
        "coder": ["compound-engineering", "systematic-debugging", ...],
        "healer": ["incident-response", "observability", ...]
      },
      "tools": {
        "remote": ["github_push_files", "github_create_pr", ...]
      }
    }
  }
}
```

Fallback skills are in `templates/skills/skill-mappings.yaml`.

## Adding a New Agent

1. Add agent config to `cto-config.json`
2. Run scaffolding:
   ```bash
   ./scaffold-skills.sh {agent}
   ./scaffold-agents.sh
   ```
3. Add docker-compose service (copy from existing agent)
4. Create task prompt in `config/task-{agent}/prompt.md`
5. Test: `./run-agent.sh {agent}`

## Development

### Modifying Skills

Skills source: `../../templates/skills/`

After modifying, regenerate:
```bash
./scaffold-skills.sh
```

### Modifying Tools

Tools source: `cto-config.json` → `agents.{name}.tools.remote`

After modifying, regenerate:
```bash
./scaffold-agents.sh
```

### Sidecar Development

Sidecar source: `../../crates/linear-sync/src/bin/linear-sidecar.rs`

Rebuild:
```bash
./build-images.sh sidecar
```
