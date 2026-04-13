# Codebase Context

## 1. Repository Overview

- **URL**: https://github.com/5dlabs/cto
- **Primary Languages**: Rust, TypeScript
- **Frameworks**: Anchor (Solana, for new work), Rust (controller/CLI crates), TypeScript/Node.js (apps), Next.js (web apps), Deno (cto-tools), Bun (intake-agent, grok)
- **Build Systems**: Cargo (Rust workspace), npm/bun (TypeScript apps), Deno tasks (cto-tools)
- **Layout**: Monorepo with `crates/` (Rust), `apps/` (TypeScript services), `infra/` (Kubernetes/Helm), `archived/` (legacy)
- **Rust Edition**: Needs manual review — visible in individual `Cargo.toml` files under `crates/`
- **Node.js Target**: ES2022, TypeScript 5.x across apps
- **Solana/Anchor**: Not yet present in the repo — the PRD targets a new `cto-pay` repo, but integration hooks land in this repo's `crates/controller/`

## 2. Service Architecture

### Rust Services (under `crates/`)

| Service | Path | Purpose | Notes |
|---------|------|---------|-------|
| **Controller** | `crates/controller/` | Kubernetes operator for CodeRun/BoltRun CRDs; reconciles agent task pods | Contains `tasks/code/controller.rs` (settlement hook target), `tasks/config.rs` (config target), `crds/coderun.rs` (CRD definition) |
| **CLI** | `crates/cli/` | Multi-adapter CLI for launching agents (Claude, Codex, Cursor, Gemini, etc.) | Adapter pattern in `src/adapters/` |
| **Config** | `crates/config/` | Agent/tool configuration library | `agents.rs`, `tools.rs`, `types.rs` |
| **CTO Lite** | `crates/cto-lite/` | Desktop app (Tauri) + MCP server + PM server | Tauri frontend, Helm charts for local K8s |
| **ACP Runtime** | `crates/acp-runtime/` | Agent Communication Protocol runtime | Client/server/registry pattern |
| **Cost** | `crates/cost/` | Cost tracking models and metrics | `tracking/tracker.rs`, `tracking/models.rs` — directly relevant to billing |

### TypeScript Services (under `apps/`)

| Service | Path | Purpose | Port/Transport | Runtime |
|---------|------|---------|---------------|---------|
| **Discord Bridge** | `apps/discord-bridge/` | HTTP-to-Discord bridge for agent visibility + elicitation | HTTP :3200 | Node.js 20 |
| **Linear Bridge** | `apps/linear-bridge/` | Linear.app integration bridge | HTTP :3100 | Node.js |
| **Intake Agent** | `apps/intake-agent/` | PRD parsing, design intake, research | Binary (stdin/stdout JSON) | Bun |
| **Intake Util** | `apps/intake-util/` | Workflow utilities (fan-out, voting, validation) | Library | Node.js |
| **Grok** | `apps/grok/` | MCP server for X/Grok research | MCP stdio | Bun |
| **CTO Tools** | `apps/cto-tools/` | MCP tool SDK codegen + runtime | HTTP JSON-RPC | Deno |
| **Web** | `apps/web/` | Next.js SaaS dashboard (auth, onboarding, settings) | HTTP :3000 | Node.js/Next.js |
| **Website** | `apps/website/` | Marketing site (Next.js + Cloudflare Pages) | HTTP | Node.js/Next.js |
| **Pitch Deck** | `apps/pitch-deck/` | Investor pitch deck (Next.js) | HTTP | Node.js/Next.js |
| **Lobster Voice** | `apps/lobster-voice/` | TTS/voice synthesis for deliberation audio | Library | Node.js |
| **NATS Messenger** | `apps/nats-messenger/` | NATS-based agent messaging | NATS | Node.js |

### Key Communication Patterns
- **HTTP REST** between bridges (Discord ↔ Linear cross-cancel via HTTP POST)
- **JSON-RPC over HTTP** for MCP tool calling (`apps/cto-tools/mcp.ts`)
- **stdin/stdout JSON protocol** for intake-agent
- **Kubernetes CRD reconciliation** for the controller (watches CodeRun/BoltRun CRDs)
- **SQLite** for bridge state persistence (`shared/bridge-state-schema.js` — shared schema)

## 3. API Contracts

### PRD-Relevant: CTO Controller (Rust)

The settlement hook integrates into the controller's CodeRun reconciliation loop. Key files:

- **`crates/controller/src/tasks/code/controller.rs`** — Main CodeRun reconciliation logic. The PRD calls for adding settlement after terminal state transitions.
- **`crates/controller/src/tasks/config.rs`** — Task configuration. The PRD calls for adding Solana RPC endpoint and operator keypair path here.
- **`crates/controller/src/crds/coderun.rs`** — CodeRun CRD definition. The PRD calls for adding `on_chain_settlement_sig` to the status subresource.
- **`crates/controller/src/cli/types.rs`** — Provider/key resolution types. Customer wallet mapping may extend these types.

### PRD-Relevant: MCP Tool Protocol

`apps/cto-tools/mcp.ts` defines the JSON-RPC transport used by agents to call tools. The `callTool` function routes to local or remote MCP servers. This is relevant if the CLI/demo needs MCP-based tool integration.

- **Endpoint pattern**: `POST /mcp` with JSON-RPC `tools/list` and `tools/call` methods
- **Auth**: `X-Agent-Id` header
- **Routing**: `LOCAL_TOOLS` env var determines which tool prefixes route to a local sidecar vs remote

### PRD-Relevant: Discord/Linear Bridge HTTP APIs

The bridges expose HTTP endpoints that the settlement system could use for notifications:

- `POST /notify` — Agent message notification
- `POST /elicitation` — Decision request
- `GET /health` — Health check
- `GET /history/decisions` — Decision history

These are documented in `apps/discord-bridge/src/http-server.ts` and `apps/linear-bridge/src/http-server.ts`.

## 4. Data Models

### Kubernetes CRDs (Controller)

- **CodeRun CRD** (`crates/controller/src/crds/coderun.rs`) — Defines the lifecycle of an agent coding task. The PRD requires adding `on_chain_settlement_sig: Option<String>` to the CRD status. Needs manual review for exact struct layout.
- **BoltRun CRD** (`crates/controller/src/crds/boltrun.rs`) — Similar CRD for Bolt agent runs.

### SQLite (Bridge State)

Shared schema in `shared/bridge-state-schema.js` used by both Discord and Linear bridges:
- `sessions` — Tracks intake sessions
- `elicitation_requests` — Decision requests
- `decision_options`, `votes`, `tally_snapshots` — Voting
- `decision_outcomes` — Final decisions
- `provider_events` — Audit trail
- `provider_message_refs` — Discord/Linear message tracking
- `design_snapshots` — Design intake state

### Web App Database (Drizzle + SQLite/Turso)

- **`apps/web/src/lib/db/schema.ts`** — Drizzle schema for the SaaS dashboard. May need a `solana_pubkey` field on user/customer model per the PRD.
- **`apps/web/drizzle.config.ts`** — Drizzle config
- **`apps/web/src/lib/db/migrate.ts`** — Migration runner

### Cost Tracking Models

- **`crates/cost/src/tracking/models.rs`** — Existing cost tracking models. Directly relevant for computing billable amounts in the settlement hook.
- **`crates/cost/src/tracking/tracker.rs`** — Cost tracker implementation.
- **`crates/cost/src/tracking/metrics.rs`** — Cost metrics.

## 5. Architectural Patterns

### Rust (Controller)

- **Pattern**: Modular Kubernetes operator with domain-specific task controllers
- **Layering**: `crds/` (CRD definitions) → `tasks/` (domain controllers per CRD type) → `cli/` (CLI adapter layer)
- **Task controllers** follow a pattern: `mod.rs` (module exports), `controller.rs` (reconciliation loop), `resources.rs` (K8s resource construction), `status.rs` (status updates), `templates.rs` (pod templates)
- **Error handling**: Uses `Result` with domain-specific error types. Needs manual review for exact error enum patterns in `crates/controller/`.
- **Security module**: `crates/controller/src/tasks/security/` — RBAC, rate limiting, token validation, audit. The settlement hook should follow these patterns.
- **Config**: `crates/controller/src/tasks/config.rs` — Centralized task configuration. New Solana-related config fields go here.

### TypeScript (Apps)

- **Pattern**: Functional modules, no class hierarchies. Factory functions for dependency injection (`createBridge()`, `createDiscordElicitationHandler()`, etc.)
- **Error handling**: Custom error classes (e.g., `ToolError` in `mcp.ts`) with error codes
- **Logging**: Simple `console.log`/`warn`/`error` with prefixed logger objects passed via constructor injection
- **Config**: Environment variables with typed config objects (`loadConfig()` pattern in `apps/discord-bridge/src/config.ts`)
- **State persistence**: SQLite via `node:sqlite` for bridges; Drizzle ORM for the web app

### Conventions

- **Rust formatting**: `rustfmt.toml` in `crates/controller/`; `clippy.toml` for lint config
- **TypeScript**: Strict mode, ESNext modules, `bundler` module resolution
- **Package naming**: `@5dlabs/` scope for published packages

## 6. Test Infrastructure

### Rust Tests

- **Framework**: Standard Rust `#[cfg(test)]` + `#[test]` macros
- **Controller tests**: `crates/controller/tests/` — Integration tests (`agent_cli_matrix_tests.rs`, `e2e_template_tests.rs`)
- **CI workflow**: `.github/workflows/controller-ci.yaml` — Runs `cargo test`, `cargo clippy`, `cargo fmt --check`
- **Additional workflows**: `.github/workflows/code-quality.yaml`, `.github/workflows/codeql.yaml`
- **Rust CI actions**: `.github/actions/rust-build/`, `.github/actions/rust-lint/`, `.github/actions/rust-test/`, `.github/actions/setup-rust/`

### TypeScript Tests

- **Grok**: Vitest (`apps/grok/tests/`) — Unit tests with mocked fetch, fs
- **CTO Tools**: Deno test (`apps/cto-tools/codegen_test.ts`, `mcp_test.ts`) — Uses `jsr:@std/assert`
- **Intake Agent**: Bun test (`apps/intake-agent/src/operations/design-intake.test.ts`, `prd-research.test.ts`)
- **Web**: Vitest (`apps/web/src/__tests__/`, `apps/web/vitest.config.ts`)

### CI/CD

- **GitHub Actions**: `.github/workflows/` — Extensive CI for controller, web, tools, research, infra, healer, etc.
- **GitLab CI**: `.gitlab-ci.yml` configs in `.gitlab/ci/` — Parallel CI on GitLab mirror
- **Docker builds**: `.github/actions/docker-build-push/action.yaml`
- **Release**: `.github/workflows/release-please.yaml`

### PRD-Relevant Testing

The PRD specifies:
- **Anchor integration tests** (TypeScript, `anchor test`) — New, not yet in repo
- **Bankrun** for fast local program tests — New
- **Bun test** for CLI — Follows existing `apps/intake-agent` pattern

## 7. Integration Points for PRD

### Feature: Solana Program (Anchor)

This is **greenfield** — the program lives in a new repo (`5dlabs/cto-pay`). No existing code to connect to in this repo.

However, the program's **account structures** and **instruction interfaces** must be compatible with:
- The controller settlement hook (this repo)
- The CLI demo script (new repo or this repo's `apps/`)

### Feature: Settlement Hook (Controller Integration)

| Integration Point | Existing File | What to Connect/Extend |
|-------------------|---------------|----------------------|
| Task config | `crates/controller/src/tasks/config.rs` | Add `solana_rpc_url: Option<String>`, `operator_keypair_path: Option<String>` to task config struct |
| CodeRun reconciliation | `crates/controller/src/tasks/code/controller.rs` | Add settlement hook after terminal state detection. Follow existing pattern of status update methods. |
| CodeRun status | `crates/controller/src/crds/coderun.rs` | Add `on_chain_settlement_sig: Option<String>` to CRD status struct |
| Pod resource info | `crates/controller/src/tasks/code/resources.rs` | Reuse pod duration/resource computation for billing |
| Security patterns | `crates/controller/src/tasks/security/` | Follow `tokens.rs` pattern for keypair management, `validation.rs` for input validation |
| Cost tracking | `crates/cost/src/tracking/` | Reuse `models.rs` and `tracker.rs` for computing billable amounts |
| CLI types | `crates/controller/src/cli/types.rs` | Add wallet/pubkey resolution for customer mapping |

### Feature: CLI / Demo Script (TypeScript)

Should follow patterns from:
- **`apps/cto-tools/`** — Deno-based TypeScript tooling with JSON-RPC transport (if building as MCP tool)
- **`apps/intake-agent/`** — Bun-based TypeScript with JSON stdin/stdout protocol (if building as a binary)
- **`apps/grok/`** — Bun-based MCP server pattern with `@modelcontextprotocol/sdk` (if exposing as MCP)

The PRD specifies **Bun runtime** with `@coral-xyz/anchor` and `@solana/web3.js`. This aligns with the `apps/intake-agent/` pattern. Recommended location: `apps/cto-pay-cli/` or in the separate `cto-pay` repo.

### Feature: Off-Chain Receipt Storage

No existing Arweave or S3 upload code in the repo. This is greenfield within the settlement hook or a new utility module.

### Feature: Customer Wallet Mapping

The web app (`apps/web/src/lib/db/schema.ts`) may need a `solana_pubkey` field. The controller's customer profile model (needs manual review — likely in `crates/controller/` or a shared types crate) needs the same field.

### Feature: CI/CD for Anchor Program

Existing CI patterns to reuse:
- `.github/actions/setup-rust/action.yaml` — Rust toolchain setup (extend for Solana/Anchor CLI)
- `.github/workflows/controller-ci.yaml` — Pattern for Rust CI
- `.github/actions/docker-build-push/action.yaml` — If the CLI needs containerization

## 8. Constraints (Do Not Change)

### Public APIs
- **MCP JSON-RPC protocol** (`apps/cto-tools/mcp.ts`) — The `callTool` interface is consumed by all agent pods. Do not change the JSON-RPC schema.
- **Bridge HTTP APIs** (`apps/discord-bridge/src/http-server.ts`, `apps/linear-bridge/src/http-server.ts`) — Elicitation protocol types are shared across bridges. Additive changes only.
- **Elicitation types** (`apps/discord-bridge/src/elicitation-types.ts`) — Shared protocol between Discord and Linear bridges. Do not modify existing fields.

### Kubernetes CRDs
- **CodeRun CRD** (`crates/controller/src/crds/coderun.rs`) — Adding fields to the status subresource is safe; changing existing spec fields requires CRD migration.
- **BoltRun CRD** (`crates/controller/src/crds/boltrun.rs`) — Same constraints as CodeRun.

### Shared Libraries
- **Bridge state schema** (`shared/bridge-state-schema.js`) — Shared SQLite schema between Discord and Linear bridges. Additive migrations only.
- **Controller security module** (`crates/controller/src/tasks/security/`) — RBAC and token validation patterns are production-critical.

### Deployment Topology
- Controller runs as a Kubernetes deployment watching CRDs
- Bridges run as Kubernetes deployments in `bots` namespace
- Agent pods are ephemeral, spawned by the controller per CodeRun
- Secrets flow: 1Password → OpenBao → External Secrets Operator → K8s secrets → pod env vars

### Build System
- Rust workspace root `Cargo.toml` (at repo root — needs manual review for workspace members list)
- `.cargo/config.toml` — Cargo configuration (cross-compilation settings, etc.)
- Each TypeScript app has its own `package.json` — no monorepo package manager (no turborepo, no nx)

### Framework Versions (Visible)
- TypeScript: `^5.0.0` across apps
- discord.js: `^14.16.0`
- `@modelcontextprotocol/sdk`: `^1.0.0`
- Node.js: 20 (Docker base images)
- Deno: Uses `jsr:@std/assert@1.0.19`
- Bun: `>=1.1.0`
- Anchor: Not yet in repo — PRD targets `0.30+`
- Solana CLI: Not yet in repo — PRD targets `1.18+`