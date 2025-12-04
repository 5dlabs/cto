<!-- 21b1d39d-5168-42e3-9f94-3dfdd90098e6 5958743a-9f68-4acf-a5d3-a7bbaf83e1b0 -->
# Complete Latitude Cluster Deployment

## Current State

- Servers cleaned up (none running)
- `cto-metal` crate complete with CLI (`metal` binary)
- Talos bootstrap automation working
- Single control plane validated end-to-end

## Phase 1: Two-Node Cluster Test

Verify the full flow works before adding stack deployment.

```bash
# Control plane
metal provision --hostname cto-cp1 --cluster-name cto-test \
  --plan c2-small-x86 --region MIA2 --output-dir /tmp/cto-cluster

# Worker
metal join --hostname cto-worker1 --plan c2-small-x86 --region MIA2 \
  --cluster-name cto-test --config-dir /tmp/cto-cluster
```

**Files:** [`crates/metal/src/bin/metal.rs`](crates/metal/src/bin/metal.rs) (already implemented)

## Phase 2: Stack Deployment Module

Create [`crates/metal/src/stack/mod.rs`](crates/metal/src/stack/mod.rs) with:

| Function | Description |

|----------|-------------|

| `deploy_argocd()` | Install ArgoCD via Helm, wait for ready |

| `deploy_vault()` | Install Vault operator + initialize |

| `deploy_cert_manager()` | Install cert-manager CRDs + controller |

| `deploy_ingress()` | Install NGrok operator or Nginx |

| `deploy_app_of_apps()` | Apply ArgoCD Application pointing to GitOps repo |

**Approach:** Shell out to `helm` and `kubectl` (same pattern as talosctl). Use existing Helm charts from [`infra/charts/`](infra/charts/).

## Phase 3: GitHub Integration

Create [`crates/metal/src/github/mod.rs`](crates/metal/src/github/mod.rs) with:

| Function | Description |

|----------|-------------|

| `create_org_repo()` | Create `{org}/cto-apps` repo via GitHub API |

| `push_initial_structure()` | Push app-of-apps template |

| `generate_app_install_urls()` | Generate install URLs for Morgan, Rex, Cipher, Blaze, Cleo |

| `verify_installations()` | Check apps are installed via GitHub API |

| `update_webhook_urls()` | Update webhook endpoints to new cluster |

**Approach:** Use existing apps (already public), user creates `5dlabs-test` org, script generates install URLs.

## Phase 4: Unified `cluster` Command

Add to [`crates/metal/src/bin/metal.rs`](crates/metal/src/bin/metal.rs):

```rust
Cluster {
    #[arg(long)] name: String,
    #[arg(long, default_value = "MIA2")] region: String,
    #[arg(long, default_value = "c2-small-x86")] cp_plan: String,
    #[arg(long, default_value = "c2-small-x86")] worker_plan: String,
    #[arg(long)] github_org: Option<String>,
    #[arg(long)] skip_stack: bool,
    #[arg(long, default_value = "/tmp/cto-cluster")] output_dir: PathBuf,
}
```

This orchestrates: provision CP -> provision worker -> deploy stack -> setup GitHub.

## Phase 5: Parallel Provisioning Optimization

Modify provisioning to run CP + worker creation in parallel:

1. Create both servers simultaneously via API
2. Wait for both to be ready
3. Trigger iPXE on both
4. Wait for both to enter Talos maintenance
5. Apply CP config first, bootstrap
6. Apply worker config, join cluster

**Files:** Add `provision_parallel()` to [`crates/metal/src/talos/bootstrap.rs`](crates/metal/src/talos/bootstrap.rs)

## Deliverable

```bash
# Single command to deploy complete CTO mirror
metal cluster --name cto-test --region MIA2 --github-org 5dlabs-test
```

This will:

1. Provision 2 bare metal servers (CP + worker)
2. Bootstrap Talos + Kubernetes
3. Deploy ArgoCD, Vault, Cert-Manager, Ingress
4. Create `5dlabs-test/cto-apps` repo
5. Print GitHub App install URLs (5 clicks)
6. Configure webhooks to point to new cluster

## Estimated Cost

- 2x c2-small-x86: ~$0.36/hr (~$9/day)
- Test run (2-3 hours): ~$1.00

### To-dos

- [x] Test 2-node cluster: provision CP + join worker + verify kubectl get nodes
- [x] Create stack module with deploy_argocd(), deploy_vault(), deploy_cert_manager()
- [x] Add deploy_ingress() for NGrok operator
- [ ] Create github module for org/repo/webhook management
- [x] Add unified 'cluster' subcommand to metal binary
- [x] Optimize with parallel CP + worker provisioning
- [ ] End-to-end test: single command deploys full CTO mirror