# CTO Lite Task Queue

## Current Phase: 2 - Core Infrastructure

## Active Task
- [x] **Create `cto-lite` Helm chart** ✅
- [x] **Create `pm-lite` crate** ✅
- [x] **Create `play-workflow-lite.yaml`** ✅

## Phase 2 Complete ✅
- [x] Create `infra/charts/cto-lite/` structure
- [x] Define Chart.yaml and values.yaml
- [x] Add controller deployment (reuse existing image)
- [x] Add pm-lite deployment template
- [x] Add Argo Workflows as dependency
- [x] Fork PM to `crates/cto-lite/pm-lite/`
- [x] Remove Linear dependencies from pm-lite
- [x] Add GitHub webhook handler to pm-lite
- [x] Create play-workflow-lite.yaml template

## Active Task
- [x] **Create `mcp-lite` crate** ✅
- [x] **Dashboard improvements** ✅

## Phase 3 Complete ✅
- [x] Create `crates/cto-lite/mcp-lite/`
- [x] Implement curated tool set (cto_trigger, cto_status, cto_logs, cto_jobs)
- [x] Add MCP background service to Tauri
- [x] Add log streaming to dashboard (WorkflowDetail component)
- [x] Connect workflow status to dashboard
- [ ] Complete OAuth callback handlers (deferred - already scaffolded)

## Phase 4 Queue (Distribution)
- [ ] Set up CI workflow for Tauri builds
- [ ] Configure code signing (Apple/Windows)
- [ ] Bundle kind/kubectl/helm/cloudflared
- [ ] Create download page

## Phase 5 Queue (Polish)
- [ ] User documentation
- [ ] Troubleshooting guide
- [ ] Beta testing

---
*Last updated: 2026-01-30 06:52 PST*
