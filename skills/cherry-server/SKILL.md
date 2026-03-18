---
name: cherry-server
description: "Provision, configure, and manage Cherry Servers bare metal instances running Talos Linux + Kubernetes for HFT trading and Solana RPC. Use when provisioning new servers, debugging boot issues, applying Talos configs, or managing the trading cluster."
---

# Cherry Server — Bare Metal Talos Provisioning

Provision Cherry Servers bare metal instances with Talos Linux, Kubernetes, and Cilium CNI — optimized for low-latency HFT trading and Solana/Agave RPC workloads.

## When to Use

- Provisioning a new Cherry Servers bare metal instance
- Rebuilding or re-provisioning an existing server
- Debugging Talos boot, networking, or disk issues
- Applying or updating Talos machine configs
- Adding worker nodes to the cluster
- Checking cluster health or server status
- Any Cherry Servers API operations

## Account Details

- **Cherry API Base:** `https://api.cherryservers.com/v1`
- **Team ID:** 190658
- **Project ID:** 264136
- **API Key:** stored in `skills/trader/.env` as `CHERRY_API_KEY` (JWT token)
- **Auth header:** `Authorization: Bearer $CHERRY_API_KEY`
- **SSH Key ID:** 13680 (jonathon-macbook, `~/.ssh/id_ed25519`)

## Provisioning Pipeline

The full pipeline for a new bare metal server:

```
Cherry API: provision server
    → Wait for "deployed" status + SSH open
    → SSH in, download Talos kernel+initramfs to /boot
    → Create GRUB entry, set as default, reboot
    → Talos boots into maintenance mode (DHCP, port 50000)
    → Discover NIC names: talosctl get links --insecure
    → Discover disks: talosctl get discoveredvolumes --insecure
    → Update controlplane.yaml with correct MACs and disk devices
    → talosctl apply-config --insecure
    → Wait for install + reboot (~3-5 min)
    → talosctl bootstrap
    → talosctl kubeconfig
    → helm install cilium
    → Deploy workloads
```

### Step 1: Provision via Cherry API

```bash
CHERRY_API_KEY=$(grep CHERRY_API_KEY skills/trader/.env | cut -d= -f2)

# Create server
curl -s -X POST "https://api.cherryservers.com/v1/projects/264136/servers" \
  -H "Authorization: Bearer ${CHERRY_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "plan": "solana-server-gen5",
    "region": "NL-Amsterdam",
    "image": "Ubuntu 24.04 64bit",
    "hostname": "node-name",
    "ssh_keys": [13680]
  }'

# Check status (poll until deployed)
curl -s "https://api.cherryservers.com/v1/servers/{SERVER_ID}" \
  -H "Authorization: Bearer ${CHERRY_API_KEY}" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print(f'State: {d[\"state\"]} | Status: {d[\"status\"]}')"

# Rebuild existing server
curl -s -X POST "https://api.cherryservers.com/v1/servers/{SERVER_ID}/actions" \
  -H "Authorization: Bearer ${CHERRY_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"type": "rebuild", "image": "Ubuntu 24.04 64bit", "ssh_keys": [13680]}'
```

### Step 2: GRUB-Based Talos Boot (Proven Method)

SSH into the Ubuntu server and set up GRUB to boot Talos:

```bash
ssh -i ~/.ssh/id_ed25519 root@{SERVER_IP} 'bash -s' << 'REMOTE'
set -euo pipefail
TALOS_VERSION="v1.9.5"

# Download Talos kernel + initramfs from GitHub releases
wget -q -O /boot/vmlinuz-talos \
  "https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/vmlinuz-amd64"
wget -q -O /boot/initramfs-talos.xz \
  "https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/initramfs-amd64.xz"

# Create GRUB entry — NO ip= kernel param (DHCP handles networking in maintenance mode)
cat > /etc/grub.d/09_talos << 'GRUBEOF'
#!/bin/sh
exec tail -n +3 $0
menuentry "Talos Linux v1.9.5" {
  linux /boot/vmlinuz-talos talos.platform=metal console=tty0 console=ttyS1,115200n8 init_on_alloc=1 slab_nomerge pti=on consoleblank=0 nvme_core.io_timeout=4294967295 printk.devkmsg=on ima_template=ima-ng ima_appraise=fix ima_hash=sha512
  initrd /boot/initramfs-talos.xz
}
GRUBEOF
chmod +x /etc/grub.d/09_talos

sed -i 's/^GRUB_DEFAULT=.*/GRUB_DEFAULT="Talos Linux v1.9.5"/' /etc/default/grub
sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=3/' /etc/default/grub
update-grub

# Reboot into Talos
nohup bash -c 'sleep 2 && reboot' &>/dev/null &
echo "REBOOT_QUEUED"
REMOTE
```

After ~90 seconds, Talos enters maintenance mode on port 50000 via DHCP.

### Step 3: Discover Hardware in Maintenance Mode

**CRITICAL: Always discover NIC names and disk layout before applying config.**

```bash
# Discover NIC names (differ between Ubuntu and Talos kernels!)
talosctl get links --insecure --nodes {IP} --endpoints {IP}

# Discover disk layout
talosctl get discoveredvolumes --insecure --nodes {IP} --endpoints {IP}
```

### Step 4: Generate and Apply Machine Config

Use `talosctl gen config` for new clusters, or update existing controlplane.yaml.

**Network config MUST use `deviceSelector.hardwareAddr`** — never hardcode interface names:

```yaml
machine:
  network:
    interfaces:
      - deviceSelector:
          hardwareAddr: "xx:xx:xx:xx:xx:xx"  # From talosctl get links
        addresses:
          - {IP}/{CIDR}
        routes:
          - network: 0.0.0.0/0
            gateway: {GATEWAY}
    nameservers:
      - 1.1.1.1
      - 8.8.8.8
```

Apply:
```bash
talosctl apply-config --insecure --nodes {IP} --endpoints {IP} --file controlplane.yaml
```

The "graceful_stop" error on apply-config is **SUCCESS** — Talos drops the connection during reboot.

### Step 5: Bootstrap Kubernetes

Wait 3-5 minutes for Talos to install to disk and reboot, then:

```bash
talosctl bootstrap --nodes {IP} --endpoints {IP} --talosconfig config/talosconfig

# If "bootstrap is not available yet" — check volume status:
talosctl get volumestatus --nodes {IP} --endpoints {IP} --talosconfig config/talosconfig
# All volumes must be "ready" before bootstrap works

talosctl kubeconfig ./kubeconfig --nodes {IP} --endpoints {IP} --talosconfig config/talosconfig --force
```

### Step 6: Install Cilium CNI

```bash
helm repo add cilium https://helm.cilium.io/ && helm repo update cilium
KUBECONFIG=./kubeconfig helm upgrade --install cilium cilium/cilium \
  --namespace kube-system -f cilium-values.yaml --wait --timeout 5m
```

## Critical Lessons Learned

### NIC Naming Across Kernels

**NIC names change between Linux kernels.** Cherry's Intel E810-XXV SFP NICs:
- Ubuntu 24.04 (kernel 6.14): `ens2f0`, `ens2f1`
- Talos v1.9.5 (kernel 6.12.18): `ens2f0np0`, `ens2f1np1`
- Cherry rescue (kernel 6.8): `ens2f0np0`, `ens2f1np1`

**ALWAYS use `deviceSelector.hardwareAddr` in Talos config, never hardcoded names.**

### No LACP Bond Required

Cherry configures LACP bond0 on Ubuntu, but Talos with a single NIC + static IP works fine for all workloads. Bond adds complexity and failure modes. Only add bond if you need link aggregation throughput.

### Cherry RAID1 Leaves Orphan Disk

Cherry installs Ubuntu on RAID1 (nvme0n1 + nvme1n1). After Talos installs to nvme0n1, nvme1n1 retains old Ubuntu partitions. Talos CANNOT use nvme1n1 for data mounts without wiping. Use the dedicated large NVMe drives (nvme2n1, nvme3n1) instead.

### Cherry Rescue Mode Resets UEFI

Cherry rescue mode ALWAYS resets UEFI boot order (EFI NVRAM variables) on exit. Never use `efibootmgr` from rescue mode — changes are lost.

### Talos initramfs Has No Early NIC Drivers

All kernel modules (including Intel `ice` driver) are inside squashfs within the initramfs. The kernel `ip=` boot parameter does NOT work because NIC drivers aren't available during early boot. Talos `machined` handles networking after squashfs mount.

### Bootstrap Requires All Volumes Ready

If any `machine.disks` volume fails to provision (wrong device, existing partitions), etcd won't start and `talosctl bootstrap` returns "not available yet" forever. Check `talosctl get volumestatus` — all volumes must be in `ready` phase.

### Cherry SSH After Rebuild

SSH port opens during Cherry's install process before cloud-init populates authorized_keys. Wait for API status `deployed` AND successful SSH login — don't rely on port check alone.

### Cilium Native Routing

Cilium `routingMode: native` requires `ipv4NativeRoutingCIDR` matching the pod CIDR (default `10.244.0.0/16`). Capabilities must be arrays, not strings. See `references/cilium-values.yaml`.

### NVMe Device Names Change Between Boots

**NVMe device names (`/dev/nvmeXn1`) are NOT stable across reboots.** The kernel enumerates NVMe devices in non-deterministic order. On the same server:
- Boot 1: nvme0n1=960GB, nvme1n1=3.8TB, nvme2n1=3.8TB, nvme3n1=960GB
- Boot 2: nvme0n1=960GB, nvme1n1=960GB, nvme2n1=3.8TB, nvme3n1=3.8TB

**ALWAYS discover disks via `talosctl get discoveredvolumes --insecure` in maintenance mode before applying config.** Use the 3.8TB drives for data (accounts + ledger) and let Talos install to a 960GB drive. The RAID1 partner (other 960GB drive) should be skipped.

### GRUB Boot Method: Use 09_ Prefix

When creating the Talos GRUB entry, use `/etc/grub.d/09_talos` (not `45_talos`) so it appears BEFORE Ubuntu entries. Also modify `/etc/default/grub` directly with `sed` to set `GRUB_DEFAULT="Talos Linux v1.9.5"` — do NOT use `grub-set-default` as it only works when `GRUB_DEFAULT=saved`.

### Cherry IPMI Power-Off Timeout

Cherry's IPMI graceful shutdown can take **20+ minutes** to timeout when the OS doesn't respond to ACPI (e.g., Talos). All Cherry API actions are locked during this period. Plan accordingly — avoid unnecessary reboots via Cherry API when running Talos.

### Residual Filesystem Signatures on Data Disks

Cherry may leave vfat/EFI signatures on NVMe drives even if they appear clean in `discoveredvolumes`. If Talos reports `filesystem type mismatch: vfat != xfs` on a data disk, use `talosctl reset --graceful=false --reboot` to wipe and retry.

## Solana Server Gen5 Disk Layout

**WARNING: NVMe device names are NOT stable across reboots. Always discover before configuring.**

Typical layout (2x 960GB NVMe + 2x 3.8TB NVMe):

| Size | Role | Notes |
|------|------|-------|
| 960 GB | Talos OS | install disk, wipe: true |
| 960 GB | Spare | Old RAID1 partner, skip |
| 3.8 TB | /var/mnt/accounts | Agave accounts DB, random read-heavy |
| 3.8 TB | /var/mnt/ledger | Agave ledger + snapshots, sequential write-heavy |

## Config Files

All config files live at `/tmp/trading-talos/` on the local workstation:
- `config/controlplane.yaml` — Talos machine config (PKI, disks, kernel, sysctls, kubelet)
- `config/talosconfig` — talosctl auth config
- `config/secrets.yaml` — Cluster PKI secrets
- `kubeconfig` — Kubernetes admin kubeconfig
- `cilium-values.yaml` — Cilium Helm values

## Verification Checklist

After provisioning, verify ALL of these:

```bash
TC="--nodes {IP} --endpoints {IP} --talosconfig config/talosconfig"

# 1. Talos version
talosctl version $TC

# 2. Disk mounts
talosctl mounts $TC | grep mnt

# 3. Kernel args
talosctl read /proc/cmdline $TC

# 4. Key sysctls
talosctl read /proc/sys/vm/max_map_count $TC        # expect 2000000
talosctl read /proc/sys/net/core/rmem_max $TC        # expect 134217728
talosctl read /proc/sys/kernel/numa_balancing $TC     # expect 0

# 5. K8s node
KUBECONFIG=./kubeconfig kubectl get nodes -o wide     # expect Ready

# 6. All pods
KUBECONFIG=./kubeconfig kubectl -n kube-system get pods  # all Running

# 7. Cilium status
KUBECONFIG=./kubeconfig kubectl -n kube-system exec ds/cilium -- cilium status
```
