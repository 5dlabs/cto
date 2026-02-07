# CTO App Development Context - Last Updated 2026-02-04

## Current State
**PR #4307** - `fix/k8s-openapi-version-compatibility`
- Fixing k8s-openapi 0.22 → 0.24 API breaking changes
- Applied fixes: `external_secret_name()` now returns `Option<String>` directly, `cm.name` wrapped in `Some()`
- CI running, waiting for green

## Development Journey
1. **Phase 1 Complete** - Tauri foundation with React UI, container runtime detection, Kind management
2. **Phase 2 Complete** - Helm chart, PM-Lite, MCP-Lite, tunnel allocation, agent prompts
3. **Phase 3 Complete** - Dashboard with real-time logs, GitHub App integration, CI/CD for Tauri builds
4. **Current** - PR health monitoring, dependency fixes

## Key Architecture Decisions
- Single binary with feature flags (not "Lite" vs "Full")
- Zero-friction initialization with auto-detection of Docker/Kind
- Smart cluster detection - reuses existing clusters
- Fallback runtime chain: OrbStack → Docker Desktop → Colima → Podman

## Swarm Mode Usage
- Used claudesp for parallel workstreams
- 4 workers at a time for Phase 1/2
- Recursive loops with acceptance criteria

## Image Generation (from earlier session)
- Replicate API (Flux Schnell model - $0.003/image)
- SeaweedFS storage for generated images
- Working HTTP service at port 3333

## CTO App Tech Stack
- Tauri 2.0 + React 18 + shadcn/ui
- Local Kind cluster with Argo Workflows
- Cloudflare tunnel for webhooks

## Files Modified
- `crates/controller/src/tasks/bolt/resources.rs`
- `crates/controller/src/tasks/code/resources.rs`
- `Cargo.lock` (k8s-openapi 0.22 → 0.24)

## Open PRs to Monitor
- #4307 (k8s-openapi fix) - Priority
- #4304, #4291, #4290 - Blocked by #4307

---

## From Earlier Session (2026-02-01 to 2026-02-04)

### Image Generation Service
- Built HTTP service with Replicate API (Flux Schnell)
- SeaweedFS bucket created (`generated-images`)
- API key configured, billing set up
- Service running at port 3333

### Recursive Loop Pattern
```javascript
// Pattern used for Phase 3
spawn task → wait → check result → success? → next task
blocked? → report to user
```

### Heartbeat Monitoring
- HEARTBEAT.md in workspace for PR monitoring
- Priority order: conflicts → failing CI → bot comments → merge green

### PR Workflow
- pixel-assist agent handles PR fixes and merges
- I implement → pixel-assist manages PR health

---

*This file auto-updates from OpenMemory context refreshes*
