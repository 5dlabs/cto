# Agent Guidelines

This document provides essential context for AI agents working on the CTO platform. For detailed documentation, see the linked files in `docs/`.

## Git Workflow

| Setting | Value | Notes |
|---------|-------|-------|
| **Base Branch** | `develop` | All PRs should target `develop`, not `main` |
| **Release Branch** | `main` | Protected, releases only |
| **Feature Branches** | `feat/<name>` | Branch from `develop` |
| **Bugfix Branches** | `fix/<name>` | Branch from `develop` |

**Important:** Always create new branches from `develop`:
```bash
git checkout develop
git pull origin develop
git checkout -b feat/my-feature
```

---

## Documentation Index

### Core Workflows

| Document | Description |
|----------|-------------|
| **[Play Workflow](docs/play-workflow.md)** | Multi-agent orchestration from PRD to shipped features |
| **[Lifecycle Verification](docs/lifecycle-verification.md)** | MCP tools for verifying each workflow stage |

### Platform & Infrastructure

| Document | Description |
|----------|-------------|
| **[Platform Infrastructure](docs/platform-infrastructure.md)** | Deployed infrastructure, operators, and services |
| **[MCP Tools Reference](docs/mcp-tools.md)** | All available MCP servers and tools |

### Development

| Document | Description |
|----------|-------------|
| **[Development Guide](docs/development-guide.md)** | Build commands, Tilt setup, coding style |
| **[CLI Reference](docs/cli-reference.md)** | Agent CLIs, tools configuration, non-interactive mode |
| **[Secrets Management](docs/secrets-management.md)** | 1Password, OpenBao, credential management |

### Operations

| Document | Description |
|----------|-------------|
| **[Troubleshooting](docs/troubleshooting.md)** | Known issues, debugging, Healer monitoring |
| **[Context Engineering](docs/context-engineering.md)** | Best practices for agent context management |
| **[Prompt Style Variants](docs/prompt-style-variants.md)** | A/B testing minimal vs standard prompts (Ralph-inspired) |

### Business & Strategy

| Document | Description |
|----------|-------------|
| **[SaaS Architecture](docs/saas-architecture.md)** | Multi-tenant SaaS platform design, shared integrations |
| **[SaaS Monetization](docs/saas-monetization.md)** | Pricing models, tiers, revenue streams |
| **[Open Source Strategy](docs/open-source-strategy.md)** | Open-core model, OSS vs SaaS feature split |

### Additional Resources

| Resource | Description |
|----------|-------------|
| **[Play Workflow Guide](docs/play-workflow-guide.html)** | Interactive flow diagram and acceptance criteria |
| **[MCP Server Binaries](docs/MCP-SERVERS-BINARIES.md)** | Building MCP servers as standalone binaries |
| **[Headscale Setup](docs/HEADSCALE-CLIENT-SETUP.md)** | VPN client configuration |

---

## Quick Reference

### Agents Overview

**Implementation Agents:**
- **Rex** (Rust) - axum, tokio, serde, sqlx
- **Grizz** (Go) - chi, grpc, pgx, redis
- **Nova** (Node.js/Bun) - Elysia, Effect, Better Auth, Drizzle
- **Blaze** (React/TS) - Next.js 15, shadcn/ui, Better Auth, TailwindCSS
- **Tap** (Expo) - expo-router, react-native, Better Auth
- **Spark** (Electron) - electron-builder, react, Better Auth
- **Vex** (Unity/C#) - XR Interaction Toolkit, OpenXR, Meta XR SDK

**Support Agents:**
- **Morgan** - Project management, PRD intake
- **Bolt** - Infrastructure setup (Task 1)
- **Cleo** - Code quality review
- **Cipher** - Security analysis
- **Tess** - Testing
- **Atlas** - Integration and merge (CI + merge only)

### Play Workflow Flow

```
PRD → Intake (Morgan) → Infrastructure (Bolt) → Implementation (Rex/Blaze) 
    → Quality (Cleo) → Security (Cipher) → Testing (Tess) → Merge (Atlas) → Done
         ↑                    |
         └── retry with fresh start (after N attempts, clears context)
```

### Cursor-Inspired Improvements

Based on [Cursor's research](https://cursor.com/blog/scaling-autonomous-coding) on scaling long-running autonomous coding agents (January 2026):

| Feature | Description | Config |
|---------|-------------|--------|
| **Fresh Start** | After N retries, clears context to combat drift and tunnel vision | `freshStartThreshold: 3` |
| **Simplified Atlas** | Merge-only role - workers handle their own conflicts | Default behavior |

#### Fresh Start Mechanism

When acceptance criteria aren't met after `freshStartThreshold` retries (default: 3):
1. Context files are cleared (`.conversation_id`, `.session_state`, `.agent_context`)
2. Agent restarts with only the task definition
3. Model rotation continues to try different approaches

This combats:
- Tunnel vision from accumulated context
- Risk-averse behavior patterns
- Context saturation causing confusion

Configure in `cto-config.json`:

```json
{
  "defaults": {
    "play": {
      "freshStartThreshold": 3
    }
  }
}
```

**Note:** Per-agent model configuration is done in the `agents` section of `cto-config.json`, not via `roleModels`.

### Key MCP Tools

| Tool | Purpose |
|------|---------|
| `intake` | Process PRD to generate tasks |
| `play` | Submit multi-agent workflow |
| `play_status` | Query workflow progress |
| `jobs` | List running workflows |
| `stop_job` | Cancel running workflow |

### Research & Code Quality Tools

| Tool | Purpose |
|------|---------|
| **Context7** | Library documentation lookup (prevents hallucinated APIs) |
| **OctoCode** | Semantic code search across GitHub for real implementations |
| **Firecrawl** | Web research and competitive analysis |

#### OctoCode Usage

OctoCode provides semantic code search and specialized review commands:

| Tool/Command | Purpose | Best For |
|--------------|---------|----------|
| `octocode_githubSearchCode` | Search code across repos | Finding implementation patterns |
| `octocode_githubSearchRepositories` | Discover repos by topic/stars | Finding reference projects |
| `octocode_githubSearchPullRequests` | Search PRs with diffs | Learning how issues were fixed |
| `/research` command | Deep code discovery | Morgan intake research |
| `/review_pull_request` command | Defects-first PR analysis | Cleo code quality reviews |
| `/review_security` command | Security audit with evidence | Cipher security analysis |

**When to use which tool:**
- **Context7** → "What's the API for Effect Schema?" (documentation)
- **OctoCode** → "How does React implement useState?" (source code)
- **Firecrawl** → "How do competitors handle this?" (web research)

### MCP Tool Usage Guidelines

#### `intake` Tool - Production Flow Only

⚠️ **NEVER use `local=true`** for the intake tool. Always use the production flow:

```
intake(project_name="my-project")  ← Correct (local defaults to false)
intake(project_name="my-project", local=true)  ← WRONG - bypasses production
```

**Production Intake Flow:**
1. MCP `intake()` creates Linear Project + PRD Issue
2. PRD content → issue description
3. `architecture.md` + `cto-config.json` → attachments
4. Morgan auto-assigned as delegate
5. PM Server webhook triggers Argo Workflow **in-cluster**
6. Actual intake processing runs in Kubernetes (not locally)

The `local=true` mode exists only for debugging the intake binary itself. It:
- Skips Linear entirely
- Runs the binary on your local machine
- Requires local API keys in `.env.local`
- Does NOT test the production workflow

### Build Commands

```bash
# Rust
cargo build --release
cargo test
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# GitOps
make -C infra/gitops validate

# Pre-commit
pre-commit run --all-files
```

### Port Forwards (for MCP tools)

```bash
kubectl port-forward svc/prometheus-server -n observability 9090:80
kubectl port-forward svc/loki-gateway -n observability 3100:80
kubectl port-forward svc/grafana -n observability 3000:80
kubectl port-forward svc/argocd-server -n argocd 8080:80
kubectl port-forward svc/argo-workflows-server -n automation 2746:2746
```

---

## Coding Style

- **Rust:** rustfmt (Edition 2021, `max_width=100`); prefer `tracing::*` over `println!`
- **YAML:** 2-space indent; begin docs with `---`
- **Markdown:** follow `.markdownlint.yaml`
- **Shell:** `bash -euo pipefail`; validate with ShellCheck

## Commit Guidelines

- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `refactor:`
- PRs must include: summary, rationale, scope, verification steps

### Pre-Push Requirements (MANDATORY)

**Before pushing code to origin or creating a pull request, you MUST run ALL of the following and ensure they pass with zero warnings/errors:**

1. **Format check:** `cargo fmt --all --check`
2. **Clippy Pedantic:** `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`
3. **Tests:** `cargo test`
4. **Pre-commit hooks:** `pre-commit run --all-files`

⚠️ **CRITICAL:** Never push code or open a PR without running Clippy in pedantic mode. The `-W clippy::pedantic` flag enables additional lints that catch common mistakes and enforce best practices. All pedantic warnings must be resolved before code is pushed.

## Security

- Never commit secrets
- Use `cto-config.json` for local configuration
- See [Secrets Management](docs/secrets-management.md) for credential handling
