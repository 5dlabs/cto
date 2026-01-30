# CTO Lite Tools Reference

## Repository Paths

- **Worktree:** `/Users/jonathonfritz/clawd-ctolite`
- **Main Repo:** `/Users/jonathonfritz/cto`
- **Branch:** `ctolite/implementation`

## Build Commands

### Rust (Backend)

```bash
# Build all crates
cargo build --release

# Build specific crate (when it exists)
cargo build -p cto-lite-tauri --release

# Run tests
cargo test

# Run Clippy pedantic
cargo clippy --all-targets -- -D warnings

# Format check
cargo fmt --check
```

### Tauri (Desktop App)

```bash
# Development (when set up)
cd crates/cto-lite/tauri
npm run tauri dev

# Production build
npm run tauri build

# Build specific platform
npm run tauri build -- --target universal-apple-darwin  # macOS
npm run tauri build -- --target x86_64-pc-windows-msvc  # Windows
npm run tauri build -- --target x86_64-unknown-linux-gnu  # Linux
```

### Frontend (React)

```bash
cd crates/cto-lite/ui

# Install dependencies
npm install

# Development server
npm run dev

# Build
npm run build

# Type check
npm run typecheck

# Lint
npm run lint
```

## Skills Reference

### Relevant Skills (In-Scope for Lite)

**Frontend:**
- `shadcn-stack` - Next.js + shadcn/ui patterns
- `tanstack-stack` - Client-first reactive patterns
- `react-best-practices` - Vercel React optimization (45 rules)
- `frontend-excellence` - Production-grade UI design
- `frontend-stack-selection` - Decision framework

**Backend:**
- `rust-patterns` - Axum, Tokio, anyhow/thiserror
- `rust-error-handling` - Error handling patterns
- `effect-patterns` - Effect TypeScript (for Nova)
- `elysia` - ElysiaJS with Bun (for Nova)
- `hono` - Hono web framework
- `bun` - Bun runtime documentation

**Desktop:**
- `electron-patterns` - Desktop patterns (reference for Tauri)
- `cloudflare-workers` - Cloudflared tunnel integration
- `cloudflare-durable-objects` - Edge state patterns

**Quality:**
- `testing-strategies` - Unit, integration, E2E patterns
- `playwright-testing` - E2E testing
- `webapp-testing` - Web app testing patterns
- `code-review` - Code quality patterns
- `code-maturity` - 9-category assessment

**Workflow:**
- `compound-engineering` - Plan→Work→Review→Compound
- `git-integration` - Git and GitHub patterns
- `verification-before-completion` - Evidence-based completion
- `writing-plans` - Implementation planning
- `executing-plans` - Task execution

**Tools:**
- `context7` - Up-to-date library documentation
- `github-mcp` - GitHub operations
- `mcp-development` - MCP server patterns
- `mcp-builder` - Building MCP servers
- `llm-docs` - LLM-optimized documentation

**Documentation:**
- `zod` - Schema validation
- `vitest` - Testing framework
- `trpc` - Type-safe APIs
- `turborepo` - Monorepo build system

### Excluded Skills (Enterprise Only)

- `kubernetes-operators` - K8s operators
- `healer` - Self-healing patterns
- `argo-events` - Event handling
- `argo-workflows` - Workflow orchestration
- `argocd-gitops` - GitOps patterns
- `storage-operators` - Mayastor/SeaweedFS
- `observability` - Prometheus/Loki/Grafana
- `secrets-management` - External Secrets/OpenBao

## Sub-Agents

The CTO Lite agent can delegate work to specialized sub-agents:

### Implementation Sub-Agents

| Agent | Purpose | Use When |
|-------|---------|----------|
| `rust-patterns` | Rust backend work | Building Tauri backend, controller |
| `shadcn-stack` | React frontend work | Building setup wizard, dashboard |
| `effect-patterns` | TypeScript patterns | Building MCP server, type-safe code |

### Quality Sub-Agents

| Agent | Purpose | Use When |
|-------|---------|----------|
| `code-review` | Review code changes | Before committing significant changes |
| `testing-strategies` | Test design | Setting up test suites |
| `security-analysis` | Security review | Before shipping |

### Documentation Sub-Agents

| Agent | Purpose | Use When |
|-------|---------|----------|
| `doc-coauthoring` | Write documentation | Creating user docs |
| `context7` | Fetch latest docs | Need current library APIs |

## GitHub CLI

```bash
# Create PR
gh pr create --title "feat(cto-lite): ..." --body "..."

# List PRs
gh pr list

# Check CI status
gh pr checks

# Merge PR
gh pr merge --squash
```

## Docker/Kind Commands

```bash
# Create Kind cluster
kind create cluster --name cto-lite

# Delete cluster
kind delete cluster --name cto-lite

# Load image into Kind
kind load docker-image ghcr.io/5dlabs/cto-lite-controller:v1.0 --name cto-lite

# Check cluster
kubectl get pods -A
```

## Helm Commands

```bash
# Template chart (validate)
helm template cto-lite infra/charts/cto-lite

# Install chart
helm install cto-lite infra/charts/cto-lite -n cto-lite --create-namespace

# Upgrade
helm upgrade cto-lite infra/charts/cto-lite -n cto-lite
```

## Key Environment Variables

```bash
# Development
export CTO_LITE=true
export RUST_LOG=debug

# API keys (from keychain in production)
export ANTHROPIC_API_KEY=sk-ant-...
export OPENAI_API_KEY=sk-proj-...
```

## Testing Strategy

1. **Unit Tests:** Each module has `#[cfg(test)]` tests
2. **Integration Tests:** `tests/` directory for cross-module tests
3. **E2E Tests:** Playwright for UI testing
4. **Manual Testing:** Native app on each platform
