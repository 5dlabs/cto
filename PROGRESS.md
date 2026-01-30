# CTO Lite Progress

## Current Status: Phase 2 In Progress 🔄

### Phase 1: Tauri App Foundation ✅

**Completed:**
- [x] Tauri 2.x project structure created
- [x] React frontend with Vite + shadcn/ui
- [x] Setup wizard with 7 steps
- [x] Rust backend with Tauri commands
- [x] TypeScript bindings for all commands
- [x] React hooks with loading/error states

### Phase 2: Core Infrastructure 🔄

**Completed:**
- [x] Create `cto-lite` Helm chart
  - Controller deployment + RBAC
  - PM server deployment
  - Cloudflared tunnel deployment
  - Secrets for API keys
  - Play workflow template (no Atlas)
  - CRDs (CodeRun, BoltRun)
- [x] Helm deployment commands in Tauri
- [x] Deploy step in setup wizard

**In Progress:**
- [ ] Fork PM server to `pm-lite` (using existing PM for now)
- [ ] Update agent prompts (no Atlas, clean PRs)
- [ ] Build tunnel allocation system
- [ ] Bundle skills into agent images
- [ ] Configure Bolt for local/Docker

### Phase 3: Dashboard and MCP ⏳

- [ ] Build workflow status/logs view
- [ ] Create MCP background service
- [ ] Integrate log streaming

### Phase 4: Distribution ⏳

- [ ] CI workflow for Tauri builds
- [ ] Code signing setup
- [ ] Binary bundling
- [ ] CDN distribution

### Phase 5: Polish ⏳

- [ ] User documentation
- [ ] Troubleshooting guide
- [ ] Beta testing

## Recent Commits

```
5cf42ea feat(cto-lite): add Deploy step to setup wizard
320515d feat(cto-lite): add Helm deployment commands
5af3cf4 feat(cto-lite): add Helm chart for local Kind deployment
fb6d817 feat(cto-lite): wire frontend to Tauri backend
0dff191 feat(cto-lite): add Tauri 2.x backend with commands
```

## File Structure

```
crates/cto-lite/
├── tauri/
│   ├── package.json
│   └── src-tauri/
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── capabilities/default.json
│       ├── icons/
│       └── src/
│           ├── main.rs
│           ├── lib.rs
│           ├── commands.rs
│           ├── state.rs
│           ├── keychain.rs
│           ├── docker.rs
│           ├── kind.rs
│           └── helm.rs          # NEW
└── ui/
    ├── package.json
    ├── src/
    │   ├── lib/tauri.ts         
    │   ├── hooks/use-tauri.ts   
    │   ├── components/
    │   │   ├── setup/
    │   │   │   ├── RuntimeStep.tsx
    │   │   │   ├── InstallStep.tsx
    │   │   │   └── DeployStep.tsx   # NEW
    │   │   └── SetupWizard/
    │   └── App.tsx
    └── dist/

infra/charts/cto-lite/           # NEW
├── Chart.yaml
├── values.yaml
├── crds/
│   ├── coderun-crd.yaml
│   └── boltrun-crd.yaml
└── templates/
    ├── _helpers.tpl
    ├── namespace.yaml
    ├── secrets.yaml
    ├── controller/
    │   ├── deployment.yaml
    │   └── rbac.yaml
    ├── pm/
    │   └── deployment.yaml
    ├── cloudflared/
    │   └── deployment.yaml
    └── workflows/
        └── play-workflow-lite.yaml
```

## Backend Commands

| Command | Description | Status |
|---------|-------------|--------|
| `check_docker` | Docker/OrbStack detection | ✅ |
| `check_kind` | Kind installation check | ✅ |
| `check_helm` | Helm installation check | ✅ |
| `get_setup_state` | Wizard state | ✅ |
| `save_setup_state` | Save wizard state | ✅ |
| `store_api_key` | Keychain storage | ✅ |
| `get_api_key` | Keychain retrieval | ✅ |
| `create_cluster` | Kind cluster creation | ✅ |
| `delete_cluster` | Kind cluster deletion | ✅ |
| `get_cluster_status` | Cluster status | ✅ |
| `deploy_chart` | Helm install/upgrade | ✅ |
| `get_release_status` | Helm status | ✅ |
| `uninstall_chart` | Helm uninstall | ✅ |
| `trigger_workflow` | Start workflow | 🔲 Stub |
| `get_workflow_status` | Workflow status | 🔲 Stub |
| `list_workflows` | List workflows | 🔲 Stub |

## Setup Wizard Steps

1. ✅ Runtime Check (Docker/Kind detection)
2. ✅ Stack Selection (Grizz/Nova)
3. ✅ API Keys (Keychain storage)
4. ✅ GitHub Connection (OAuth stub)
5. ✅ Cloudflare Tunnel (OAuth stub)
6. ✅ Create Cluster (Kind)
7. ✅ Deploy (Helm chart)

## Build Commands

```bash
# Build UI
cd crates/cto-lite/ui && npm run build

# Check Rust backend
cd crates/cto-lite/tauri/src-tauri && cargo check

# Run development
cd crates/cto-lite/tauri && npm run tauri dev
```

## Next Steps

1. **Test the app** - Run `npm run tauri dev` to test the full flow
2. **Install Helm** - Required for deployment
3. **Build agent images** - Need to containerize agents
4. **Implement workflow commands** - Connect to Argo
