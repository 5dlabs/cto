# Codebase Context

## 1. Repository Overview

- **URL**: https://github.com/5dlabs/cto
- **Primary Languages**: Rust, TypeScript
- **Frameworks**:
  - Rust: Custom Kubernetes controller using `kube-rs`, Tauri desktop app, Anchor (referenced in PRD but not yet in repo)
  - TypeScript: Next.js (apps/web, apps/website, apps/pitch-deck), Discord.js (apps/discord-bridge), Bun runtime (apps/intake-agent, apps/grok), Deno (apps/cto-tools)
- **Build Systems**: Cargo workspaces (Rust), npm/bun/deno (TypeScript), GitHub Actions CI/CD
- **Layout**: Monorepo with `crates/` (Rust), `apps/` (TypeScript services), `infra/` (Helm charts, K8s manifests), `shared/` (cross-app code), `intake/` (workflow definitions)
- **Runtime Versions Observed**: Node 20, Bun 1.1+, Deno (latest), Rust edition 2021, TypeScript 5.x

## 2. Service Architecture

### Relevant Services for PRD

| Service | Path | Language/Framework | Purpose | Ports/Transport |
|---------|------|--------------------|---------|-----------------|
| CTO Controller | `crates/controller/` | Rust (kube-rs) | Kubernetes operator managing CodeRun CRDs, task lifecycle, agent orchestration | K8s API, in-cluster |
| CTO CLI | `crates/cli/` | Rust | CLI adapter for multiple AI coding agents (Claude, Codex, Cursor, etc.) | Local process |
| CTO Config | `crates/config/` | Rust | Agent configuration, tool definitions, type system | Library crate |
| CTO Tools (MCP) | `apps/cto-tools/` | TypeScript (Deno) | MCP tool-calling runtime + codegen for agent SDK | HTTP JSON-RPC |
| Discord Bridge | `apps/discord-bridge/` | TypeScript (Node 20) | Human-in-the-loop elicitation via Discord | HTTP :3200 |
| Linear Bridge | `apps/linear-bridge/` | TypeScript (Node 20) | Human-in-the-loop elicitation via Linear | HTTP :3100 |
| Intake Agent | `apps/intake-agent/` | TypeScript (Bun) | PRD parsing, design intake, research operations | Stdin/stdout JSON protocol |
| Grok MCP Server | `apps/grok/` | TypeScript (Bun) | X/Grok search MCP server for investor/CTO research | stdio MCP |
| Web App | `apps/web/` | TypeScript (Next.js) | Customer dashboard, onboarding, auth | HTTP (Cloudflare) |
| CTO Lite (Tauri) | `crates/cto-lite/tauri/` | Rust (Tauri) + TypeScript | Desktop app for local CTO operation | Local app |

### Services NOT Directly Relevant
- `apps/pitch-deck/`, `apps/website/`, `apps/lobster-voice/`, `apps/nats-messenger/`, `avatar/` — not touched by this PRD.

## 3. API Contracts

### CTO Controller — Internal K8s API
- **CRDs**: `BoltRun` (`crates/controller/src/crds/boltrun.rs`), `CodeRun` (`crates/controller/src/crds/coderun.rs`)
- **Task Lifecycle**: Controller reconciles CodeRun CRDs through states. Terminal states trigger cleanup in `crates/controller/src/tasks/code/controller.rs`.
- **Auth**: K8s RBAC, service accounts

### Settlement Hook Integration Points (from PRD)
The PRD specifies these files in the CTO repo as integration targets:

| File | Purpose |
|------|---------|
| `crates/controller/src/tasks/config.rs` | Add Solana RPC endpoint + operator keypair path |
| `crates/controller/src/tasks/code/controller.rs` | After terminal state: compute bill → build receipt → submit `settle_task` |
| `crates/controller/src/tasks/code/status.rs` | CodeRun status updates — add `on_chain_settlement_sig` field |
| `crates/controller/src/tasks/code/resources.rs` | Pod resource construction — metering source for billing |

### MCP Tool Protocol (`apps/cto-tools/mcp.ts`)
- JSON-RPC 2.0 over HTTP POST
- `tools/list` and `tools/call` methods
- Routing: local sidecar (`LOCAL_TOOLS_URL`) vs remote (`TOOLS_SERVER_URL`) based on tool prefix
- Custom headers: `X-Agent-Id`, `X-Agent-Prewarm`
- Retry with exponential backoff (1s, 2s, 4s)

### Discord Bridge HTTP API (`apps/discord-bridge/src/http-server.ts`)
- `POST /notify` — Agent message notification
- `POST /elicitation` — Human-in-the-loop decision rendering
- `POST /elicitation/cancel` — Cross-cancel
- `POST /design-review` — Design variant selection
- `GET /health` — Health check
- `GET /history/decisions`, `/history/sessions`, `/history/waiting`, `/history/decision-audit`, `/history/design`
- `GET /elicitation/status/:id` — Status check

## 4. Data Models

### Rust CRDs (Kubernetes)
- **Database engine**: Kubernetes etcd (CRD storage)
- **ORM**: `kube-rs` derive macros (`CustomResource`)
- **Key CRDs**:
  - `CodeRun` — `crates/controller/src/crds/coderun.rs` — represents a task execution
  - `BoltRun` — `crates/controller/src/crds/boltrun.rs` — represents infrastructure operations
- **Migration strategy**: CRD versioning via K8s API

### Web App Database (`apps/web/`)
- **Database engine**: SQLite (via `better-sqlite3` or similar; Drizzle ORM configured)
- **ORM**: Drizzle — `apps/web/src/lib/db/schema.ts`
- **Migration tool**: Drizzle Kit — `apps/web/drizzle.config.ts`
- **Auth**: Better Auth — `apps/web/src/lib/auth/`

### Bridge State DB (SQLite)
- **Shared schema**: `shared/bridge-state-schema.js` (imported by both Discord and Linear bridges)
- **Tables**: `sessions`, `provider_events`, `elicitation_requests`, `decision_options`, `votes`, `tally_snapshots`, `decision_outcomes`, `provider_message_refs`, `design_snapshots`, `decision_vote_tallies`
- **Used by**: `apps/discord-bridge/src/state/bridge-state-db.ts`, `apps/linear-bridge/src/state/bridge-state-db.ts`

### PRD-Relevant New Data (Solana Program Accounts)
The PRD defines new on-chain data structures (`OperatorConfig`, `CustomerBalance`, `TaskReceipt`) that do not exist in the current codebase. These will be Anchor PDAs in a new repo (`5dlabs/cto-pay`), not in this repo.

## 5. Architectural Patterns

### Layering Convention
- **Controller (Rust)**: Layered by concern under `crates/controller/src/`:
  - `crds/` — Custom Resource Definitions
  - `tasks/` — Per-task-type reconciliation (code/, bolt/, heal/, play/, cancel/, security/, label/)
  - `cli/` — CLI adapter layer with adapter pattern for multiple AI tools
  - `tasks/config.rs` — Centralized configuration
  - `tasks/types.rs` — Shared types
  - `tasks/tool_catalog.rs`, `tasks/tool_inventory.rs` — MCP tool management

### Error Handling
- **Rust**: `anyhow` / `thiserror` style (needs manual review of specific crates)
- **TypeScript**: Try-catch with structured error types (e.g., `ToolError` in `apps/cto-tools/mcp.ts` with numeric error codes)
- **Discord Bridge**: Best-effort error handling with logging, graceful degradation for interaction acknowledgment

### Logging/Observability
- **TypeScript apps**: Simple `console.log`/`console.warn`/`console.error` with `[service]` prefix
- **Rust**: Needs manual review; likely `tracing` crate based on K8s operator patterns
- **Bridge state**: SQLite-based event logging (`appendProviderEvent`)

### Configuration Approach
- **Environment variables**: Primary config mechanism across all services
- **Config files**: `config/controller-config.yaml`, `.codex/config.toml`, various `deno.json`/`package.json`
- **Secrets pipeline**: 1Password → OpenBao → External Secrets Operator → K8s secrets → pod env vars (documented in PRD)

### DI / Composition
- **TypeScript**: Factory functions returning interface implementations (e.g., `createBridge()`, `createDiscordClient()`, `createDiscordElicitationHandler()`, `createBridgeStateDb()`)
- **Rust Controller**: Module-level organization with explicit struct construction

### Adapter Pattern (CLI)
- `crates/cli/src/adapters/` — One adapter per AI tool (claude.rs, codex.rs, cursor.rs, etc.)
- `crates/cli/src/adapter.rs` — Trait definition
- `crates/cli/src/factory.rs` — Factory for adapter selection
- Same pattern mirrored in controller: `crates/controller/src/cli/adapters/`

## 6. Test Infrastructure

### Rust Tests
- **Framework**: Built-in Rust test framework + integration tests
- **Location**: `crates/controller/tests/` — `agent_cli_matrix_tests.rs`, `e2e_template_tests.rs`
- **CI**: `controller-ci.yaml` workflow — builds, lints (clippy), tests
- **Linting**: Clippy (pedantic), rustfmt — configured in `crates/controller/clippy.toml`, `crates/controller/rustfmt.toml`

### TypeScript Tests
- **Deno tests**: `apps/cto-tools/codegen_test.ts`, `apps/cto-tools/mcp_test.ts` — Deno native test runner with `jsr:@std/assert`
- **Bun tests**: `apps/intake-agent/src/operations/design-intake.test.ts`, `prd-research.test.ts` — Bun test runner
- **Vitest**: `apps/grok/tests/` — `config.test.ts`, `grok-client.test.ts`, `server.test.ts`; `apps/web/vitest.config.ts`
- **Test pattern**: Mock external dependencies (fetch, filesystem), test pure functions and integration flows

### CI/CD (GitHub Actions)
- **Workflows directory**: `.github/workflows/`
- **Relevant workflows**:
  - `controller-ci.yaml` — Rust controller build/test
  - `tools-ci.yaml` — Tools CI
  - `tools-smoke-test.yml` — Smoke tests
  - `code-quality.yaml` — Cross-repo quality checks
  - `codeql.yaml` — Security scanning
  - `skills-security-scan.yaml` — Skills security scan
- **Reusable actions**: `.github/actions/` — `docker-build-push/`, `rust-build/`, `rust-lint/`, `rust-test/`, `setup-rust/`, `setup-rust-k8s/`

### Test Conventions for New Code
- Rust: Integration tests alongside unit tests; Clippy pedantic; rustfmt enforced
- TypeScript: Test files co-located with source (`*.test.ts`); mock external APIs via `globalThis.fetch` stubbing
- All PR tests must pass before merge (Atlas merge gate)

## 7. Integration Points for PRD

### PRD Feature: Solana Program (Anchor)
- **Target**: New repo `5dlabs/cto-pay` — **does not exist in current codebase**
- **Reuse**: Nothing directly from current repo for the Anchor program itself
- **Reference**: Rust patterns from `crates/` (error handling, project structure) as style guidance
- **CI pattern**: Follow `.github/actions/rust-build/action.yaml` and `.github/actions/rust-test/action.yaml` for Rust CI setup

### PRD Feature: Settlement Hook (Controller Integration)
- **Connect to**: `crates/controller/src/tasks/code/controller.rs` — task reconciliation loop, add settlement call after terminal state
- **Connect to**: `crates/controller/src/tasks/config.rs` — add Solana RPC config fields
- **Connect to**: `crates/controller/src/tasks/code/status.rs` — add `on_chain_settlement_sig` to CodeRun status
- **Connect to**: `crates/controller/src/tasks/code/resources.rs` — pod resource specs used for billing computation
- **Connect to**: `crates/controller/src/tasks/types.rs` — shared task types, add billing-related types
- **Extend**: `crates/controller/src/crds/coderun.rs` — add settlement fields to CRD spec/status
- **Secrets**: Follow existing pattern — 1Password → OpenBao → External Secrets Operator → K8s secret → pod env. Reference `crates/cto-lite/tauri/src/commands/credentials.rs` for credential handling patterns.
- **Cargo dependency**: Add `solana-sdk`, `solana-client`, `anchor-client` to `crates/controller/Cargo.toml`

### PRD Feature: CLI / Demo Script (TypeScript)
- **Target**: New package, likely in `apps/cto-pay-cli/` or the new `cto-pay` repo
- **Reuse pattern**: Follow `apps/cto-tools/` for Deno-based tool structure, OR follow `apps/grok/` for Bun-based CLI structure
- **Package structure reference**: `apps/grok/package.json` — MCP server with CLI scripts, vitest tests
- **TypeScript config reference**: `apps/grok/tsconfig.json` — ES2022, ESNext modules, bundler resolution, strict mode

### PRD Feature: Off-Chain Receipt Storage
- **No existing Arweave/S3 integration** in the codebase to reuse
- **New infrastructure**: Needs to be built, likely as a utility module in the `cto-pay` repo or as a shared library

### PRD Feature: CI/CD for New Repo
- **Reuse**: `.github/actions/rust-build/action.yaml`, `.github/actions/rust-test/action.yaml` for Rust CI
- **Reference**: `.github/workflows/controller-ci.yaml` for Rust CI workflow pattern
- **Reference**: `.github/workflows/tools-ci.yaml` for TypeScript CI workflow pattern

## 8. Constraints (Do Not Change)

### Public APIs / Interfaces
- **CRD schemas**: `crates/controller/src/crds/coderun.rs`, `crates/controller/src/crds/boltrun.rs` — extending is OK, removing/renaming fields is not (production K8s clusters depend on these)
- **MCP tool protocol**: `apps/cto-tools/mcp.ts` — JSON-RPC 2.0 contract must be maintained
- **Discord Bridge HTTP API**: `apps/discord-bridge/src/http-server.ts` — existing endpoints must remain stable
- **Bridge state DB schema**: `shared/bridge-state-schema.js` — migration-safe additions only

### Production Schemas
- **Bridge SQLite schema**: Used by both Discord and Linear bridges in production; additive changes only
- **Web app Drizzle schema**: `apps/web/src/lib/db/schema.ts` — requires Drizzle migrations for changes

### Controller Architecture
- **Task module structure**: `crates/controller/src/tasks/` — each task type has its own module (`code/`, `bolt/`, `heal/`, etc.). New settlement logic should integrate into `code/` module, not create a new top-level task type.
- **Adapter pattern**: `crates/controller/src/cli/adapters/` — do not change the adapter trait or factory pattern

### Deploy Topology
- **K8s-native**: Controller runs as a K8s deployment/operator; new settlement functionality must work within this context
- **Secrets pipeline**: 1Password → OpenBao → External Secrets Operator → K8s secret — new secrets (Solana operator keypair) must use this pipeline, not introduce a parallel secrets mechanism

### Cargo Workspace
- **Root `Cargo.toml`**: Workspace members defined at root; new crates (if any) in the main repo must be added as workspace members
- **`.cargo/config.toml`**: Cargo configuration exists; respect existing build settings

### Important Note: Separate Repo
The PRD targets a **new repo** (`5dlabs/cto-pay`). The integration points in the CTO controller (`crates/controller/`) are the only files in *this* repo that would be modified. The Anchor program and CLI/demo script should live in the new repo. Task generation should account for this split.