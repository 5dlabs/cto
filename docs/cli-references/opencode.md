# OpenCode CLI Reference

**Repository:** [github.com/sst/opencode](https://github.com/sst/opencode)  
**Package:** `opencode-ai`  
**License:** MIT  
**Stars:** 35.5k+ ⭐
**Website:** [opencode.ai](https://opencode.ai/)

## Overview

OpenCode is the AI coding agent built for the terminal. It's 100% open source, provider-agnostic, and built by neovim users and the creators of [terminal.shop](https://terminal.shop/). It features a client/server architecture that allows remote control (e.g., from a mobile app).

## Key Differences from Claude Code

- 100% open source
- Not coupled to any provider (Claude, OpenAI, Google, or local models)
- Out of the box LSP support
- Focus on TUI (Terminal UI)
- Client/server architecture for remote control

## Installation

### Quick Install (curl)

```bash
curl -fsSL https://opencode.ai/install | bash
```

### Package Managers

```bash
# npm/bun/pnpm/yarn
npm i -g opencode-ai@latest

# Windows (Scoop)
scoop bucket add extras; scoop install extras/opencode

# Windows (Chocolatey)
choco install opencode

# macOS/Linux (Homebrew)
brew install opencode

# Arch Linux
paru -S opencode-bin

# Any OS (mise)
mise use --pin -g ubi:sst/opencode

# Nix
nix run nixpkgs#opencode  # or github:sst/opencode for latest dev
```

### Installation Directory

The install script respects these priorities:

1. `$OPENCODE_INSTALL_DIR` — Custom installation directory
2. `$XDG_BIN_DIR` — XDG Base Directory Specification path
3. `$HOME/bin` — Standard user binary directory
4. `$HOME/.opencode/bin` — Default fallback

```bash
# Examples
OPENCODE_INSTALL_DIR=/usr/local/bin curl -fsSL https://opencode.ai/install | bash
XDG_BIN_DIR=$HOME/.local/bin curl -fsSL https://opencode.ai/install | bash
```

## Configuration

OpenCode uses JSON/JSONC configuration files:

- Project config: `opencode.json` or `opencode.jsonc`
- Global config: `~/.opencode/config.json`

### Memory/Guidance File

OpenCode uses `AGENTS.md` files (same as Codex) for persistent guidance and project context.

## Built-in Agents

OpenCode includes two built-in agents you can switch between using the `Tab` key:

| Agent | Description |
|-------|-------------|
| **build** | Default, full access agent for development work |
| **plan** | Read-only agent for analysis and code exploration. Denies file edits by default, asks permission before running bash commands. Ideal for exploring unfamiliar codebases or planning changes. |

### Subagents

- **general** — For complex searches and multi-step tasks. Used internally and can be invoked using `@general` in messages.

## Key Features

- **Provider-agnostic** — Works with Claude, OpenAI, Google, or local models
- **LSP support** — Built-in Language Server Protocol integration
- **TUI focus** — Rich terminal interface built by neovim users
- **Client/server architecture** — Drive remotely from mobile or other clients
- **OpenCode Zen** — Optional managed model service from [opencode.ai/zen](https://opencode.ai/zen)

## CTO Platform Configuration

```json
{
  "cli": "opencode",
  "model": "opencode-sonnet",
  "cliConfig": {
    "model": "opencode-sonnet",
    "maxTokens": 16384,
    "temperature": 0.65,
    "instructions": "Follow OpenCode best practices",
    "provider": {
      "name": "anthropic",
      "envKey": "ANTHROPIC_API_KEY"
    }
  }
}
```

## Provider Configuration

OpenCode supports multiple providers:

### Anthropic (Claude)

```json
{
  "provider": {
    "name": "anthropic",
    "envKey": "ANTHROPIC_API_KEY"
  }
}
```

### OpenAI

```json
{
  "provider": {
    "name": "openai",
    "envKey": "OPENAI_API_KEY"
  }
}
```

### OpenCode Zen (Managed)

Use the recommended models provided through [OpenCode Zen](https://opencode.ai/zen).

## Documentation

- [Official Documentation](https://opencode.ai/docs)
- [Agents Documentation](https://opencode.ai/docs/agents)
- [Contributing Guide](https://github.com/sst/opencode/blob/dev/CONTRIBUTING.md)
- [Style Guide](https://github.com/sst/opencode/blob/dev/STYLE_GUIDE.md)

## Architecture

OpenCode is built with:

- TypeScript (62.5%)
- Python (13.0%)
- Go (10.4%)
- CSS (7.6%)

The monorepo structure includes:

- `packages/` — Core packages
- `sdks/vscode/` — VS Code extension
- `specs/` — Specifications
- `infra/` — Infrastructure (SST-based)

## Community

- [Discord](https://opencode.ai/discord)
- [X.com](https://x.com/opencode)
- [GitHub Discussions](https://github.com/sst/opencode/discussions)

