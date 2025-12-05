# Gemini CLI Reference

**Repository:** [github.com/google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli)  
**Package:** `@google/gemini-cli`  
**License:** Apache-2.0  
**Stars:** 85.8k+ ⭐

## Overview

Gemini CLI is an open-source AI agent that brings the power of Gemini directly into your terminal. It provides lightweight access to Gemini, giving you the most direct path from your prompt to the model.

## Why Gemini CLI?

- **🎯 Free tier**: 60 requests/min and 1,000 requests/day with personal Google account
- **🧠 Powerful Gemini 2.5 Pro**: Access to 1M token context window
- **🔧 Built-in tools**: Google Search grounding, file operations, shell commands, web fetching
- **🔌 Extensible**: MCP (Model Context Protocol) support for custom integrations
- **💻 Terminal-first**: Designed for developers who live in the command line
- **🛡️ Open source**: Apache 2.0 licensed

## Installation

### Prerequisites

- Node.js version 20 or higher
- macOS, Linux, or Windows

### Quick Install

```bash
# Using npx (no installation required)
npx https://github.com/google-gemini/gemini-cli

# Install globally with npm
npm install -g @google/gemini-cli

# Install with Homebrew (macOS/Linux)
brew install gemini-cli
```

### Release Tags

```bash
# Latest stable
npm install -g @google/gemini-cli@latest

# Preview (weekly, may contain regressions)
npm install -g @google/gemini-cli@preview

# Nightly (daily builds, experimental)
npm install -g @google/gemini-cli@nightly
```

## Authentication Options

### Option 1: Login with Google (OAuth)

**Best for:** Individual developers and Gemini Code Assist license holders.

```bash
# Start Gemini CLI and follow browser auth flow
gemini

# With Google Cloud Project (for paid licenses)
export GOOGLE_CLOUD_PROJECT="YOUR_PROJECT_ID"
gemini
```

**Benefits:**
- Free tier: 60 requests/min, 1,000 requests/day
- Gemini 2.5 Pro with 1M token context window
- No API key management

### Option 2: Gemini API Key

**Best for:** Developers who need specific model control or paid tier access.

```bash
export GEMINI_API_KEY="YOUR_API_KEY"
gemini
```

Get your key from [aistudio.google.com/apikey](https://aistudio.google.com/apikey).

### Option 3: Vertex AI

**Best for:** Enterprise teams and production workloads.

```bash
export GOOGLE_API_KEY="YOUR_API_KEY"
export GOOGLE_GENAI_USE_VERTEXAI=true
gemini
```

## Configuration

### Settings File

Configure Gemini CLI in `~/.gemini/settings.json`.

### Memory/Guidance File

Gemini CLI uses `GEMINI.md` files for persistent guidance and project context. Place a `GEMINI.md` file in your project root for project-specific instructions.

### MCP Servers

Configure MCP servers in `~/.gemini/settings.json`:

```json
{
  "mcpServers": {
    "github": {
      "command": "mcp-server-github",
      "env": { "GITHUB_TOKEN": "..." }
    }
  }
}
```

Usage:
```bash
> @github List my open pull requests
> @slack Send a summary of today's commits to #dev channel
```

## Key Features

### Code Understanding & Generation

- Query and edit large codebases
- Generate new apps from PDFs, images, or sketches using multimodal capabilities
- Debug issues and troubleshoot with natural language

### Automation & Integration

- Automate operational tasks like querying pull requests or handling complex rebases
- Use MCP servers to connect new capabilities
- Run non-interactively in scripts for workflow automation

### Advanced Capabilities

- Ground queries with built-in Google Search for real-time information
- Conversation checkpointing to save and resume complex sessions
- Custom context files (GEMINI.md) to tailor behavior for projects

## Usage Examples

### Basic Usage

```bash
# Start in current directory
gemini

# Include multiple directories
gemini --include-directories ../lib,../docs

# Use specific model
gemini -m gemini-2.5-flash
```

### Non-Interactive Mode

```bash
# Simple text response
gemini -p "Explain the architecture of this codebase"

# JSON output for scripting
gemini -p "Explain the architecture" --output-format json

# Stream JSON events for monitoring
gemini -p "Run tests and deploy" --output-format stream-json
```

### Project Examples

```bash
# Start a new project
cd new-project/
gemini
> Write me a Discord bot that answers questions using a FAQ.md file

# Analyze existing code
git clone https://github.com/google-gemini/gemini-cli
cd gemini-cli
gemini
> Give me a summary of all changes from yesterday
```

## CTO Platform Configuration

```json
{
  "cli": "gemini",
  "model": "gemini-2.5-pro",
  "cliConfig": {
    "model": "gemini-2.5-pro",
    "maxTokens": 64000
  }
}
```

## GitHub Integration

Integrate Gemini CLI into GitHub workflows with [Gemini CLI GitHub Action](https://github.com/google-github-actions/run-gemini-cli):

- **Pull Request Reviews**: Automated code review with contextual feedback
- **Issue Triage**: Automated labeling and prioritization
- **On-demand Assistance**: Mention `@gemini-cli` in issues and PRs
- **Custom Workflows**: Build automated workflows tailored to your needs

## Built-in Tools

- **File System Operations**: Read, write, and manage files
- **Shell Commands**: Execute terminal commands
- **Web Fetch & Search**: Fetch web content and search with Google

## Documentation

- [Official Documentation](https://geminicli.com/docs/)
- [Quickstart Guide](https://github.com/google-gemini/gemini-cli/blob/main/docs/get-started/index.md)
- [Authentication Setup](https://github.com/google-gemini/gemini-cli/blob/main/docs/get-started/authentication.md)
- [Configuration Guide](https://github.com/google-gemini/gemini-cli/blob/main/docs/get-started/configuration.md)
- [MCP Server Integration](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md)
- [Commands Reference](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/commands.md)
- [Headless Mode (Scripting)](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/headless.md)

## Architecture

Gemini CLI is built with TypeScript and uses:

- Ink library for terminal UI
- Tree-sitter for code parsing
- Native MCP support

## Community

- [GitHub Discussions](https://github.com/google-gemini/gemini-cli/discussions)
- [GitHub Issues](https://github.com/google-gemini/gemini-cli/issues)
- [Official Roadmap](https://github.com/orgs/google-gemini/projects/11)

