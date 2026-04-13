# Codebase Context

## 1. Repository Overview

- **URL**: https://github.com/5dlabs/cto
- **Primary Languages**: Rust (backend platform, controllers, CLI), TypeScript (bridges, web apps, tools)
- **Build System**: Cargo workspace (Rust), Bun/npm (TypeScript apps), Next.js (web apps)
- **Layout**: Monorepo with `crates/` (Rust), `apps/` (TypeScript services), `infra/` (GitOps/K8s), `skills/` (agent skills), `templates/` (Handlebars templates for agent workflows)
- **Rust Edition**: 2021
- **Workspace Version**: 0.2.52
- **Key Frameworks**: Anchor (target for PRD), Axum (HTTP), Kube-rs (K8s operators), Tokio (async), Next.js 16 (web), React 19, Tailwind 4

## 2. Service Architecture

### Rust Crates (relevant to PRD)

| Service | Path | Purpose | Key Dependencies |
|---------|------|---------|-----------------|
| **controller** | `crates/controller/` | Kubernetes agent controller — reconciles CodeRun CRDs, manages agent lifecycle, dispatches tasks | kube 0.98, k8s-openapi 0.24, axum 0.8.8, octocrab 0.49, reqwest 0.12 |
| **config** | `crates/config/` | Shared CTO configuration types and generation for Play workflows | serde, tracing |
| **cli** | `crates/cli/` | Shared CLI types and adapters for AI coding assistants | tokio, serde, handlebars, schemars |
| **pm** | `crates/pm/` | Project management server (Linear, Asana, Jira integration) | axum, kube, reqwest, linear-sink |
| **notify** | `crates/notify/` | Notification system for platform events | tokio, reqwest, chrono |
| **healer** | `crates/healer/` | Self-healing platform monitor, spawns remediation agents | axum, reqwest, handlebars, notify |
| **scm** | `crates/scm/` | Unified SCM abstraction for GitHub and GitLab | reqwest, base64 |
| **acp-runtime** | `crates/acp-runtime/` | Shared ACP (Agent Client Protocol) runtime helpers | agent-client-protocol 0.10.0, tokio, serde |
| **intake** | `crates/intake/` | PRD → tasks and prompts for AI-driven development | handlebars, reqwest, cli, config, pm |
| **tools** | `crates/tools/` | Dynamic MCP tool management and proxy server | axum, kube, reqwest |
| **dex-indexer** | `crates/dex-indexer/` | Solana DEX swap indexer (Yellowstone gRPC → QuestDB) | yellowstone-grpc-client 12.1, questdb-rs 6.0, tokio |

### TypeScript Apps (relevant to PRD)

| Service | Path | Runtime | Purpose |
|---------|------|---------|---------|
| **web** | `apps/web/` | Next.js 16, React 19 | CTO web dashboard (auth, onboarding, settings) |
| **intake-util** | `apps/intake-util/` | Bun | Intake workflow utility (fan-out, classify, validate) |
| **intake-agent** | `apps/intake-agent/` | Bun | Claude Agent SDK wrapper for intake PRD parsing |
| **linear-bridge** | `apps/linear-bridge/` | Node.js/tsx | HTTP-to-Linear bridge for agent visibility |
| **discord-bridge** | `apps/discord-bridge/` | Node.js/tsx | HTTP-to-Discord bridge for agent visibility |
| **cto-tools** | `apps/cto-tools/` | Deno | MCP tools codegen |

### Existing Solana/Trading Infrastructure

| Path | Purpose | Relevance to PRD |
|------|---------|-----------------|
| `skills/trader/` | Multiple Solana trading skills (jup-skill, solana-connect, moltiumv2, etc.) | Existing `@solana/web3.js` patterns, wallet handling, transaction submission |
| `skills/trader/jup-skill/` | Jupiter API skill — fetch, sign, execute Solana txns | TypeScript Solana tx patterns with `@solana/web3.js ^1.95.0`, `bs58 ^6.0.0` |
| `skills/trader/solana-connect/` | Secure toolkit for AI agents to interact with Solana | Wallet management patterns, `@solana/web3.js ^1.98.0` |
| `crates/dex-indexer/` | Solana DEX indexer with Yellowstone gRPC | Rust Solana ecosystem experience, though different from Anchor program dev |

## 3. API Contracts

### Controller Binary

- **Binary**: `agent-controller` (`crates/controller/src/bin/agent_controller.rs`)
- **Exposed**: HTTP server (Axum) + Kubernetes controller reconciliation loops
- **Auth**: K8s RBAC, GitHub App tokens, Linear OAuth
- **Key CRDs**: `CodeRun` (`crates/controller/src/crds/coderun.rs`), `BoltRun` (`crates/controller/src/crds/boltrun.rs`)

### Controller Task Reconciliation (Settlement Hook integration point)

- **Task controller**: `crates/controller/src/tasks/code/controller.rs` — the reconciliation loop that manages CodeRun lifecycle. The PRD specifies adding a settlement hook here after terminal state.
- **Task resources**: `crates/controller/src/tasks/code/resources.rs` — pod resource construction; metering source for billing dimensions.
- **Task status**: `crates/controller/src/tasks/code/status.rs` — CodeRun status management where `on_chain_settlement_sig` would be added.
- **Task config**: Likely in `crates/controller/src/tasks/` area for adding Solana RPC config.

### PM Server

- **Binary**: `pm-server` (`crates/pm/src/bin/pm-server.rs`)
- **Endpoints**: Webhook handlers, OAuth endpoints (`/oauth/refresh/{agent}`), intake handlers
- **Integration**: Linear GraphQL API, GitHub/GitLab via SCM crate

### Web App API Routes

- `apps/web/src/app/api/auth/[...all]/route.ts` — Better Auth integration
- `apps/web/src/app/api/health/route.ts` — Health check
- `apps/web/src/app/api/onboarding/` — Onboarding flow including managed GitHub setup
- `apps/web/src/app/api/validate-key/route.ts` — API key validation
- `apps/web/src/app/api/validate-provider-key/route.ts` — Provider key validation

## 4. Data Models

### Database (Web App)

- **Engine**: PostgreSQL via Neon (`@neondatabase/serverless`)
- **ORM**: Drizzle ORM 0.45.1
- **Schema**: `apps/web/src/lib/db/schema.ts`
- **Migration tool**: Drizzle Kit 0.31.9 (`drizzle-kit generate`, `drizzle-kit migrate`, `drizzle-kit push`)
- **Config**: `apps/web/drizzle.config.ts`

### Kubernetes CRDs

- **CodeRun**: `crates/controller/src/crds/coderun.rs` — primary task CRD. The PRD's settlement hook integrates here. The CRD status would need a new field `on_chain_settlement_sig`.
- **BoltRun**: `crates/controller/src/crds/boltrun.rs` — deployment-related CRD

### Key Rust Data Types

- **Controller CLI types**: `crates/controller/src/cli/types.rs` — provider/key resolution (referenced in PRD)
- **Config types**: `crates/config/src/types.rs` — shared configuration types
- **Agent types**: `crates/config/src/agents.rs` — agent configuration

## 5. Architectural Patterns

### Layering / Architecture

- **No single dominant pattern** — pragmatic layered architecture:
  - Rust crates use module-based separation (`src/bin/`, `src/tasks/`, `src/cli/`, `src/crds/`)
  - Controller follows Kubernetes operator pattern with reconciliation loops
  - TypeScript apps follow Next.js App Router conventions
  - Agent workflows use Handlebars templates (`templates/`) rendered at runtime

### Error Handling

- **Rust**: `anyhow` for application errors, `thiserror` for library error types (workspace dependencies)
- **TypeScript**: Standard try/catch, no unified error framework visible

### Logging/Tracing

- **Rust**: `tracing` + `tracing-subscriber` (workspace deps with `env-filter` and `json` features)
- **OpenTelemetry**: Full OTLP stack — `opentelemetry 0.31.0`, `opentelemetry-otlp 0.31`, `tracing-opentelemetry 0.31.0` (used in controller)
- **Convention**: Use `tracing::*` macros, never `println!`

### Configuration

- **Rust**: Environment variables + config files, `clap` for CLI args (workspace dep with `derive`, `env`, `cargo` features)
- **Secrets Pipeline**: 1Password → OpenBao → External Secrets Operator → K8s secrets → pod env vars (documented in `docs/secrets-management.md`, referenced by PRD)
- **TypeScript**: `.env` files, environment variables

### Dependency Injection

- **Rust**: Trait-based abstraction with `async-trait` (workspace dep), no DI container
- **Controller**: Uses trait objects for CLI adapters (`crates/controller/src/cli/adapter.rs`), factory pattern (`adapter_factory.rs`)

### Template System

- **Handlebars 6.4**: Used for agent prompt/workflow templates (`templates/` directory, `crates/controller/`, `crates/intake/`)

### Clippy/Lint Convention

- **Pedantic clippy** enabled at workspace and crate level (some crates use `deny`, others `warn`)
- **Specific allows**: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `too_many_lines`, etc.
- **Per-crate `clippy.toml`**: Some crates have custom thresholds (e.g., `crates/metal/clippy.toml`)

## 6. Test Infrastructure

### Rust Testing

- **Framework**: Built-in `#[cfg(test)]` + `tokio::test`
- **Mocking**: `mockall 0.14` (workspace dep)
- **HTTP mocking**: `wiremock 0.6` (workspace dep)
- **Test serialization**: `serial_test 3.4` (workspace dep)
- **Temp files**: `tempfile 3.26`
- **Organization**: Unit tests inline in modules, integration tests in `tests/` directories
- **CI Profile**: `[profile.ci]` defined in root `Cargo.toml` — optimized for fast builds (`lto = false`, `opt-level = 2`, `codegen-units = 16`)

### TypeScript Testing

- **Web app**: Vitest 4.0.18 with `@testing-library/react`, `@testing-library/jest-dom`, jsdom, `@vitest/coverage-v8`
- **Config**: `apps/web/vitest.config.ts`
- **Setup**: `apps/web/src/__tests__/setup.ts`
- **Linear bridge**: Vitest 1.6.1
- **Grok**: Vitest 1.0.0
- **Bun apps**: `bun test` (intake-util, intake-agent)

### CI/CD

- **GitHub Actions**: Extensive workflow collection in `.github/workflows/` (most archived to conserve quota)
- **Active workflow**: `.github/workflows/agents-build.yaml` (push-based automation)
- **Available workflows**: `controller-ci.yaml`, `tools-ci.yaml`, `healer-ci.yaml`, `web-ci.yaml`, `research-ci.yaml`, `code-quality.yaml`, `codeql.yaml`
- **Rust CI**: Custom actions at `.github/actions/` — `setup-rust/`, `rust-build/`, `rust-lint/`, `rust-test/`
- **Docker**: `.github/actions/docker-build-push/` for container builds
- **GitLab CI**: Parallel CI config in `.gitlab/ci/` (the repo mirrors to GitLab)

### Code Quality

- **Rust**: `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`
- **TypeScript (web)**: ESLint 9, Prettier 3.4.2, `prettier-plugin-tailwindcss`
- **TypeScript (website)**: ESLint 9 with `eslint-config-next`

## 7. Integration Points for PRD

### 7.1 Solana Program (Anchor) — NEW REPO (`cto-pay`)

The PRD targets a **separate repository** (`https://github.com/5dlabs/cto-pay`). This is a greenfield Anchor program. However, patterns from the existing codebase inform it:

- **Existing Solana Rust patterns**: `crates/dex-indexer/` uses `bs58`, Solana-adjacent libraries (Yellowstone gRPC). The workspace `Cargo.toml` already includes relevant dependencies.
- **Existing Solana TypeScript patterns**: Multiple skills under `skills/trader/` use `@solana/web3.js`, `bs58`, transaction signing patterns.
- **Reference**: `skills/trader/jup-skill/` — TypeScript Solana transaction patterns with `@solana/web3.js ^1.95.0`.
- **Reference**: `skills/trader/solana-connect/` — Wallet management, keypair handling.

### 7.2 Settlement Hook — Controller Integration

This is the primary integration with the existing CTO codebase:

| What | Where | What Changes |
|------|-------|-------------|
| **CodeRun CRD** | `crates/controller/src/crds/coderun.rs` | Add `on_chain_settlement_sig: Option<String>` to CodeRun status |
| **Task reconciliation** | `crates/controller/src/tasks/code/controller.rs` | Add post-terminal-state hook: compute bill → build receipt → submit `settle_task` |
| **Task status** | `crates/controller/src/tasks/code/status.rs` | Handle new settlement status fields |
| **Resource metering** | `crates/controller/src/tasks/code/resources.rs` | Extract billable dimensions (pod duration, infra tier) |
| **Config** | `crates/controller/src/tasks/` (new file or extend existing config) | Add Solana RPC endpoint, operator keypair path |
| **CLI types** | `crates/controller/src/cli/types.rs` | Add `solana_pubkey` to customer/provider model |
| **Controller Cargo.toml** | `crates/controller/Cargo.toml` | Add `solana-sdk`, `solana-client`, `anchor-client` dependencies |

**Key existing patterns to follow**:
- The controller already uses `reqwest` with `blocking` feature for sync HTTP within template rendering
- Secrets are managed via existing pipeline: 1Password → OpenBao → External Secrets → K8s secret
- The controller binary is `agent-controller` at `crates/controller/src/bin/agent_controller.rs`
- Template system uses Handlebars (`handlebars 6.4`)
- Tracing/logging follows `tracing` crate conventions
- Error handling uses `anyhow` + `thiserror`

### 7.3 CLI / Demo Script (TypeScript)

- **Runtime**: Bun (consistent with `apps/intake-util/`, `apps/intake-agent/`)
- **Existing Bun patterns**: `apps/intake-util/package.json` shows Bun build pattern (`bun build src/index.ts --compile --outfile intake-util`)
- **Solana TS patterns**: Reuse patterns from `skills/trader/jup-skill/` and `skills/trader/solana-connect/`
- **Package structure**: Follow existing `apps/` conventions — `package.json`, `tsconfig.json`, `src/` directory

### 7.4 Off-Chain Receipt Storage

- **PRD specifies**: Arweave or S3
- **Existing infra**: SeaweedFS operator for S3-compatible storage (`infra/gitops/applications/operators/README.md` lists `seaweedfs-operator.yaml`)
- **Deployment agent**: Bolt handles infrastructure and deployment

### 7.5 CI/CD for New Repo

- **Existing patterns**: `.github/actions/rust-build/`, `.github/actions/rust-test/` for Rust CI
- **Controller CI**: `.github/workflows/controller-ci.yaml` — reference for Rust service CI
- **Docker builds**: `.github/actions/docker-build-push/action.yaml`

### 7.6 Secrets Management

- **Documented**: `docs/secrets-management.md` (referenced in PRD)
- **Pipeline**: 1Password → OpenBao → External Secrets Operator → K8s secrets
- **Existing example**: Linear OAuth tokens, GitHub App keys follow this pipeline
- **New secret needed**: Solana operator keypair (for signing `settle_task` transactions)

## 8. Constraints (Do Not Change)

### Public APIs / Interfaces

- **CodeRun CRD schema**: The CRD is used by multiple services. Adding fields is safe; removing or renaming existing fields is not.
- **Controller HTTP API**: Any existing endpoints must remain backward-compatible.
- **PM server endpoints**: Used by Linear bridge and other services.

### Production Schemas

- **Web app database** (`apps/web/src/lib/db/schema.ts`): Drizzle schema, Neon PostgreSQL. Changes require migration via `drizzle-kit`.
- **Kubernetes CRDs**: Changes to CRD specs require careful versioning (used by `kube 0.98` with `k8s-openapi 0.24`).

### Shared Library Interfaces

- **`crates/config/`**: Shared types used by controller, intake, healer, PM, acp-runtime. Changes ripple across multiple crates.
- **`crates/scm/`**: Used by PM, healer, research. Interface changes affect multiple consumers.
- **`crates/acp-runtime/`**: Used by controller, healer, MCP servers. Protocol-level shared code.
- **`crates/notify/`**: Used by healer, controller. Notification abstractions.

### Workspace Dependencies

- **Root `Cargo.toml`**: Workspace dependency versions are pinned. New dependencies for the settlement hook should be added as workspace deps if they'll be shared, or as crate-local deps if controller-only.
- **Key version pins**: `tokio 1.50`, `axum 0.8.8`, `kube 0.98`, `serde 1.0`, `thiserror 2.0.18`, `anyhow 1.0`

### Deploy Topology

- **Controller**: Deployed as `agent-controller` binary in Kubernetes (namespace `cto`)
- **ArgoCD GitOps**: All K8s resources managed via ArgoCD (`infra/gitops/`)
- **Helm charts**: `infra/charts/` (referenced but not fully visible in packed output)
- **Docker images**: Built via `infra/images/` Dockerfiles, pushed to `ghcr.io/5dlabs/`

### Existing Solana Infrastructure

- **Solana validator node**: Running in the cluster (`skills/trader/k8s/` manifests for `agave-rpc`)
- **QuestDB**: Time-series DB for DEX data
- **Yellowstone gRPC**: Solana streaming infrastructure
- **These are in production** — do not modify Solana trader infrastructure as part of CTO Pay.

### Clippy/Lint Standards

- Any Rust code added to existing crates must pass `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` with the crate's configured allows.
- The controller crate does not have a `clippy.toml` but inherits workspace lint settings.

### Branch Strategy

- **Default branch**: Visible from PR template and workflow triggers — likely `main` or `develop` (detection logic in templates suggests both exist)
- **PR convention**: Conventional Commits (`feat:`, `fix:`, `chore:`)
- **Release**: Release-please (`.release-please-manifest.json` at version `0.2.52`)