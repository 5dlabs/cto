# Claude Code CLI Reference

**Repository:** [github.com/anthropics/claude-code](https://github.com/anthropics/claude-code)  
**Package:** `@anthropic-ai/claude-code`  
**License:** Proprietary (Anthropic)  
**Stars:** 44.6k+ ⭐

## Overview

Claude Code is an agentic coding tool that lives in your terminal, understands your codebase, and helps you code faster by executing routine tasks, explaining complex code, and handling git workflows — all through natural language commands. Use it in your terminal, IDE, or tag @claude on GitHub.

## Installation

### macOS/Linux (curl)

```bash
curl -fsSL https://claude.ai/install.sh | bash
```

### Homebrew (macOS)

```bash
brew install --cask claude-code
```

### Windows

```powershell
irm https://claude.ai/install.ps1 | iex
```

### NPM

```bash
npm install -g @anthropic-ai/claude-code
```

> **Note:** If installing with NPM, you also need to install [Node.js 18+](https://nodejs.org/en/download/)

## Quick Start

```bash
# Navigate to your project directory
cd /path/to/your/project

# Run Claude Code
claude
```

## Configuration

### Memory/Guidance File

Claude Code uses `CLAUDE.md` files for persistent guidance and project context. Place a `CLAUDE.md` file in your project root to provide Claude with project-specific instructions.

### MCP Integration

Claude Code has native MCP (Model Context Protocol) support. Configure MCP servers via the standard MCP configuration patterns. The `tools` CLI bridges STDIO ↔ HTTP for tool integration.

## Key Features

- **Agentic coding** — Executes routine tasks, explains complex code, handles git workflows
- **Natural language interface** — Communicate through natural language commands
- **Codebase understanding** — Maps and understands your entire codebase
- **Git integration** — Automatic commits with sensible messages
- **GitHub integration** — Tag @claude in issues and PRs
- **IDE support** — Use in terminal, IDE, or browser

## Available Models

| Model | Identifier |
|-------|------------|
| Claude Sonnet 4 | `claude-sonnet-4-20250514` |
| Claude Opus 4 | `claude-opus-4-1-20250805` |
| Claude Sonnet (alias) | `sonnet` |
| Claude Opus (alias) | `opus` |
| Claude Haiku (alias) | `haiku` |

## CTO Platform Configuration

```json
{
  "cli": "claude",
  "model": "claude-sonnet-4-20250514"
}
```

## Plugins

Claude Code supports plugins that extend functionality with custom commands and agents. See the [plugins directory](https://github.com/anthropics/claude-code/blob/main/plugins/README.md) for available plugins.

## Documentation

- [Official Documentation](https://docs.anthropic.com/en/docs/claude-code/overview)
- [CLI Reference](https://code.claude.com/docs/en/cli-reference)
- [Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)

## Data & Privacy

When you use Claude Code, Anthropic collects feedback including usage data (code acceptance/rejections), associated conversation data, and user feedback submitted via the `/bug` command.

See [data usage policies](https://docs.anthropic.com/en/docs/claude-code/data-usage) for full details.

## Community

- [Claude Developers Discord](https://anthropic.com/discord)
- [GitHub Issues](https://github.com/anthropics/claude-code/issues)
- Use `/bug` command to report issues directly within Claude Code

