# Mayastor Firewall Configuration Required

## Current Status

**Working:**
- ✅ All Mayastor pods running (agent-core, io-engine, etcd, NATS, CSI components)
- ✅ io-engine successfully registered with agent-core
- ✅ Worker node disk pool created and online (893GB available)
- ✅ Network routing functional between nodes

**Blocked:**
- ❌ Control plane disk pools stuck in "Creating" state
- ❌ Talos firewall blocking required Mayastor ports
- ❌ Cannot apply patches: talosctl context not configured

## Root Cause

Mayastor components use `hostNetwork: true` for performance (DPDK, direct NVMe access). This means they bind directly to the node's IP addresses and require specific ports to be opened in the Talos firewall:

- **Port 50051/tcp**: agent-core gRPC API
- **Port 50052/tcp**: agent-core HA cluster communication
- **Port 10124/tcp**: io-engine NVMe-oF target

Currently, the Talos firewall is blocking these ports, preventing agent-core from communicating with io-engine on the control plane node (10.8.0.1).

## What's Needed

### 1. Configure talosctl

The `talosconfig` file is not present in the repository (it contains admin credentials and shouldn't be committed). You need to either:

- **Option A**: Locate your existing `talosconfig` file and set:
  ```bash
  export TALOSCONFIG=/path/to/talosconfig
  ```

- **Option B**: If you have physical access to the control plane node, you can retrieve the admin credentials from `/var/run/secrets/talos/admin.crt` and `/var/run/secrets/talos/admin.key`

### 2. Apply Firewall Patches

Once talosctl is configured, run the provided script:

```bash
cd infra/talos/config/simple
./apply-mayastor-firewall.sh
```

Or apply manually:

```bash
# Control plane
talosctl patch machineconfig -n 10.8.0.1 --patch @mayastor-network-patch.yaml

# Worker
talosctl patch machineconfig -n 10.8.0.2 --patch @mayastor-network-patch.yaml
```

### 3. Verify

Within 20 seconds of applying the patches, check that the control plane disk pools transition to "Created":

```bash
kubectl get diskpools -n mayastor
```

Expected output:
```
NAME            NODE                STATE    POOL-STATUS   CAPACITY
pool-cp-nvme1   admin-cto-cp1       Created  Online        ~960 GiB
pool-cp-nvme2   admin-cto-cp1       Created  Online        ~960 GiB
pool-cp-nvme3   admin-cto-cp1       Created  Online        ~960 GiB
worker1-pool    admin-cto-worker1   Created  Online        893 GiB
```

This will give you **~2.8TB** of total Mayastor storage capacity.

## Technical Details

The patch file at `config/simple/mayastor-network-patch.yaml` opens the required ports for the `10.0.0.0/8` subnet, which includes:
- Pod CIDR: `10.244.0.0/16`
- Service CIDR: `10.96.0.0/12`
- Node IPs: `10.8.0.0/24`

The firewall rules are applied immediately without requiring a node reboot.

## References

- Full setup guide: `config/simple/MAYASTOR-SETUP.md`
- Disk pool definitions: `infra/gitops/manifests/storage/mayastor-diskpools.yaml`
- Network patch: `config/simple/mayastor-network-patch.yaml`
