# CTO Lite Task Queue

## Current Phase: 2 - Core Infrastructure

## Active Task
- [ ] **Create `cto-lite` Helm chart** - Simplified chart for Kind deployment

## Phase 2 Queue
- [ ] Create `infra/charts/cto-lite/` structure
- [ ] Define Chart.yaml and values.yaml
- [ ] Add controller deployment (reuse existing image)
- [ ] Add pm-lite deployment template
- [ ] Add Argo Workflows installation
- [ ] Add CRD for CodeRun
- [ ] Fork PM to `crates/cto-lite/pm-lite/`
- [ ] Remove Linear dependencies from pm-lite
- [ ] Add GitHub webhook handler to pm-lite
- [ ] Create play-workflow-lite.yaml template

## Phase 3 Queue (Next)
- [ ] Create `crates/cto-lite/mcp-lite/`
- [ ] Implement curated tool set
- [ ] Add MCP background service to Tauri
- [ ] Complete OAuth callback handlers
- [ ] Add log streaming to dashboard
- [ ] Connect workflow status to dashboard

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
