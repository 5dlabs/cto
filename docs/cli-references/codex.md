# OpenAI Codex CLI Reference

**Repository:** [github.com/openai/codex](https://github.com/openai/codex)  
**Package:** `@openai/codex`  
**License:** Apache-2.0  
**Stars:** 51.9k+ ⭐

## Overview

Codex CLI is a lightweight coding agent from OpenAI that runs locally on your computer. It provides an interactive terminal UI for AI-assisted coding and can be used for automation via scripts and CI pipelines.

> **Note:** If you want Codex in your code editor (VS Code, Cursor, Windsurf), [install in your IDE](https://developers.openai.com/codex/ide). For the cloud-based agent, **Codex Web**, go to [chatgpt.com/codex](https://chatgpt.com/codex).

## Installation

### NPM (recommended)

```bash
npm install -g @openai/codex
```

### Homebrew (macOS)

```bash
brew install --cask codex
```

### Binary Downloads

Download from [GitHub Releases](https://github.com/openai/codex/releases/latest):

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `codex-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `codex-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `codex-x86_64-unknown-linux-musl.tar.gz` |
| Linux (arm64) | `codex-aarch64-unknown-linux-musl.tar.gz` |

## Quick Start

```bash
# Run Codex
codex

# Sign in with ChatGPT (recommended)
# Select "Sign in with ChatGPT" when prompted
```

## Authentication

### ChatGPT Plan (Recommended)

Sign into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan.

### API Key

For usage-based billing, see [authentication docs](https://github.com/openai/codex/blob/main/docs/authentication.md#usage-based-billing-alternative-use-an-openai-api-key).

## Configuration

Codex uses TOML-based configuration stored in `~/.codex/config.toml`.

### Example Config

```toml
model = "gpt-5-codex"
model_provider = "openai"

[mcp_servers]
[mcp_servers.tools]
command = ["tools", "--url", "http://localhost:8080"]

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
wire_api = "chat"

[tui]
disable_mouse_capture = false
```

## Memory/Guidance File

Codex uses layered `AGENTS.md` files for persistent guidance. Place an `AGENTS.md` file in your project root for project-specific instructions.

## MCP Integration

Codex supports STDIO-only MCP clients. Configure MCP servers in your `config.toml`:

```toml
[mcp_servers.my_server]
command = ["my-mcp-server"]
env = { API_KEY = "..." }
```

## Key Features

- **Interactive TUI** — Rich terminal interface with mouse support
- **Approval policies** — Control what Codex can execute automatically
- **Sandbox execution** — Safe command execution environment
- **MCP support** — Extend with Model Context Protocol servers
- **Session management** — Resume previous conversations
- **Non-interactive mode** — Script and CI/CD automation

## Available Models

| Model | Identifier |
|-------|------------|
| GPT-5 Codex | `gpt-5-codex` |
| GPT-5 | `gpt-5` |
| O3 | `o3` |
| O1-mini | `o1-mini` |

## CTO Platform Configuration

```json
{
  "cli": "codex",
  "model": "gpt-5-codex",
  "cliConfig": {
    "model": "gpt-5-codex",
    "maxTokens": 64000,
    "temperature": 0.7,
    "approvalPolicy": "on-request",
    "sandboxPreset": "workspace-write",
    "settings": {
      "reasoningEffort": "medium"
    }
  }
}
```

## Execpolicy

Configure rules that govern what commands Codex can execute. See [Execpolicy quickstart](https://github.com/openai/codex/blob/main/docs/execpolicy.md).

## Usage Examples

### Interactive Mode

```bash
# Start interactive session
codex

# With initial prompt
codex "refactor the auth module"
```

### Non-Interactive Mode

```bash
# Single prompt execution
codex exec "fix the failing tests"

# With specific model
codex -m gpt-5-codex "add input validation"
```

### Session Management

```bash
# List sessions
codex ls

# Resume latest
codex resume

# Resume specific session
codex --resume="chat-id-here"
```

## Documentation

- [Getting Started](https://github.com/openai/codex/blob/main/docs/getting-started.md)
- [Configuration](https://github.com/openai/codex/blob/main/docs/config.md)
- [Sandbox & Approvals](https://github.com/openai/codex/blob/main/docs/sandbox.md)
- [Authentication](https://github.com/openai/codex/blob/main/docs/authentication.md)
- [TypeScript SDK](https://github.com/openai/codex/blob/main/sdk/typescript/README.md)
- [GitHub Action](https://github.com/openai/codex-action)
- [FAQ](https://github.com/openai/codex/blob/main/docs/faq.md)

## Architecture

Codex is built in Rust and distributed via npm. The codebase consists of:

- `codex-cli/` — TypeScript CLI wrapper
- `codex-rs/` — Core Rust implementation
- `shell-tool-mcp/` — MCP server for shell commands
- `sdk/typescript/` — TypeScript SDK for automation

## Community

- [OpenAI Developer Forum](https://community.openai.com)
- [GitHub Issues](https://github.com/openai/codex/issues)

