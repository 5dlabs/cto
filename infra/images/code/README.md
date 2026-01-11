# Every Code Agent

This image provides [Every Code](https://github.com/just-every/code) integration for multi-agent code generation and orchestration tasks.

Every Code is a community fork of OpenAI Codex CLI that adds:

- **Multi-agent commands**: `/plan`, `/solve`, `/code`, `/auto`
- **Browser integration**: `/chrome`, `/browser`
- **Multi-provider support**: OpenAI, Claude, Gemini, Qwen
- **Enhanced reasoning controls**: Configurable reasoning effort and summaries
- **Theme system**: Customizable TUI themes

## Features

- Every Code CLI (Rust binary)
- Multi-provider AI support (OpenAI, Anthropic, Google, Qwen)
- MCP server support for extended capabilities
- AGENTS.md memory file support
- TOML-based configuration

## Environment Variables

The following environment variables can be configured:

### Provider API Keys (configure based on model provider)

- `OPENAI_API_KEY` - OpenAI API key
- `ANTHROPIC_API_KEY` - Anthropic API key (for Claude models)
- `GOOGLE_API_KEY` - Google API key (for Gemini models)

### Configuration

- `CODE_HOME` - Override config directory location (default: `~/.code`)
- `OPENAI_BASE_URL` - Use OpenAI-compatible API endpoints

## Configuration

Every Code uses TOML configuration at `~/.code/config.toml`:

```toml
# Model settings
model = "gpt-5.1"
model_provider = "openai"

# Behavior
approval_policy = "never"  # For CI/automation
sandbox_mode = "workspace-write"
model_reasoning_effort = "medium"

# MCP servers
[mcp_servers.tools]
command = "tools"
args = ["--url", "http://localhost:3000/mcp"]

# Theme
[tui.theme]
name = "dark-nebula"
```

## Multi-Agent Commands

Every Code's killer feature is multi-agent consensus/racing:

| Command | Description |
|---------|-------------|
| `/plan` | Claude + Gemini + GPT-5 create a consolidated plan (consensus) |
| `/solve` | Fastest-first race between models |
| `/code` | Multi-worktree implementation with consensus review |
| `/auto` | Auto Drive: multi-step task orchestration |

## Usage

### Command Line Interface

```bash
# Show help
code --help

# Run with a prompt
code "Implement a REST API for user management"

# Non-interactive mode (for CI/automation)
code --no-approval "Run tests and fix failures"

# Read-only mode
code --read-only "Analyze code quality"

# Use specific model
code --model claude-opus-4-5-20251101 "Review this PR"
```

### CTO Integration

For Play workflow, we recommend:

| Task Type | Command | Reasoning |
|-----------|---------|-----------|
| Planning | `/plan` | Consensus-based task breakdown |
| Implementation | Default or `/code` | Complex features benefit from multi-agent |
| Debugging | `/solve` | Fastest resolution via racing |
| Orchestration | `/auto` | Autonomous multi-step sequences |

## Building

This image is automatically built from the base runtime image and downloads the Every Code binary from GitHub releases.

### Build Arguments

- `BASE_IMAGE` - Base runtime image (default: `ghcr.io/5dlabs/runtime:latest`)
- `VERSION` - Every Code version tag (default: `latest`)
- `TARGETARCH` - Target architecture (amd64 or arm64)

### Manual Build

```bash
docker build -t code:local \
  --build-arg VERSION=v0.6.45 \
  .
```

## Memory Files

Every Code supports `AGENTS.md` or `CLAUDE.md` files for project context:

```markdown
# Project Context
This is a React TypeScript application with:
- Authentication via JWT
- PostgreSQL database
- Express.js backend

## Key files:
- `/src/auth/` - Authentication logic
- `/src/api/` - API client code
```

## Links

- [Every Code GitHub](https://github.com/just-every/code)
- [Original Codex CLI](https://github.com/openai/codex)
- [CTO Controller Code Adapter](../../crates/controller/src/cli/adapters/code.rs)
