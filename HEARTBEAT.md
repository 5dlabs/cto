# HEARTBEAT.md

# Talos Bare Metal Orchestrator - talos-orchestrator project

**Goal**: Fully automated Talos Linux installation on Scaleway bare metal via Rust

---

## Current Status: ✅ BUILD COMPLETE

Binary: `talos-orchestrator/target/release/talos-orchestrator`
Scripts: `scripts/download-talosctl.sh`, `scripts/download-talos-image.sh`

### Phase 1: Core Implementation ✅ DONE
- [x] Project structure (Cargo.toml, src/)
- [x] Scaleway API client (rescue mode, power, server status)
- [x] SSH handler (connect, execute, upload)
- [x] talosctl wrapper (dd write, bootstrap)
- [x] State machine orchestrator
- [x] Config loading (YAML + env vars)
- [x] CLI with subcommands

### Phase 2: Build ✅ DONE
- [x] `cargo check` passes
- [x] `cargo build --release` succeeds (37.9s)

### Phase 3: Download Scripts ✅ DONE
- [x] `scripts/download-talosctl.sh` (arm64/darwin support)
- [x] `scripts/download-talos-image.sh` (caching support)

### Phase 4: Integration Test ⏳ READY FOR CREDENTIALS
- [x] Credentials verified via API
- [x] Environment variable support (SCW_ACCESS_KEY, SCW_SECRET_KEY, SCW_DEFAULT_PROJECT_ID)
- [x] Server list confirmed (talos-cp, talos-worker)
- [ ] Test rescue mode enable/disable
- [ ] Verify SSH connectivity

### Phase 5: Production Install ⏳ REQUIRES CREDENTIALS
- [ ] Run full install on Metal #0685
- [ ] Verify Talos API responds on port 50000
- [ ] Verify talosctl bootstrap succeeds

---

## Acceptance Criteria

✅ ALL of:
- Binary compiles and runs ✅
- Download scripts work
- Full install completes on Metal #0685
- Talos API responds on port 50000
- talosctl bootstrap succeeds

---

## Manual Intervention Required

1. **Scaleway credentials** - Set SCALEWAY_PROJECT_ID, SCALEWAY_ACCESS_KEY, SCALEWAY_SECRET_KEY
2. **Server selection** - Metal #0685 confirmed
3. **Final approval** - Before running on production hardware
