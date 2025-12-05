# Cursor Agent CLI Reference

**Documentation:** [cursor.com/docs/cli](https://cursor.com/docs/cli/overview)  
**Package:** `cursor-agent`  
**License:** Proprietary (Cursor)

## Overview

Cursor CLI lets you interact with AI agents directly from your terminal to write, review, and modify code. Whether you prefer an interactive terminal interface or print automation for scripts and CI pipelines, the CLI provides powerful coding assistance right where you work.

## Installation

```bash
curl https://cursor.com/install -fsS | bash
```

## Quick Start

```bash
# Run interactive session
cursor-agent

# Start with initial prompt
cursor-agent "refactor the auth module to use JWT tokens"
```

## Modes

### Interactive Mode

Start a conversational session with the agent to describe your goals, review proposed changes, and approve commands:

```bash
# Start interactive session
cursor-agent

# Start with initial prompt
cursor-agent "refactor the auth module to use JWT tokens"
```

### Non-Interactive Mode (Print Mode)

Use print mode for non-interactive scenarios like scripts, CI pipelines, or automation:

```bash
# Run with specific prompt and model
cursor-agent -p "find and fix performance issues" --model "gpt-5"

# Use with git changes included for review
cursor-agent -p "review these changes for security issues" --output-format text
```

## Session Management

Resume previous conversations to maintain context across multiple interactions:

```bash
# List all previous chats
cursor-agent ls

# Resume latest conversation
cursor-agent resume

# Resume specific conversation
cursor-agent --resume="chat-id-here"
```

## Available Models

| Model | CLI Identifier | Provider |
|-------|---------------|----------|
| Composer 1 | `composer-1` | Cursor |
| Auto | `auto` | Mixed |
| Claude 4.5 Sonnet | `sonnet-4.5` | Anthropic |
| Claude 4.5 Sonnet (Thinking) | `sonnet-4.5-thinking` | Anthropic |
| Claude 4.5 Opus | `opus-4.5` | Anthropic |
| Claude 4.5 Opus (Thinking) | `opus-4.5-thinking` | Anthropic |
| Gemini 3 Pro | `gemini-3-pro` | Google |
| GPT-5 | `gpt-5` | OpenAI |
| GPT-5.1 | `gpt-5.1` | OpenAI |
| GPT-5 High | `gpt-5-high` | OpenAI |
| GPT-5.1 High | `gpt-5.1-high` | OpenAI |
| GPT-5 Codex | `gpt-5-codex` | OpenAI |
| GPT-5 Codex High | `gpt-5-codex-high` | OpenAI |
| GPT-5.1 Codex | `gpt-5.1-codex` | OpenAI |
| GPT-5.1 Codex High | `gpt-5.1-codex-high` | OpenAI |
| Grok | `grok` | xAI |

## Model Selection Guide

### For General Tasks
- **Recommended:** `opus-4.5` or `auto`
- Best balanced performance for most coding tasks

### For Complex Reasoning
- **Recommended:** `opus-4.5-thinking` or `sonnet-4.5-thinking`
- Extended thinking capabilities for complex problems

### For Code Generation
- **Recommended:** `gpt-5.1-codex` or `gpt-5.1-codex-high`
- Optimized for code-specific tasks

### For Speed
- **Recommended:** `sonnet-4.5` or `auto`
- Fastest response times

### For Multi-Provider Coverage
- **Recommended rotation:** `opus-4.5`, `gemini-3-pro`, `gpt-5.1-codex`
- Ensures fallback across providers

## Configuration

Cursor Agent uses environment variables and CLI arguments for configuration.

### CTO Platform Configuration

```json
{
  "cli": "cursor",
  "model": "gpt-5-cursor",
  "cliConfig": {
    "model": "gpt-5-cursor",
    "maxTokens": 64000,
    "temperature": 0.7,
    "settings": {
      "sandboxMode": "danger-full-access",
      "approvalPolicy": "never",
      "editor": {
        "vimMode": true
      }
    }
  }
}
```

## MCP Integration

Cursor supports MCP (Model Context Protocol) servers. Configure MCP in your settings or via the CLI.

See [MCP documentation](https://cursor.com/docs/cli/mcp) for configuration details.

## Usage Examples

```bash
# Basic usage
cursor agent --model sonnet-4.5 "Write a hello world program"

# With extended thinking
cursor agent --model sonnet-4.5-thinking "Solve this complex algorithm problem"

# Using auto selection
cursor agent --model auto "Help me with this task"

# In print mode (scripting)
cursor agent --model gpt-5-codex --print "Generate a REST API"
```

## Verifying Available Models

```bash
# Interactive CLI
cursor agent
# Then type: /model

# Check help
cursor agent --help | grep -A 2 "model"
```

## Cloud Agents

Cursor also supports Cloud Agents (formerly Background Agents) for running agents in the cloud. See [Cloud Agents documentation](https://cursor.com/docs/cloud-agent).

## GitHub Integration

Cursor integrates with GitHub for:

- Pull request context and reviews
- Issue triage and implementation
- Automated workflows

See [GitHub Integration](https://cursor.com/docs/integrations/github).

## Shell Mode

Cursor CLI includes a shell mode for terminal-focused workflows. See [Shell Mode documentation](https://cursor.com/docs/cli/shell-mode).

## Headless Mode

For CI/CD and automation, use headless mode. See [Headless documentation](https://cursor.com/docs/cli/headless/overview).

## Documentation

- [CLI Overview](https://cursor.com/docs/cli/overview)
- [Installation](https://cursor.com/docs/cli/installation)
- [Using Agent in CLI](https://cursor.com/docs/cli/using)
- [Shell Mode](https://cursor.com/docs/cli/shell-mode)
- [MCP Integration](https://cursor.com/docs/cli/mcp)
- [Headless Mode](https://cursor.com/docs/cli/headless/overview)

## Legacy/Deprecated Models

The following models have been removed or deprecated:

- ❌ `cheetah` — Removed from Cursor CLI
- `sonnet-4` → Use `sonnet-4.5`
- `gpt-4o` → Use `gpt-5.1` or `gpt-5.1-codex`
- `opus-4.1` → Use `opus-4.5`

