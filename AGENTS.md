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
- **Forge** (Unreal/Godot) - Unreal Engine 5, C++, Blueprints, GDScript

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

### Fast Dev Image Builds (Bypass GitHub Actions)

For rapid iteration on intake/runtime changes without waiting for full CI:

#### Prerequisites

```bash
# Install cross-compilation tools (one-time)
just install-cross-tools

# Authenticate with GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
```

#### Quick Commands

| Command | Description |
|---------|-------------|
| `just dev-runtime-image` | Build runtime with local intake → push to `ghcr.io/5dlabs/runtime:dev` |
| `just dev-claude-image` | Build Claude image with local intake → push to `ghcr.io/5dlabs/claude:dev` |
| `just dev-image-local` | Build locally without pushing (for testing) |
| `just dev-runtime-all` | Build with all binaries (intake + pm-activity) |

#### How It Works

1. **Cross-compiles** intake binary for linux-x86_64 using `cargo-zigbuild` (~1-2 min on Mac)
2. **Creates overlay image** that replaces just the binary in existing runtime/claude image
3. **Pushes to GHCR** with `dev` tag

#### Using the Dev Image

```bash
# Option A: Test with a CodeRun
kubectl apply -f - <<EOF
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-dev-intake
  namespace: cto
spec:
  cli: claude
  prompt: "Run intake --version and report the output"
  image: ghcr.io/5dlabs/claude:dev
EOF

# Option B: Patch existing deployment
kubectl set image deployment/claude-agent claude=ghcr.io/5dlabs/claude:dev -n cto

# Option C: Set in cto-config.json for play workflows
# "agentImage": "ghcr.io/5dlabs/claude:dev"
```

#### Full Script Options

```bash
./scripts/build-dev-image.sh --help

# Examples:
./scripts/build-dev-image.sh --binary intake --image runtime --push
./scripts/build-dev-image.sh --binary all --image claude --tag my-feature --push
```

### Port Forwards (for MCP tools)

```bash
kubectl port-forward svc/prometheus-server -n observability 9090:80
kubectl port-forward svc/loki-gateway -n observability 3100:80
kubectl port-forward svc/grafana -n observability 3000:80
kubectl port-forward svc/argocd-server -n argocd 8080:80
kubectl port-forward svc/argo-workflows-server -n automation 2746:2746
```

### Local Development with launchd (Background Services)

For running CTO services in the background without a terminal window, use the launchd integration. Services auto-restart when you rebuild binaries.

#### Prerequisites

```bash
# Install fswatch (required for binary watching)
brew install fswatch

# Install cloudflared (for tunnel to receive webhooks)
brew install cloudflared

# Build release binaries
cargo build --release
```

#### Quick Start

```bash
# 1. Install and start all services (one-time setup)
just launchd-install

# 2. Verify everything is running
just launchd-status

# 3. Monitor logs in a TUI
just launchd-monitor
```

#### Management Commands

| Command | Description |
|---------|-------------|
| `just launchd-install` | Generate plist files and start all services |
| `just launchd-uninstall` | Stop and remove all services |
| `just launchd-status` | Show service status and health |
| `just launchd-logs` | Tail all service logs (raw) |
| `just launchd-monitor` | **Open lnav TUI** for log viewing with search/filter |
| `just launchd-multitail` | Split pane view of all logs |
| `just launchd-restart` | Restart all services manually |
| `just launchd-start` | Start services (if stopped) |
| `just launchd-stop` | Stop services (without unloading) |

#### Development Workflow

Once installed, the watcher monitors `target/release/` and auto-restarts services when binaries change:

```bash
# Normal development - just rebuild, services restart automatically
cargo build --release --bin agent-controller
# → watcher detects change → controller restarts automatically

# Rebuild healer - both healer AND healer-sensor restart
cargo build --release --bin healer
# → watcher restarts ai.5dlabs.cto.healer
# → watcher restarts ai.5dlabs.cto.healer-sensor

# Or use the convenience command to build all
just build-and-restart
```

#### Services Managed

| Service | Binary | Port | Health Endpoint | Description |
|---------|--------|------|-----------------|-------------|
| `ai.5dlabs.cto.controller` | `agent-controller` | 8080 | `/health` | CodeRun CRD orchestrator |
| `ai.5dlabs.cto.pm-server` | `pm-server` | 8081 | `/health` | Linear webhooks & PM |
| `ai.5dlabs.cto.healer` | `healer` | 8082 | `/health` | Self-healing monitor |
| `ai.5dlabs.cto.healer-sensor` | `healer` | - | - | GitHub Actions failure sensor |
| `ai.5dlabs.cto.tunnel` | `cloudflared` | - | pm-dev.5dlabs.ai | Cloudflare tunnel for webhooks |
| `ai.5dlabs.cto.watcher` | - | - | - | Binary change watcher |

#### Log Locations

| Path | Description |
|------|-------------|
| `/tmp/cto-launchd/controller.log` | Controller stdout |
| `/tmp/cto-launchd/pm-server.log` | PM Server stdout |
| `/tmp/cto-launchd/healer.log` | Healer stdout |
| `/tmp/cto-launchd/healer-sensor.log` | Healer Sensor stdout |
| `/tmp/cto-launchd/tunnel.log` | Cloudflare tunnel logs |
| `/tmp/cto-launchd/watcher.log` | Watcher events (shows restarts) |
| `/tmp/cto-launchd/*.err` | stderr for each service |

#### Monitoring with lnav (Recommended)

```bash
just launchd-monitor
```

**lnav keybindings:**
- `/` - search for text
- `n`/`N` - next/prev search result
- `e`/`E` - jump to next/prev error
- `:filter-in <pattern>` - show only matching lines
- `:filter-out <pattern>` - hide matching lines
- `Tab` - cycle through files
- `q` - quit

#### For AI Agents

When working on CTO platform code:

1. **Before starting work**, verify services are running:
   ```bash
   just launchd-status
   ```

2. **After making code changes**, rebuild the affected binary:
   ```bash
   cargo build --release --bin <binary-name>
   ```
   The watcher will automatically restart the service.

3. **To view logs** for debugging:
   ```bash
   # Quick tail
   tail -f /tmp/cto-launchd/controller.log
   
   # Or use the TUI
   just launchd-monitor
   ```

4. **If services aren't running** after a system restart:
   ```bash
   just launchd-start
   ```

5. **If things are broken**, full reinstall:
   ```bash
   just launchd-uninstall && just launchd-install
   ```

#### Troubleshooting

| Issue | Solution |
|-------|----------|
| Services not running after reboot | Run `just launchd-start` (services don't auto-start on login) |
| Service shows "loaded" but no PID | Check error log: `cat /tmp/cto-launchd/<service>.err` |
| Watcher not detecting changes | Reinstall: `just launchd-uninstall && just launchd-install` |
| Binary not found errors | Run `cargo build --release` first |
| Port already in use | Run `just kill-ports` then `just launchd-restart` |
| Need to update env vars | Reinstall to regenerate plists with new `.env.local` values |

#### Manual launchctl Commands (Advanced)

```bash
# List all CTO services
launchctl list | grep 5dlabs

# View a specific service's info
launchctl print gui/$(id -u)/ai.5dlabs.cto.controller

# Manually kick (restart) a service
launchctl kickstart -k gui/$(id -u)/ai.5dlabs.cto.controller

# View plist files
ls ~/Library/LaunchAgents/ai.5dlabs.cto.*.plist
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
