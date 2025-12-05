# Factory Droid CLI Reference

**Repository:** [github.com/Factory-AI/factory](https://github.com/Factory-AI/factory)  
**Documentation:** [docs.factory.ai](https://docs.factory.ai/)  
**Website:** [factory.ai](https://factory.ai/)  
**License:** Proprietary (Factory AI)  
**Stars:** 327+ ⭐

## Overview

Factory is the agent-native development platform. It works across CLI, Web, Slack/Teams, Linear/Jira, and Mobile. The agent, **Droid**, is top performing in terminal benchmarks.

## Key Features

- **Multi-platform** — CLI, Web, Slack/Teams, Linear/Jira, Mobile
- **Top-performing agent** — Droid leads terminal benchmarks
- **CI/CD automation** — Automate builds, refactors, migrations
- **Failure diagnosis** — AI diagnoses failures, fixes tests, maintains code
- **GitHub integration** — Workflows for issues and PRs

## Installation

### CLI Quickstart

See [CLI Quickstart](https://docs.factory.ai/cli/getting-started/quickstart) for detailed installation.

### VS Code Extension

Install from [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=Factory.factory-vscode-extension).

## Authentication

Factory requires an API key:

```bash
export FACTORY_API_KEY="your-api-key"
droid
```

## CTO Platform Configuration

```json
{
  "cli": "factory",
  "model": "gpt-5-factory-high",
  "cliConfig": {
    "model": "gpt-5-factory-high",
    "maxTokens": 64000,
    "settings": {
      "sandboxMode": "danger-full-access",
      "approvalPolicy": "never",
      "reasoningEffort": "high"
    }
  }
}
```

> **Note:** Ensure `FACTORY_API_KEY` is mounted for Factory runs.

## Available Models

| Model | Identifier | Description |
|-------|------------|-------------|
| Factory High | `gpt-5-factory-high` | High reasoning effort |
| Factory Standard | `gpt-5-factory` | Standard reasoning |

## GitHub Workflows

Factory provides GitHub Actions workflows:

- `droid.yml` — Main Droid workflow
- `droid-review.yml` — Code review workflow

### Using Factory in GitHub Actions

```yaml
name: Droid
on:
  issues:
    types: [opened, labeled]
  pull_request:
    types: [opened, synchronize]

jobs:
  droid:
    runs-on: ubuntu-latest
    steps:
      - uses: Factory-AI/factory/.github/workflows/droid.yml@main
        with:
          api_key: ${{ secrets.FACTORY_API_KEY }}
```

## CLI Overview

The Droid CLI provides:

- Interactive terminal UI
- Automated code changes
- Test fixing
- Refactoring assistance
- Migration support

See [CLI Overview](https://docs.factory.ai/cli/getting-started/overview) for full documentation.

## Community Builds

Community integrations and examples:

- [factory-mcp](https://github.com/iannuttall/factory-mcp) — MCP integration to search Factory docs
- [Factory CLI with Claude/Codex via CLIProxyAPI](https://gist.github.com/chandika/c4b64c5b8f5e29f6112021d46c159fdd)
- [GrayPane Flight Search](https://github.com/punitarani/flights-tracker) — Example project built with Factory

## Use Cases

### CI/CD Automation

Factory Droids run refactors, migrations, and builds at scale.

### Failure Diagnosis

Agents diagnose failures, fix tests, and maintain code automatically.

### Code Review

Automated code review with contextual feedback.

## Documentation

- [CLI Quickstart](https://docs.factory.ai/cli/getting-started/quickstart)
- [CLI Overview](https://docs.factory.ai/cli/getting-started/overview)
- [Full Documentation](https://docs.factory.ai/)
- [Community Builds](https://github.com/Factory-AI/factory/blob/main/community-builds.md)

## Community

- [GitHub Discussions](https://github.com/Factory-AI/factory/discussions)
- [GitHub Issues](https://github.com/Factory-AI/factory/issues)

