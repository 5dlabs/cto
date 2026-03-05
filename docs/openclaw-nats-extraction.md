# openclaw-nats: Open-Source Repo Extraction Plan

## Purpose

Extract the NATS messaging infrastructure (bridges + plugin) into a standalone
open-source repo at `github.com/5dlabs/openclaw-nats`. These components are used
by both the CTO repo and the OpenClaw repo and should live in a single place.

## What Moves

| Component | Current Location | Lines | Runtime Deps |
|-----------|-----------------|-------|-------------|
| NATS Messenger (OpenClaw plugin) | `apps/nats-messenger/` | ~850 | `nats ^2.28.0` |
| Discord Bridge | `apps/discord-bridge/` | ~1,500 | `nats ^2.28.0`, `discord.js ^14.16.0` |
| Linear Bridge | `apps/linear-bridge/` | ~2,300 | `nats ^2.28.0` |

## Known Discrepancy to Fix

`AgentMessage` is **duplicated verbatim** in three separate `types.ts` files:

- `apps/nats-messenger/types.ts` (canonical)
- `apps/discord-bridge/src/types.ts` (copy with comment "matches nats-messenger wire format")
- `apps/linear-bridge/src/types.ts` (copy with comment "matches nats-messenger wire format")

The new repo should have a single shared types package that all three import from.

## Target Repo Structure

```
openclaw-nats/
├── package.json                     # workspaces root
├── tsconfig.base.json               # shared TS config
├── README.md
├── LICENSE                          # MIT
│
├── packages/
│   ├── types/                       # @openclaw/nats-types
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── src/
│   │       └── index.ts             # AgentMessage, MessagePriority, AgentMessageType,
│   │                                # ProcessedMessage, ElicitationRequest/Response, etc.
│   │
│   ├── messenger/                   # @openclaw/nats-messenger
│   │   ├── package.json             # deps: @openclaw/nats-types, nats
│   │   ├── tsconfig.json
│   │   ├── openclaw.plugin.json
│   │   └── src/
│   │       ├── index.ts             # plugin registration
│   │       ├── client.ts            # NATS connection management
│   │       ├── service.ts           # OpenClaw service wrapper
│   │       ├── processor.ts         # message formatting
│   │       ├── tool.ts              # agent-facing tool
│   │       ├── actions.ts           # session delivery
│   │       └── config.ts            # configuration validation
│   │
│   ├── discord-bridge/              # @openclaw/discord-bridge
│   │   ├── package.json             # deps: @openclaw/nats-types, nats, discord.js
│   │   ├── tsconfig.json
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── index.ts
│   │       ├── bridge.ts
│   │       ├── discord-client.ts
│   │       ├── nats-tap.ts
│   │       ├── config.ts
│   │       ├── types.ts             # Discord-specific types only (ConversationState, RoomState)
│   │       ├── elicitation-handler.ts
│   │       └── elicitation-types.ts
│   │
│   └── linear-bridge/               # @openclaw/linear-bridge
│       ├── package.json             # deps: @openclaw/nats-types, nats
│       ├── tsconfig.json
│       ├── Dockerfile
│       └── src/
│           ├── index.ts
│           ├── bridge.ts
│           ├── linear-client.ts
│           ├── issue-manager.ts
│           ├── agent-session-manager.ts
│           ├── webhook-server.ts
│           ├── nats-tap.ts
│           ├── config.ts
│           ├── types.ts             # Linear-specific types only (LinearConversationState, IssueMapping)
│           ├── elicitation-handler.ts
│           └── elicitation-types.ts
│
├── infra/
│   ├── manifests/
│   │   ├── discord-bridge/
│   │   │   └── deployment.yaml      # Namespace, SA, Deployment, ExternalSecret
│   │   └── linear-bridge/
│   │       └── deployment.yaml      # Namespace, SA, Deployment, Service, ExternalSecret
│   └── gitops/
│       ├── discord-bridge.yaml       # ArgoCD Application
│       └── linear-bridge.yaml        # ArgoCD Application
│
├── tests/
│   ├── docker-compose.yml            # NATS + both bridges for local testing
│   ├── test-bridges.sh               # Integration test script
│   └── .env.example                  # Token template
│
└── .github/
    ├── actions/
    │   └── docker-build-push/
    │       └── action.yaml           # Shared Docker build action
    └── workflows/
        ├── ci.yml                    # Test all packages on PR
        ├── discord-bridge-publish.yml
        └── linear-bridge-publish.yml
```

## Implementation Steps

### 1. Create the repo

```bash
gh repo create 5dlabs/openclaw-nats --public --description "NATS messaging infrastructure for OpenClaw agents"
```

### 2. Initialize monorepo

- Root `package.json` with npm workspaces pointing to `packages/*`
- Shared `tsconfig.base.json` (target ESNext, strict, bundler resolution)
- MIT LICENSE

### 3. Extract shared types into `packages/types/`

Pull the `AgentMessage` interface and related types out of all three `types.ts` files
into a single `@openclaw/nats-types` package:

```typescript
// packages/types/src/index.ts
export interface AgentMessage {
  from: string;
  to?: string;
  subject: string;
  message: string;
  priority: MessagePriority;
  timestamp: string;
  replyTo?: string;
  type?: AgentMessageType;
  role?: string;
  metadata?: Record<string, string>;
}

export type MessagePriority = "normal" | "urgent";
export type AgentMessageType = "message" | "discovery_ping" | "discovery_pong";

export interface ProcessedMessage {
  from: string;
  content: string;
  timestamp: string;
  priority: MessagePriority;
}

// Elicitation types (shared between both bridges)
export interface ElicitationRequest { ... }
export interface ElicitationResponse { ... }
```

### 4. Move nats-messenger to `packages/messenger/`

- Update imports: `from './types'` → `from '@openclaw/nats-types'`
- Keep `openclaw.plugin.json` — this is the OpenClaw plugin manifest
- Remove `AgentMessage` and shared types from local types, keep plugin-specific types

### 5. Move discord-bridge to `packages/discord-bridge/`

- Update imports: `AgentMessage` from `@openclaw/nats-types`
- Keep Discord-specific types (`ConversationState`, `RoomState`) in local `types.ts`
- Copy Dockerfile as-is

### 6. Move linear-bridge to `packages/linear-bridge/`

- Update imports: `AgentMessage` from `@openclaw/nats-types`
- Keep Linear-specific types (`LinearConversationState`, `IssueMapping`) in local `types.ts`
- Copy Dockerfile as-is

### 7. Move infrastructure

- Copy K8s manifests from `infra/manifests/{discord,linear}-bridge/`
- Copy ArgoCD apps from `infra/gitops/applications/workloads/{discord,linear}-bridge.yaml`
- Update ArgoCD `source.repoURL` to `https://github.com/5dlabs/openclaw-nats`
- Update ArgoCD `source.path` to `infra/manifests/{discord,linear}-bridge`

### 8. Move CI/CD

- Copy `.github/actions/docker-build-push/` (shared Docker build action)
- Copy `.github/workflows/{discord,linear}-bridge-publish.yml`
- Add `ci.yml` workflow that runs `npm test` and `tsc --noEmit` across all packages on PR
- Update Dockerfile contexts for monorepo paths

### 9. Move test infrastructure

- Copy `tests/bridges/` → `tests/`
- Update `docker-compose.yml` build contexts for new paths

### 10. Update CTO repo

After the new repo is live:

- Delete `apps/nats-messenger/`, `apps/discord-bridge/`, `apps/linear-bridge/` from CTO
- Delete `infra/manifests/{discord,linear}-bridge/` from CTO
- Delete `infra/gitops/applications/workloads/{discord,linear}-bridge.yaml` from CTO
- Delete `.github/workflows/{discord,linear}-bridge-publish.yml` from CTO
- Delete `tests/bridges/` from CTO
- Update ArgoCD to point at the new repo

### 11. Update OpenClaw repo

- Replace any local `nats-messenger` copy with `@openclaw/nats-messenger` dependency
- Or: add `openclaw-nats` as a git submodule if preferred

## Elicitation Types Consolidation

Both bridges have nearly identical `elicitation-handler.ts` and `elicitation-types.ts`.
Consider whether to:

- **Option A**: Move shared elicitation types into `@openclaw/nats-types` and keep
  platform-specific handlers in each bridge
- **Option B**: Create a `@openclaw/bridge-common` package with shared handler logic
  and platform adapters

Option A is simpler and recommended for the initial extraction.

## Docker Image Names

Keep existing GHCR image names to avoid breaking deployments:

- `ghcr.io/5dlabs/discord-bridge` (unchanged)
- `ghcr.io/5dlabs/linear-bridge` (unchanged)

## Version Strategy

- Start at `0.1.0` for all packages
- Use [changesets](https://github.com/changesets/changesets) for coordinated versioning
- Publish `@openclaw/nats-types` to npm (public)
- Bridge packages stay unpublished (Docker images only)
- `@openclaw/nats-messenger` published to npm for OpenClaw plugin consumption

## Verification Checklist

- [ ] `npm install` at root installs all workspaces
- [ ] `tsc --noEmit` passes in all packages
- [ ] Discord bridge Docker build succeeds
- [ ] Linear bridge Docker build succeeds
- [ ] `docker compose up` starts NATS + both bridges
- [ ] `test-bridges.sh` passes (messages flow through)
- [ ] GitHub Actions trigger correctly on package-scoped changes
- [ ] ArgoCD syncs from new repo successfully
- [ ] OpenClaw plugin loads `@openclaw/nats-messenger` correctly
- [ ] No remaining NATS/bridge references in CTO repo (except ArgoCD pointers)
