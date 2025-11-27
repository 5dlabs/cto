 CTO Platform-in-a-Box: Architecture Document

## Overview

This document describes the technical architecture for the CTO Platform-in-a-Box product,
a distributable ISO that installs a complete cloud-native development platform on
bare-metal hardware.

## Product Tiers

The platform is offered in three tiers, each building on shared components:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Product Tier Architecture                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                    CTO Platform Core (Shared)                               ││
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────────────────┐ ││
│  │  │ Agent Ctrl  │ │ MCP Server  │ │ Workflows   │ │ Observability Stack   │ ││
│  │  │ & Agents    │ │             │ │             │ │ (Prometheus/Grafana)  │ ││
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └───────────────────────┘ ││
┐ ┌─────────────┐ ┌─────────────┐ ┌───────────────────────┐ ││
│  │  │ ArgoCD      │ │ Vault       │ │ CNPG        │ │ MinIO                 │ ││
│  │  │ (GitOps)    │ │ (Secrets)   │ │ (Postgres)  │ │ (Object Storage)      │ ││
┘ └─────────────┘ └─────────────┘ └───────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                            ▲              ▲              ▲                       │
│                            │              │              │                       │
│  ┌─────────────────────────┴──────────────┴──────────────┴─────────────────────┐│
                                                              ││
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────────────┐ ││
│  │  │                  │  │                  │  │                            │ ││
│  │  │  TIER 1:         │  │  TIER 2:         │  │  TIER 3:                   │ ││
ce   │  │  CTO Enterprise  │  │  CTO Platform              │ ││
│  │  │  ──────────────  │  │  ──────────────  │  │  ────────────              │ ││
│  │  │                  │  │                  │  │                            │ ││
ISO  │  │  • Multi-node    │  │  • Helm charts only        │ ││
│  │  │  • Single node   │  │  • HA control    │  │  • BYOK (Bring Your        │ ││
│  │  │  • Talos Linux   │  │    plane         │  │    Own Kubernetes)         │ ││
│  │  │  • Zero-to-prod  │  │  • Node scaling  │  │  • Any K8s distro          │ ││
│  │  │  • OTA updates   │  │  • Distributed   │  │  • GitOps deployment       │ ││
│  │  │                  │  │    storage       │  │  • Self-managed infra      │ ││
│  │  │                  │  │  • Enterprise    │  │                            │ ││
│  │  │  Target:         │  │    support       │  │  Target:                   │ ││
│  │  │  Small teams,    │  │                  │  │  DevOps teams with         │ ││
│  │  │  startups,       │  │  Target:         │  │  existing K8s expertise    │ ││
│  │  │  edge deploys    │  │  Growing orgs,   │  │  and infrastructure        │ ││
│  │  │                  │  │  enterprises     │  │                            │ ││
│  │  │  MVP ✓           │  │  Post-MVP        │  │  Post-MVP                  │ ││
│  │  │                  │  │                  │  │                            │ ││
│  │  └──────────────────┘  └──────────────────┘  └────────────────────────────┘ ││
│  │                                                                              ││
│  └──────────────────────────────────────────────────────────────────────────────┘│
│                                                                                  │
│  Upgrade Paths:                                                                  │
│  ─────────────                                                                   │
│  • Appliance → Enterprise: Add nodes to existing cluster                        │
│  • Platform → Appliance: Migration tooling (export/import)                      │
│  • Platform → Enterprise: Not direct; would require Appliance migration         │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Tier 1: CTO Appliance (MVP Focus)

**Delivery**: Bootable ISO
**Target**: Non-DevOps users, small teams, edge deployments
**Includes**:
- Talos Linux (immutable OS)
- Single-node Kubernetes
- Full platform stack
- OTA update operator
- Setup wizard

### Tier 2: CTO Enterprise (Post-MVP)

**Delivery**: Appliance upgrade + node enrollment
**Target**: Growing organizations, enterprises
**Includes**:
- Everything in Appliance
- Multi-node cluster support
- HA control plane (3+ nodes)
- Distributed storage (Longhorn replication)
- Node discovery and enrollment
- Enterprise support SLAs

### Tier 3: CTO Platform (Post-MVP)

**Delivery**: Helm charts + documentation
**Target**: DevOps teams with existing Kubernetes
**Includes**:
- CTO platform components only
- Helm charts with configurable values
- GitOps-ready manifests
- Compatibility with major K8s distributions
- Self-managed infrastructure

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              Platform-in-a-Box                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Bootable ISO                                        │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  ┌───────────────┐  │ │
│  │  │  Bootstrap   │  │   Talos      │  │   Platform    │  │   Container   │  │ │
│  │  │  Environment │  │   Installer  │  │   Manifests   │  │   Images      │  │ │
│  │  │  (initramfs) │  │   Image      │  │   (GitOps)    │  │   (air-gap)   │  │ │
│  │  └──────────────┘  └──────────────┘  └───────────────┘  └───────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                         │                                        │
│                                         ▼                                        │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Talos Linux                                         │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  Immutable OS │ API-Driven │ containerd │ Kubernetes Components      │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                         │                                        │
│                                         ▼                                        │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Kubernetes Cluster                                  │ │
│  │                                                                             │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐ │ │
│  │  │  Core Platform  │  │  Storage Layer  │  │  Observability              │ │ │
│  │  │  ─────────────  │  │  ─────────────  │  │  ─────────────              │ │ │
│  │  │  ArgoCD         │  │  Longhorn       │  │  Prometheus                 │ │ │
│  │  │  Vault          │  │  MinIO          │  │  Grafana                    │ │ │
│  │  │  CNPG           │  │  Local Path     │  │  Loki                       │ │ │
│  │  │  Cert-Manager   │  │  Provisioner    │  │  Alertmanager               │ │ │
│  │  │  Ingress-NGINX  │  │                 │  │                             │ │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘ │ │
│  │                                                                             │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐ │ │
│  │  │  CTO Platform   │  │  OTA Updates    │  │  User Workloads             │ │ │
│  │  │  ─────────────  │  │  ─────────────  │  │  ─────────────              │ │ │
│  │  │  Agent Ctrl     │  │  Update         │  │  Applications               │ │ │
│  │  │  MCP Server     │  │  Operator       │  │  deployed via               │ │ │
│  │  │  Workflows      │  │  License Svc    │  │  GitOps/UI                  │ │ │
│  │  │  Agents         │  │  Health Mon     │  │                             │ │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘ │ │
│  │                                                                             │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. Bootstrap Environment

The bootstrap environment is a minimal Linux system that runs from RAM during installation.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Bootstrap Environment                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Setup Wizard                          │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │    │
│  │  │  Web UI     │  │  Config     │  │  Hardware       │  │    │
│  │  │  (HTTP/S)   │  │  Generator  │  │  Detection      │  │    │
│  │  │             │  │             │  │                 │  │    │
│  │  │  - Wizard   │  │  - Talos    │  │  - Disk scan    │  │    │
│  │  │    screens  │  │    machconf │  │  - NIC detect   │  │    │
│  │  │  - Progress │  │  - Helm     │  │  - RAM/CPU      │  │    │
│  │  │  - Logs     │  │    values   │  │  - UEFI/BIOS    │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Installation Engine                   │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │    │
│  │  │  Disk       │  │  Talos      │  │  Bootstrap      │  │    │
│  │  │  Manager    │  │  Installer  │  │  Orchestrator   │  │    │
│  │  │             │  │             │  │                 │  │    │
│  │  │  - Partition│  │  - Write    │  │  - Sequence     │  │    │
│  │  │  - Format   │  │    image    │  │    control      │  │    │
│  │  │  - Mount    │  │  - Apply    │  │  - Health       │  │    │
│  │  │             │  │    config   │  │    checks       │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Network Services                      │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │    │
│  │  │  DHCP       │  │  mDNS       │  │  Console        │  │    │
│  │  │  Client     │  │  Advertiser │  │  Fallback       │  │    │
│  │  │             │  │             │  │                 │  │    │
│  │  │  Auto-      │  │  Broadcasts │  │  Text UI if     │  │    │
│  │  │  configure  │  │  cto-setup  │  │  web unavail    │  │    │
│  │  │  network    │  │  .local     │  │                 │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Technology Stack:**
- Base: Alpine Linux or custom minimal initramfs
- Web Server: Go with embedded static files (single binary)
- mDNS: Avahi or Go implementation
- Disk Tools: parted, mkfs utilities
- Talos: talosctl embedded

### 1b. Setup Experience Options

Multiple setup interfaces for different skill levels and scenarios:

```
──────────────────────────────────────────┐
│                     Server Console Display                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│     CTO Platform - Setup Ready                                   │
│     ══════════════════════════                                   │
│                                                                  │
────────────────────────────┐     │
│     │                                                      │     │
│     │   OPTION A: Web Browser (Recommended)               │     │
│     │   Open https://192.168.1.105 or cto-setup.local     │     │
│     │                                                      │     │
│     │   OPTION B: Scan QR Code                            │     │
│     │   ▄▄▄▄▄▄▄ ▄▄▄   ▄▄▄▄▄▄▄                             │     │
│     │   █ ▄▄▄ █ ▄█▄▀█ █ ▄▄▄ █   Scan to setup            │     │
│     │   █ ███ █ █▄█▀▄ █ ███ █   from your phone          │     │
│     │   █▄▄▄▄▄█ ▄▀▄ █ █▄▄▄▄▄█                             │     │
│     │                                                      │     │
p (TUI)                     │     │
│     │   Press [ENTER] for text-based setup                │     │
                             │     │
│     │   OPTION D: CLI Setup (Advanced)                    │     │
│     │   Press [F3] for command-line setup                 │     │
│     │                                                      │     │
│     └─────────────────────────────────────────────────────┘     │
│                                                                  │
│     Network: eth0 = 192.168.1.105 (DHCP)                        │
...                               │
│                                                                  │
│     [F2] Network Config   [F3] CLI Mode   [F10] Shell           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘


#### Option A: Web Wizard (vSphere-style)

**Best for:** Most users, non-technical teams
**Requires:** Second device with web browser

```
┌─────────────────────────────────────────────────────────────────┐
│  CTO Platform Setup                              Step 2 of 7    │
──────────────────────────────────────────────────────┤
│                                                                  │
│  Organization Setup                                              │
────────                                               │
│                                                                  │
│  Organization Name                                               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Acme Corporation                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Admin Email                                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
com                                           │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Password                              Confirm Password          │
│  ┌────────────────────────┐            ┌────────────────────┐   │
│  │ ••••••••••••           │            │ ••••••••••••       │   │
│  └────────────────────────┘            └────────────────────┘   │
ngth: ████████░░ Strong                           │
│                                                                  │
│                                                                  │
│  ┌─────────┐                                    ┌──────────┐    │
│  │  Back   │                                    │  Next →  │    │
│  └─────────┘                                    └──────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Option B: QR Code / Mobile Setup

**Best for:** Quick setup, users without nearby computer
**Requires:** Smartphone with camera

```
Flow:
1. Console displays QR code containing setup URL
2. User scans with phone camera
3. Phone browser opens setup wizard
4. Setup completes on phone
5. Server receives config via:
   - Direct local network (if phone on same network)
   - Cloud relay (if different networks, temporary tunnel)

QR Code Options:
┌────────────────────────────────────────┐
│                                        │
│  Option B1: Local Network              │
│  QR → https://192.168.1.105/setup     │
│  (Phone must be on same network)       │
│                                        │
│  Option B2: Cloud Relay                │
│  QR → https://setup.5dlabs.io/abc123  │
│  (Works from any network)              │
│  (Temporary secure tunnel)             │
│                                        │
└────────────────────────────────────────┘
```

#### Option C: Text User Interface (TUI)

**Best for:** Servers without network access, headless setup
**Requires:** Keyboard + monitor attached to server

```
┌─────────────────────────────────────────────────────────────────┐
│                  CTO Platform Setup - TUI                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │    Network Configuration                                │    │
│  │    ─────────────────────                                │    │
│  │                                                          │    │
│  │    Interface: eth0                                       │    │
│  │                                                          │    │
│  │    (●) DHCP - Automatic configuration                   │    │
│  │    ( ) Static IP                                         │    │
│  │                                                          │    │
│  │    Current IP: 192.168.1.105                            │    │
│  │    Gateway:    192.168.1.1                              │    │
│  │    DNS:        192.168.1.1                              │    │
│  │                                                          │    │
│  │                                                          │    │
│  │    [Tab] Switch field  [Enter] Select  [Esc] Back       │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐                  │
│  │   Back   │    │   Skip   │    │   Next   │                  │
│  └──────────┘    └──────────┘    └──────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Option D: CLI Setup (Advanced)

**Best for:** DevOps engineers, automation, scripting
**Requires:** Keyboard access or SSH (post-network-config)

```bash
# Interactive CLI setup
$ cto-setup init

CTO Platform Setup CLI v1.0.0
─────────────────────────────

? License key: lic_abc123...
✓ License valid: Team tier, 3 nodes

? Organization name: Acme Corp
? Admin email: admin@acme.com
? Admin password: ••••••••••••
? Confirm password: ••••••••••••

? Network configuration:
  ❯ DHCP (auto)
    Static IP

? Select installation disk:
  ❯ /dev/nvme0n1 (1TB NVMe Samsung 980 Pro)
    /dev/sda (500GB SSD)

? Enable telemetry? (Y/n): Y

Review configuration:
─────────────────────
Organization: Acme Corp
Admin: admin@acme.com
Network: DHCP (eth0)
Disk: /dev/nvme0n1
Telemetry: Enabled

? Proceed with installation? (Y/n): Y

Installing... ████████████████████░░░░░░░░░░ 65%
```

```bash
# Non-interactive (for automation)
$ cto-setup init \
    --license "lic_abc123..." \
    --org "Acme Corp" \
    --admin-email "admin@acme.com" \
    --admin-password-file /tmp/password \
    --network dhcp \
    --disk /dev/nvme0n1 \
    --telemetry full \
    --yes
```

#### Option E: Pre-Configured ISO (Enterprise)

**Best for:** Enterprise deployments, multi-site rollouts
**Requires:** Customer portal access

```
Customer Portal Flow:
┌─────────────────────────────────────────────────────────────────┐
│  Generate Custom ISO                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Organization: Acme Corp                                        │
│  License: lic_abc123 (Enterprise, unlimited nodes)              │
│                                                                  │
│  Pre-configure:                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ ☑ Organization name: Acme Corp                          │    │
│  │ ☑ Admin email: admin@acme.com                           │    │
│  │ ☑ Network: Static IP 10.0.1.50/24, GW 10.0.1.1         │    │
│  │ ☑ DNS: 10.0.1.10, 10.0.1.11                            │    │
│  │ ☑ Domain: cto.acme.internal                             │    │
│  │ ☑ NTP: ntp.acme.internal                                │    │
│  │ ☐ Proxy settings                                        │    │
│  │ ☑ Telemetry: Full (enterprise support)                  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ISO Type:                                                       │
│  (●) Connected (~2GB) - pulls images during install             │
│  ( ) Air-gapped (~30GB) - all images bundled                    │
│                                                                 
│                        [ Download Custom ISO ]                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Result: Zero-touch installation
- Boot from USB
- Installation starts automatically  
- No prompts, no interaction needed
- Platform ready in ~30 minutes
```

#### Setup Experience Matrix

| Aspect | Web Wizard | QR/Mobile | TUI | CLI | Pre-Config |
|--------|------------|-----------|-----|-----|------------|
| **Skill level** | Beginner | Beginner | Intermediate | Advanced | Any |
| **Second device needed** | Yes | Phone | No | No | No |
| **Network required** | Yes | Depends | No | No | No |
| **Air-gap compatible** | No | No | Yes | Yes | Yes |
| **Automatable** | No | No | No | Yes | Yes |
| **Best UX** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Enterprise ready** | ✓ | ✗ | ✓ | ✓ | ✓✓✓ |

### 2. Hardware Detection & Dynamic Config Generation

The bootstrap environment performs a comprehensive hardware scan to dynamically 
generate the appropriate Talos configuration.

```
┌─────────────────────────────────────────────────────────────────┐
│                 Hardware Detection Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Bootstrap Phase: Hardware Scan                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Detected Hardware                                       │    │
│  │  ═══════════════════                                     │    │
│  │                                                          │    │
│  │  CPU:                                                    │    │
│  │  ├─ Architecture: x86_64 (amd64)                        │    │
│  │  ├─ Model: Intel Xeon Silver 4410Y                      │    │
│  │  ├─ Cores: 12 physical, 24 logical                      │    │
│  │  ├─ Features: AVX-512, AES-NI, VMX                      │    │
│  │  └─ Microcode: 0x2b000461                               │    │
│  │                                                          │    │
│  │  Memory:                                                 │    │
│  │  ├─ Total: 64 GB DDR5 ECC                               │    │
│  │  ├─ DIMMs: 4x 16GB @ 4800MHz                            │    │
│  │  └─ NUMA nodes: 1                                       │    │
│  │                                                          │    │
│  │  Storage:                                                │    │
│  │  ├─ /dev/nvme0n1: Samsung 980 Pro 1TB (NVMe)           │    │
│  │  │   └─ SMART: Healthy, 98% life remaining              │    │
│  │  ├─ /dev/sda: Dell BOSS-N1 240GB (Boot)                │    │
│  │  │   └─ SMART: Healthy                                  │    │
│  │  └─ Controller: Intel VMD, HBA mode                     │    │
│  │                                                          │    │
│  │  Network:                                                │    │
│  │  ├─ eth0: Broadcom BCM57416 10GbE (connected)          │    │
│  │  │   └─ MAC: aa:bb:cc:dd:ee:ff                          │    │
│  │  ├─ eth1: Broadcom BCM57416 10GbE (no link)            │    │
│  │  └─ eno1: Intel I350 1GbE (iDRAC shared)               │    │
│  │                                                          │    │
│  │  Platform:                                               │    │
│  │  ├─ Vendor: Dell Inc.                                   │    │
│  │  ├─ Model: PowerEdge R660                               │    │
│  │  ├─ BIOS: 1.6.2 (UEFI, Secure Boot capable)            │    │
│  │  ├─ BMC: iDRAC9 Enterprise                              │    │
│  │  └─ Serial: XXXXXXX                                     │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Hardware Profile Matching                                       │
─────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Known Hardware Profiles:                                │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ ✓ dell-poweredge-r660    (MATCHED)                  ││    │
│  │  │   dell-poweredge-r760                               ││    │
│  │  │   hpe-proliant-dl360-gen11                          ││    │
│  │  │   supermicro-sys-1029p                              ││    │
│  │  │   generic-x86_64                                    ││    │
│  │  │   generic-arm64                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  Profile provides:                                       │    │
│  │  • Optimized kernel parameters                          │    │
│  │  • Known-good driver configuration                      │    │
│  │  • Storage controller settings                          │    │
│  │  • Network interface naming                             │    │
│  │  • Power management settings                            │    │
│  │  • BMC/IPMI integration                                 │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Dynamic Talos Config Generator                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Inputs:                                                 │    │
│  │  • Hardware profile (matched or generic)                │    │
│  │  • Detected hardware specs                              │    │
│  │  • User wizard selections                               │    │
│  │  • License tier (affects features)                      │    │
│  │                                                          │    │
│  │  Generated Config Sections:                              │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ machine:                                            ││    │
│  │  │   install:                                          ││    │
│  │  │     disk: /dev/nvme0n1  # Auto-selected best disk   ││    │
│  │  │     image: ghcr.io/.../installer:v1.8.0-amd64       ││    │
│  │  │   network:                                          ││    │
│  │  │     interfaces:                                     ││    │
│  │  │       - interface: eth0  # First connected NIC      ││    │
│  │  │         dhcp: true                                  ││    │
│  │  │   kubelet:                                          ││    │
│  │  │     extraArgs:                                      ││    │
│  │  │       system-reserved: memory=4Gi,cpu=1000m         ││    │
│  │  │       # Auto-calculated based on 64GB RAM           ││    │
│  │  │   kernel:                                           ││    │
│  │  │     modules:                                        ││    │
│  │  │       - name: nvme  # For NVMe storage              ││    │
│  │  │       - name: ixgbe # For Intel 10GbE               ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Hardware Detection Tools:**

```go
// Hardware detection module structure
type HardwareInfo struct {
    CPU       CPUInfo
    Memory    MemoryInfo
    Storage   []StorageDevice
    Network   []NetworkInterface
    Platform  PlatformInfo
    Profile   string  // Matched hardware profile
}

type CPUInfo struct {
    Architecture  string   // x86_64, aarch64
    Model         string
    Cores         int
    Threads       int
    Features      []string // AVX, AES-NI, etc.
}

type StorageDevice struct {
    Path        string   // /dev/nvme0n1
    Type        string   // nvme, sata, sas
    Size        uint64
    Model       string
    Serial      string
    SMARTHealth string
    Rotational  bool     // true = HDD, false = SSD
}

// Auto-select best installation disk
func SelectInstallDisk(devices []StorageDevice) StorageDevice {
    // Priority:
    // 1. NVMe SSD (fastest)
    // 2. SATA/SAS SSD
    // 3. Largest available
    // 4. Skip disks with existing data (warn user)
}

// Calculate resource reservations based on hardware
func CalculateReservations(hw HardwareInfo) ResourceReservations {
    // Scale reservations based on available resources
    // Minimum: 2GB system + 1GB kube (32GB RAM)
    // Scale up for larger systems
}
```

**Architecture Support Roadmap:**

```
┌─────────────────────────────────────────────────────────────────┐
│                 Architecture Support Roadmap                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  v1.0 (MVP):                                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  x86_64 (amd64) - Intel/AMD                             │    │
│  │  • Dell PowerEdge R660/R760                             │    │
│  │  • HPE ProLiant DL360 Gen11                             │    │
│  │  • Generic x86_64 fallback                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  v1.2:                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  x86_64 expanded                                        │    │
│  │  • Supermicro servers                                   │    │
│  │  • Lenovo ThinkSystem                                   │    │
│  │  • Intel NUC (dev/edge)                                 │    │
│  │  • Mini PCs (Minisforum, Beelink)                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  v1.3:                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  aarch64 (ARM64)                                        │    │
│  │  • Ampere Altra (cloud-native ARM)                      │    │
│  │  • AWS Graviton-compatible                              │    │
│  │  • Raspberry Pi 5 (edge/dev)                           │    │
│  │  • NVIDIA Jetson (AI edge)                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  v2.0:                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Community hardware profiles                            │    │
│  │  • User-contributed profiles via GitHub                 │    │
│  │  • Automatic detection improvements                     │    │
│  │  • RISC-V (experimental)                               │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Generic Fallback Mode:**

For unrecognized hardware, the system uses intelligent defaults:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Generic Hardware Mode                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ⚠️ Unrecognized Hardware                                       │
│                                                                  │
│  Your hardware (Vendor: Unknown, Model: Custom Build) is not    │
│  in our tested hardware list. Installation will proceed with    │
│  generic settings.                                               │
│                                                                  │
│  Detected Configuration:                                         │
│  • CPU: x86_64, 8 cores                                         │
│  • RAM: 32 GB                                                   │
│  • Disk: /dev/sda (500GB SSD)                                   │
│  • Network: eth0 (1GbE, connected)                              │
│                                                                  │
│  Generic mode uses:                                              │
│  • Conservative kernel parameters                               │
│  • Standard driver set                                          │
│  • Safe resource reservations                                   │
│                                                                  │
│  [Continue with Generic Mode]  [Report Hardware for Support]    │
│                                                                  │
│  □ Help improve CTO Platform by submitting hardware info        │
│    (No personal data, only hardware specs)                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2b. Talos Machine Configuration

The setup wizard generates Talos machine configuration based on user inputs and detected hardware.

```yaml
# Example generated machine configuration (simplified)
version: v1alpha1
machine:
  type: controlplane  # Single node acts as both
  token: <generated>
  
  network:
name: cto-platform
    interfaces:
      - interface: eth0
        dhcp: true  # Or static based on wizard
        
  install:
    disk: /dev/sda
    image: ghcr.io/siderolabs/installer:v1.8.0
    bootloader: true
    wipe: true
    
t:
    extraArgs:
stem-reserved: memory=2Gi,cpu=500m
      kube-reserved: memory=1Gi,cpu=500m
      
  # Single-node specific: allow workloads on control plane
  allowSchedulingOnControlPlane: true
  
cluster:
rName: cto-platform
  controlPlane:
    endpoint: https://cto-platform:6443
    
  network:

      name: cilium
    podSubnets:
      - 10.244.0.0/16
    serviceSubnets:
      - 10.96.0.0/12
      
  # Bootstrap ArgoCD via inline manifests
  inlineManifests:
    - name: argocd-namespace
      contents: |
        apiVersion: v1
        kind: Namespace
        metadata:
          name: argocd


### 3. OTA Update Operator

The update operator manages platform updates without user intervention.

```
──────────────────────────────────────────────────────────────┐
│                    OTA Update Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────┐       ┌──────────────────────────┐   │
│  │   Update Operator    │       │   5D Labs Update Server  │   │
│  │   (in-cluster)       │       │   (cloud-hosted)         │   │
                      │       │                          │   │
│  │  ┌────────────────┐  │       │  ┌────────────────────┐  │   │
│  │  │ Update         │◄─┼───────┼──│ Release Manifest   │  │   │
│  │  │ Controller     │  │       │  │ Registry           │  │   │
│  │  └────────────────┘  │       │  └────────────────────┘  │   │
│  │          │           │       │                          │   │
│  │          ▼           │       │  ┌────────────────────┐  │   │
│  │  ┌────────────────┐  │       │  │ Container Image    │  │   │
│  │  │ Scheduler      │  │       │  │ Mirror             │  │   │
  │ (maint window) │  │       │  └────────────────────┘  │   │
│  │  └────────────────┘  │       │                          │   │
│  │          │           │       │  ┌────────────────────┐  │   │
│  │          ▼           │       │  │ License            │  │   │
│  │  ┌────────────────┐  │       │  │ Validation         │  │   │
│  │  │ Upgrade        │  │       │  └────────────────────┘  │   │
│  │  │ Executor       │  │       │                          │   │
│  │  └────────────────┘  │       └──────────────────────────┘   │
│  │          │           │                                       │
│  │          ▼           │       ┌──────────────────────────┐   │
│  │  ┌────────────────┐  │       │   Air-Gap Alternative    │   │
│  │  │ Rollback       │  │       │                          │   │
│  │  │ Manager        │  │       │   USB/Download:          │   │
│  │  └────────────────┘  │       │   - Signed update bundle │   │
│  │                      │       │   - Apply via CLI/UI     │   │
│  └──────────────────────┘       └──────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Update CRD:**

```yaml
apiVersion: cto.5dlabs.io/v1alpha1
kind: PlatformUpdate
metadata:
  name: platform-update-config
  namespace: cto-system
spec:
  # Update channel
  channel: stable  # stable | beta | edge
  
  # Automatic updates
  autoUpdate:
    enabled: true
    maintenanceWindow:
      start: "02:00"
      end: "05:00"
      timezone: "UTC"
      daysOfWeek: ["Saturday", "Sunday"]
  
  # Update server configuration
  updateServer:
 https://updates.5dlabs.io
    # For air-gapped: use local server or disable

  # Notification settings
  notifications:
    email:
      enabled: true
      recipients: ["admin@example.com"]
    webhook:
      enabled: false
      url: ""
      
  # Rollback configuration
ck:
    automatic: true
thCheckTimeout: 300  # seconds
    
status:
tVersion: "1.2.3"
  availableVersion: "1.2.4"
  lastCheck: "2025-11-27T10:00:00Z"
date: "2025-11-20T03:15:00Z"
  state: UpToDate  # UpToDate | UpdateAvailable | Updating | RollingBack | Failed
  history:
    - version: "1.2.3"
      appliedAt: "2025-11-20T03:15:00Z"
atus: Success
    - version: "1.2.2"
      appliedAt: "2025-11-01T03:22:00Z"
      status: Success


**Update Flow:**

```
1. Check for Updates
   └─► Query update server with current version and license
   └─► Receive available updates manifest

2. Pre-Stage (Background)
   └─► Download container images
   └─► Download Talos upgrade image if needed
   └─► Verify signatures on all artifacts

3. Wait for Maintenance Window
   └─► Respect configured schedule
   └─► Or allow manual trigger

4. Execute Update
─► Create backup snapshot
   └─► Update Helm releases via ArgoCD
   └─► If Talos update needed:
       └─► Call Talos upgrade API
       └─► Node reboots into new image
   └─► Run health checks

5. Verify & Finalize
   └─► All pods healthy?
   └─► API server responding?
   └─► Core services operational?
─► If YES: Mark complete, notify admin
   └─► If NO: Trigger automatic rollback

ollback (if needed)
   └─► Restore previous Helm releases
   └─► Talos automatic fallback to previous image
   └─► Notify admin of failure
```

### 4. Agent Configuration System (BYOK)

The platform is fully agnostic - users bring their own API keys and choose their CLIs/models.


┌─────────────────────────────────────────────────────────────────┐
│                Agent Configuration Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  GitHub: 5dlabs/cto-catalog                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  cli-catalog.json                                        │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ {                                                   ││    │
│  │  │   "clis": [                                         ││    │
│  │  │     {                                               ││    │
│  │  │       "id": "cursor",                               ││    │
│  │  │       "name": "Cursor CLI",                         ││    │
│  │  │       "version": "0.44.x",                          ││    │
│  │  │       "install": "npm install -g @anthropic-ai/...",││    │
│  │  │       "supported_models": ["claude-*", "gpt-*"]     ││    │
│  │  │     },                                              ││    │
│  │  │     {                                               ││    │
d": "claude-code",                          ││    │
│  │  │       "name": "Claude Code",                        ││    │
│  │  │       "version": "1.x",                             ││    │
│  │  │       "install": "npm install -g @anthropic-ai/...",││    │
│  │  │       "supported_models": ["claude-*"]              ││    │
│  │  │     },                                              ││    │
│  │  │     { "id": "aider", ... },                         ││    │
│  │  │     { "id": "windsurf", ... },                      ││    │
│  │  │     { "id": "continue", ... }                       ││    │
│  │  │   ]                                                 ││    │
│  │  │ }                                                   ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  model-catalog.json                                      │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ {                                                   ││    │
│  │  │   "providers": [                                    ││    │
│  │  │     {                                               ││    
│  │  │       "id": "anthropic",                            ││    │
│  │  │       "name": "Anthropic",                          ││    │
│  │  │       "key_env": "ANTHROPIC_API_KEY",               ││    │
│  │  │       "models": [                                   ││    │
│  │  │         { "id": "claude-sonnet-4", "name": "..." }, ││    │
│  │  │         { "id": "claude-opus-4", "name": "..." }    ││    │
                                            ││    │
│  │  │     },                                              ││    │
│  │  │     { "id": "openai", "models": [...] },            ││    │
│  │  │     { "id": "google", "models": [...] },            ││    │
│  │  │     { "id": "ollama", "models": [...] }             ││    │
│  │  │   ]                                                 ││    │
│  │  │ }                                                   ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              │ Periodic sync                     │
│                              ▼                                   │
│  Platform (in-cluster)                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Catalog Sync Service                                    │    │
│  │  - Pulls from GitHub daily (or on-demand)               │    │
│  │  - Caches locally for air-gap scenarios                 │    │
│  │  - Validates catalog integrity (signed)                 │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Configuration UI (Dashboard)                            │    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  Agent: Rex (Backend Developer)                     ││    │
│  │  │                                                     ││    │
│  │  │  CLI:    [Cursor CLI        ▼]                      ││    │
 [Claude Sonnet 4   ▼]                      ││    │
│  │  │                                                     ││    │
│  │  │  API Key: [••••••••••••••••••] [Test] [Rotate]      ││    │
│  │  │  Status:  ✓ Connected                               ││    │
│  │  │                                                     ││    │
│  │  │  [Save Configuration]                               ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  API Keys stored in:                                     │    │
│  │  ┌───────────────────┐                                  │    │
│  │  │  Vault            │  secret/cto/agents/rex           │    │
│  │  │                   │  - ANTHROPIC_API_KEY             │    │
│  │  │                   │  - cli_preference: cursor        │    │
│  │  │                   │  - model_preference: sonnet-4    │    │
│  │  └───────────────────┘                                  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Key Design Principles:**

1. **No Vendor Lock-in**: Platform doesn't care which CLI or model you use
2. **Secure Key Storage**: All API keys in Vault, never in ConfigMaps
3. **Live Updates**: Catalog updates without platform upgrade
4. **Offline Support**: Cached catalog works in air-gapped environments
5. **Per-Agent Config**: Each agent can use different CLI/model combinations

**Configuration Flow:**

```
                     System Response
───────────────────────────────────────────────────────────────────
1. Open Agent Settings        →      Load available CLIs/models from catalog
2. Select CLI (e.g., Cursor)  →      Filter to compatible models
3. Select Model (e.g., Sonnet)→      Show API key requirements
4. Enter API Key              →      Store encrypted in Vault
5. Click "Test"               →      Make test API call, verify connectivity
6. Save                       →      Update agent ConfigMap, restart if needed
```

: Custom Agent Builder (v1.2+)**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Custom Agent Builder (Future)                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Create New Agent                                        │    │
                                              │    │
│  │  Name:        [My Custom Agent            ]              │    │
│  │  Role:        [Security Reviewer          ]              │    │
n: [Reviews PRs for security...]              │    │
│  │                                                          │    │
│  │  Base Template: [○ Rex  ○ Blaze  ○ Cypher  ● Blank]     │    │
                                              │    │
│  │  System Prompt:                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
a security-focused code reviewer...         ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
                                              │    │
│  │  Tools: [✓] GitHub  [✓] MCP  [ ] Database  [✓] Shell    │    │
│  │                                                          │    │
                                              │    │
│  │  [✓] On PR opened                                        │    │
│  │  [ ] On commit                                           │    │
] Scheduled                                           │    │
│  │  [ ] Manual only                                         │    │
│  │                                                          │    │
│  │  [Save as Template]  [Deploy Agent]                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘


### 5. Platform Guardian Agent (AI-Powered DevOps)

vOps agent that monitors observability data and dynamically manages resources.

```
┌─────────────────────────────────────────────────────────────────┐
│                   Platform Guardian Architecture                 │
├─────────────────────────────────────────────────────────────────┤
                                                   │
│  Observability Stack (Data Sources)                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Prometheus  │  │    Loki     │  │   Kubernetes Events     │  │
│  │ (metrics)   │  │   (logs)    │  │   (state changes)       │  │
│  │             │  │             │  │                         │  │
/Memory  │  │ Error logs  │  │ Pod lifecycle           │  │
k I/O    │  │ App logs    │  │ Node conditions         │  │
work     │  │ Audit logs  │  │ Resource changes        │  │
│  │ Latency     │  │             │  │                         │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                      │                │
│         └────────────────┼──────────────────────┘                │
│                          ▼                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
                Guardian Agent                        │    │
│  │              (AI-Powered DevOps Agent)                   │    │
│  │                                                          │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │    │
 Collector  │  │  AI Engine  │  │  Executor       │  │    │
│  │  │  ─────────  │  │  ─────────  │  │  ────────       │  │    │
            │  │             │  │                 │  │    │
│  │  │  Queries:   │  │  Analyzes:  │  │  Actions:       │  │    │
 - PromQL   │  │  - Patterns │  │  - kubectl      │  │    │
 - LogQL    │  │  - Trends   │  │  - Helm         │  │    │
│  │  │  - K8s API  │  │  - Anomalies│  │  - Talos API    │  │    │
│  │  │             │  │  - Root     │  │  - ArgoCD       │  │    │
 Streams:   │  │    cause    │  │                 │  │    │
│  │  │  - Alerts   │  │  - Predict  │  │  Decides:       │  │    │
│  │  │  - Events   │  │    issues   │  │  - Scale up/down│  │    │
│  │  │  - Metrics  │  │             │  │  - Restart      │  │    │
│  │  │             │  │  Uses LLM   │  │  - Rebalance    │  │    │
            │  │  for complex│  │  - Clean up     │  │    │
│  │  │             │  │  analysis   │  │  - Alert human  │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │    │
│  │         │                │                │              │    │
     └────────────────┼────────────────┘              │    │
│  │                          ▼                               │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │                 Decision Engine                      ││    │
│  │  │                                                      ││    │
│  │  │  Rules:                                              ││    │
│  │  │  - Pod CrashLoopBackOff > 5 min → Restart with      ││    │
│  │  │    increased resources                               ││    │
│  │  │  - Cert expires < 7 days → Trigger renewal          ││    │
│  │  │  - Disk > 85% → Clean old logs, notify if > 95%     ││    │
│  │  │  - Node NotReady > 2 min → Cordon, alert            ││    │
│  │  │  - PVC pending > 5 min → Check storage, notify      ││    │
│  │  │  - OOM kills detected → Increase limits, warn       ││    │
│  │  │  - CVE detected → Auto-patch if safe, else notify   ││    │
│  │  │                                                      ││    │
│  │  │  Safety Constraints:                                 ││    │
│  │  │  - Never delete user data                           ││    │
│  │  │  - Never modify user workloads without approval     ││    │
│  │  │  - Always create backup before destructive action   ││    │
  │  - Rate limit remediations (no infinite loops)      ││    │
│  │  │  - Escalate to human after N failed attempts        ││    │
────────────────────────────────────────────────────┘│    │
│  │                          │                               │    │
│  │                          ▼                               │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │                  Action Log                          ││    │
│  │  │                                                      ││    │
│  │  │  2025-11-27 02:15:03  Renewed cert for ingress      ││    │
│  │  │  2025-11-27 02:15:01  Cleaned 2.3GB old logs        ││    │
│  │  │  2025-11-26 14:22:17  Restarted vault-0 (OOMKilled) ││    │
  │  2025-11-26 14:22:18  Increased vault memory limit  ││    │
│  │  │  ...                                                 ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
─────────────────────────────────────────────────────────┘    │
│                          │                                       │
│                          ▼                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                 User Dashboard                           │    │
                                                          │    │
│  │  Platform Health: ████████████████████░░ 92% Healthy    │    │
│  │                                                          │    │
│  │  This Week:                                              │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
  │  ✓ Guardian handled 12 issues automatically         ││    │
│  │  │  ✓ 3 certificates renewed                           ││    │
│  │  │  ✓ 4.2 GB storage reclaimed                         ││    │
│  │  │  ⚠ 1 issue needs your attention                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  [View Details]  [Configure Guardian]                    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                       │
│                          │ (opt-in)                              │
│                          ▼                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                 5D Labs Telemetry                        │    │
│  │                                                          │    │
│  │  Receives (anonymized):                                  │    │
│  │  - Health scores over time                              │    │
│  │  - Common issue patterns                                │    │
│  │  - Remediation success rates                            │    │
│  │  - Component failure frequencies                        │    │
                                              │    │
│  │  Does NOT receive:                                       │    │
│  │  - Source code                                          │    │
│  │  - Secrets or credentials                               │    │
│  │  - Business data                                        │    │
│  │  - Personally identifiable information                  │    │
│  │                                                          │    │
│  │  Benefits:                                               │    │
│  │  - Proactive alerts for known issues                    │    │
│  │  - Faster support when issues escalate                  │    │
│  │  - Platform improvements for everyone                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Guardian Remediation Playbooks:**

| Issue | Detection | Auto-Fix | Escalation |
|-------|-----------|----------|------------|
| Pod CrashLoop | Restart count > 5 | Restart with more resources | After 3 restart attempts |
| Cert Expiring | < 7 days remaining | Trigger cert-manager renewal | If renewal fails |
| Disk Full | > 85% capacity | Clean logs, old images | If > 95% after cleanup |
| Node Unhealthy | NotReady > 2 min | Cordon, reschedule pods | Immediately for control plane |
OMKilled event | Increase memory limit | If keeps happening |
| DB Connection | Connection errors | Restart connection pool | If DB is actually down |
| Backup Failed | Backup job failed | Retry with backoff | After 3 retries |
| PVC Pending | Pending > 5 min | Check provisioner | If storage unavailable |
| CVE Detected | Trivy scan | Auto-patch if minor | Major CVEs always notify |
| Config Drift | Checksum mismatch | Reapply from GitOps | If reconciliation fails |

**User-Friendly Notifications:**

```
┌─────────────────────────────────────────────────────────────────┐
│  ✓ Guardian Auto-Fixed an Issue                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  What happened:                                                  │
│  Your database was running low on memory and crashed.           │
│                                                                  │
│  What Guardian did:                                              │
│  1. Restarted the database                                      │
│  2. Increased its memory allocation from 2GB to 4GB             │
│  3. Verified all your data is intact                            │
│                                                                  │
│  Impact:                                                         │
│  About 30 seconds of database unavailability at 2:15 AM.        │
│  No data was lost.                                              │
│                                                                  │
│  Recommendation:                                                 │
│  Your workload may benefit from the next tier up.               │
                                                  │
│                                                                  │
│                                            [Dismiss] [Details]  │
└─────────────────────────────────────────────────────────────────┘
```

### 6. License System

Cryptographic offline license validation:

```
┌─────────────────────────────────────────────────────────────────┐
│                    License Architecture                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  5D Labs Backend                     Customer Platform           │
│  ───────────────                     ─────────────────           │
│                                                                  │
│  ┌─────────────────┐                ┌─────────────────┐         │
│  │  License        │                │  License        │         │
│  │  Generator      │                │  Validator      │         │
│  │                 │                │                 │         │
│  │  Private Key    │    License     │  Public Key     │         │
│  │  (Ed25519)      │───────────────►│  (embedded)     │         │
│  │                 │    File        │                 │         │
│  └─────────────────┘                └─────────────────┘         │
│                                             │                    │
│                                             ▼                    │
│                                      ┌─────────────────┐         │
│                                      │  Features       │         │
│                                      │  Enabled Based  │         │
                       │  on License     │         │
│                                      └─────────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**License File Format:**

```json
{
  "version": 1,
  "payload": {
    "customer_id": "cust_abc123",
    "customer_name": "Acme Corporation",
    "email": "admin@acme.com",
    "issued_at": "2025-11-27T00:00:00Z",
    "expires_at": "2026-11-27T00:00:00Z",
    "features": [
      "core",
lity",
      "agents",
      "backup"

    "limits": {
      "max_nodes": 1,
paces": 10,
      "max_agents": 5
    },
r": "standard"
  },
  "signature": "base64-encoded-ed25519-signature"

```

ic (Rust/Go):**

```rust
ocode for license validation
pub fn validate_license(license_file: &str, public_key: &[u8]) -> Result<License> {
    let license: LicenseFile = serde_json::from_str(license_file)?;
    
    // Verify signature
    let payload_bytes = serde_json::to_vec(&license.payload)?;
    let signature = base64::decode(&license.signature)?;

    verify_ed25519(public_key, &payload_bytes, &signature)?;
    
    // Check expiration
ayload.expires_at < Utc::now() {
        return Err(LicenseError::Expired);
    }
    
    // Check node count (would query cluster)
    let current_nodes = get_node_count();
odes > license.payload.limits.max_nodes {
        return Err(LicenseError::NodeLimitExceeded);
    }
    
    Ok(License::from(license.payload))
}
```

### 5. Storage Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Storage Architecture                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
cal Disk Layout (Single Node)                              │
│  ──────────────────────────────────                              │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  /dev/sda (or nvme0n1)                                   │    │
│  │  ┌─────────┬────────────┬───────────────────────────┐   │    │
│  │  │  EFI    │   Talos    │       Data Partition      │   │    │
  │  512MB  │   OS 10GB  │       (remainder)         │   │    │
        │            │                           │   │    │
  │ /boot   │ /          │  Longhorn / Local Path    │   │    │
│  │  │ /efi    │ (immutable)│  /var/lib/longhorn        │   │    │
│  │  └─────────┴────────────┴───────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Kubernetes Storage Classes                                      │
│  ──────────────────────────                                      │
│                                                                  │
│  ┌─────────────────────┐  ┌─────────────────────────────────┐   │
│  │  longhorn (default) │  │  local-path                     │   │
│  │                     │  │                                 │   │
  - Replicated       │  │  - Single node only             │   │
│  │  - Snapshots        │  │  - No replication               │   │
  - Backup to MinIO  │  │  - Best performance             │   │
│  │  - Good for DBs     │  │  - Good for caches              │   │
│  └─────────────────────┘  └─────────────────────────────────┘   │
                                                               │
│  Object Storage (MinIO)                                          │
│  ──────────────────────                                          │
                                                               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  MinIO Tenant                                            │    │
│  │                                                          │    │
│  │  Buckets:                                                │    │
  - backups/        Velero backups, DB dumps              │    │
│  │  - artifacts/      Build artifacts, releases             │    │
│  │  - logs/           Long-term log retention               │    │
│  │  - user/           User-created buckets                  │    │
                                                          │    │
│  │  Single-node mode: erasure coding disabled               │    │
│  │  Multi-node mode: erasure coding for redundancy          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
──────────────────────────────────────────────────────────┘
```

### 6. Network Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Network Architecture                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  External Access                                                 │
│  ───────────────                                                 │
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
 │  Internet   │───►│  Router/    │───►│  Platform Node      │  │
│  │             │    │  Firewall   │    │                     │  │
│  └─────────────┘    └─────────────┘    │  External IP or     │  │
│                                        │  Port Forward:      │  │
│                            Ports:      │  - 443 (HTTPS)      │  │
│                            - 443       │  - 6443 (K8s API)   │  │
│                            - 6443      │                     │  │
│                                        └─────────────────────┘  │
│                                                                  │
│  Internal Kubernetes Networking                                  │
│  ──────────────────────────────                                  │
                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Cilium CNI                                              │    │
 │                                                          │    │
│  │  Pod Network: 10.244.0.0/16                              │    │
│  │  Service Network: 10.96.0.0/12                           │    │
│  │                                                          │    │
│  │  Features:                                               │    │
 │  - Network Policies (enabled by default)                 │    │
│  │  - eBPF-based routing (high performance)                 │    │
│  │  - Service mesh capabilities (optional)                  │    │
│  │  - Hubble observability (optional)                       │    │
│  └─────────────────────────────────────────────────────────┘    │
                                                                 │
│  Ingress                                                         │
│  ───────                                                         │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Ingress-NGINX                                           │    │
│  │                                                          │    │
│  │  - LoadBalancer service (or NodePort on bare metal)      │    │
│  │  - TLS termination                                       │    │
│  │  - Rate limiting                                         │    │
│  │  - Automatic cert-manager integration                    │    │
│  │                                                          │    │
│  │  Routes:                                                 │    │
│  │  - platform.example.com     → CTO Dashboard              │    │
│  │  - argocd.example.com       → ArgoCD UI                  │    │
│  │  - grafana.example.com      → Grafana                    │    │
│  │  - vault.example.com        → Vault UI                   │    │
│  │  - *.apps.example.com       → User applications          │    │
│  └─────────────────────────────────────────────────────────┘    │
                                                           │
│  DNS Options                                                     │
│  ───────────                                                     │
│                                                                  │
│  1. Custom Domain (recommended):                                 │
│     - User provides domain                                       │
│     - DNS A record points to platform IP                         │
│     - Let's Encrypt for certificates                             │
│                                                                  │
│  2. Local Access (default):                                      │
│     - mDNS: cto-platform.local                                   │
│     - Self-signed certificates                                   │
│     - /etc/hosts entry option                                    │
│                                                                  │
│  3. nip.io (automatic):                                          │
│     - Uses IP-based DNS: 192.168.1.100.nip.io                    │
│     - Works without DNS configuration                            │
│     - Self-signed or Let's Encrypt                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Installation Sequence


┌─────────────────────────────────────────────────────────────────┐
│                    Installation Sequence                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Phase 1: Boot & Discovery (0-2 min)                            │
│  ───────────────────────────────────                            │
│  1. BIOS/UEFI loads ISO boot sector                             │
│  2. GRUB/systemd-boot loads kernel + initramfs                  │
│  3. Bootstrap environment starts in RAM                          │
│  4. Network auto-configuration (DHCP)                            │
│  5. mDNS advertises cto-setup.local                             │
│  6. Web server starts on port 80/443                            │
│  7. Console displays access URL and IP                          │
│                                                                  │
│  Phase 2: Configuration (2-10 min, user-dependent)              │
│  ─────────────────────────────────────────────────              │
│  1. User accesses wizard via browser                            │
│  2. License validation                                          │
│  3. Organization & admin setup                                   │
│  4. Network configuration (DHCP or static)                      │
│  5. Storage selection and allocation                            │
│  6. Optional: domain, features, maintenance window              │
│  7. Review and confirm                                          │
│                                                                  │
│  Phase 3: OS Installation (5-10 min)                            │
│  ───────────────────────────────────                            │
│  1. Partition target disk                                        │
rmat partitions (EFI, BOOT, DATA)                         │
│  3. Write Talos image to disk                                   │
│  4. Generate machine configuration from wizard inputs           │
│  5. Write machine config to appropriate location                │
│  6. Configure bootloader                                        │
│  7. Unmount and reboot                                          │
│                                                                  │
│  Phase 4: Talos Bootstrap (3-5 min)                             │
│  ──────────────────────────────────                             │
│  1. Talos boots from disk                                       │
│  2. Applies machine configuration                               │
│  3. Starts containerd                                           │
│  4. Pulls Kubernetes components                                 │
│  5. Bootstraps etcd (single node)                               │
│  6. Starts API server, controller-manager, scheduler            │
│  7. Joins kubelet to cluster                                    │
│  8. Cluster is operational                                      │
│                                                                  │
│  Phase 5: Platform Deployment (10-20 min)                       │
│  ────────────────────────────────────────                       │
│  1. Apply initial ArgoCD manifests (inline in Talos config)     │
│  2. ArgoCD deploys App-of-Apps                                  │
│  3. Core services deploy in dependency order:                   │
│     a. cert-manager                                             │
│     b. ingress-nginx                                            │
│     c. Longhorn                                                 │
│     d. MinIO                                                    │
│     e. Vault (+ auto-unseal setup)                              │
│     f. CNPG                                                     │
│     g. Observability stack                                      │
│     h. CTO platform components                                  │
 OTA update operator                                      │
│  4. Health checks verify all components                         │
│  5. Admin user created in auth system                           │
│                                                                  │
│  Phase 6: Ready (total: 20-40 min)                              │
│  ─────────────────────────────────                              │
│  1. Installation complete notification                          │
│  2. Redirect to platform dashboard                              │
│  3. First-run wizard for additional setup                       │
│  4. Platform operational                                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
cto/
├── platform-installer/              # Platform-in-a-Box product

│   ├── bootstrap/                   # Bootstrap environment
│   │   ├── Dockerfile               # Build bootstrap image
├── Makefile                 # Build automation
│   │   │
│   │   ├── cmd/                     # Go binaries
│   └── wizard/              # Setup wizard
│   │   │       └── main.go
│   │   │
├── internal/                # Internal packages
│   │   │   ├── config/              # Configuration generation
│   │   │   │   ├── talos.go         # Talos machine config
│   │   └── helm.go          # Helm values generation
│   │   │   │
│   │   │   ├── hardware/            # Hardware detection
│   │   ├── disk.go          # Disk discovery
│   │   │   │   ├── network.go       # NIC discovery
│   │   │   │   └── system.go        # CPU, RAM, etc.
│   │   │
│   │   │   ├── installer/           # Installation engine
│   │   │   │   ├── partition.go     # Disk partitioning
│   │   │   │   ├── talos.go         # Talos installation
│   │   │   │   └── orchestrator.go  # Sequence control
│   │   │   │
│   │   │   ├── license/             # License validation
│   │   │   └── validator.go
│   │   │   │
│   │   │   └── web/                 # Web UI server
│   │   │       ├── server.go
│       ├── handlers.go
│   │   │       └── websocket.go     # Progress streaming
│   │   │
│   │   ├── web/                     # Frontend assets
│   │   │   ├── templates/           # Go templates
│   │   │   │   ├── layout.html
│   │   ├── welcome.html
│   │   │   │   ├── license.html
│   │   │   │   ├── organization.html
│   │   │   │   ├── network.html
│   │   │   │   ├── storage.html
│   │   │   │   ├── features.html
│   │   │   │   ├── review.html
│   │   │   └── progress.html
│   │   │   │
│   │   │   └── static/              # Static assets
│   │   │       ├── css/
│   │   │       ├── js/
│   │   │       └── images/
│   │   │
│   │   └── scripts/                 # Shell scripts
│       ├── partition.sh
│   │       ├── install-talos.sh
│   │       └── post-install.sh
│   │
│   ├── manifests/                   # GitOps manifests
│   │   ├── app-of-apps.yaml         # Root ArgoCD application
│   │   │
│   │   └── platform/                # Platform apps
│       ├── cert-manager.yaml
│   │       ├── ingress-nginx.yaml
│   │       ├── longhorn.yaml
│   │       ├── minio.yaml
│   │       ├── vault.yaml
│   │       ├── cnpg.yaml
│   │       ├── observability.yaml
│   │       ├── cto-platform.yaml
│   │       └── update-operator.yaml
│   │
│   ├── images/                      # Air-gap support
│   │   ├── image-list.txt           # All required images
│   │   └── pull-images.sh           # Script to bundle images
│   │
│   ├── iso-builder/                 # ISO build automation
│   │   ├── Dockerfile               # Build environment
│   │   ├── build-connected.sh       # Small ISO
│   │   ├── build-airgap.sh          # Large ISO with images
│   │   └── grub.cfg                 # Bootloader config
│   │
│   └── operator/                    # OTA Update Operator
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── controller.rs        # Kubernetes controller
│           ├── updater.rs           # Update execution
│           ├── talos.rs             # Talos API client
│           └── rollback.rs          # Rollback logic
│
├── update-server/                   # Backend services
│   ├── api/                         # Update API
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│       ├── releases.rs          # Release management
│   │       ├── licenses.rs          # License generation
│   │       └── telemetry.rs         # Usage analytics
│   │
│   ├── registry/                    # Container image mirror
│   │   └── docker-compose.yaml      # Registry setup
│   │
│   └── releases/                    # Release artifacts
│       ├── 1.0.0/
│       │   ├── manifest.json
│       │   └── images.tar
│       └── latest -> 1.0.0
│
├── cto-catalog/                     # Separate GitHub repo (5dlabs/cto-catalog)
│   ├── cli-catalog.json             # Supported CLIs
│   ├── model-catalog.json           # Supported models/providers
│   ├── agent-templates/             # Pre-built agent templates
│   │   ├── rex.json
│   │   ├── blaze.json
│   │   └── cypher.json
│   └── README.md                    # Contribution guide
│
└── infra/                           # Existing infrastructure
    ├── charts/                      # Helm charts
    ├── gitops/                      # GitOps config
    └── talos/                       # Talos reference configs
```

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Physical access to server | Disk encryption (future), UEFI Secure Boot |
| Network attacks | Network policies, TLS everywhere, firewall defaults |
| License bypass | Cryptographic signatures, feature flags |
| Supply chain attacks | Signed images, reproducible builds |
| Credential theft | Vault for secrets, short-lived tokens |
| Update tampering | Signed updates, integrity verification |

### Security Defaults

1. **TLS Everywhere**: All internal and external communication encrypted
2. **Network Policies**: Default-deny with explicit allows
3. **RBAC**: Minimal permissions, namespace isolation
4. **Secrets**: Vault-managed, never in ConfigMaps
5. **Image Policy**: Only pull from trusted registries
6. **Audit Logging**: All API calls logged
7. **Automatic Updates**: Security patches applied automatically

## Future Considerations

### Tier 2: Multi-Node Architecture (v1.1)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Multi-Node Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Control Plane (3 nodes for HA)                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  CP Node 1  │  │  CP Node 2  │  │  CP Node 3  │              │
│  │  - etcd     │  │  - etcd     │  │  - etcd     │              │
│  │  - API      │  │  - API      │  │  - API      │              │
│  │  - ctrl-mgr │  │  - ctrl-mgr │  │  - ctrl-mgr │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  Worker Nodes (N nodes)                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Worker 1   │  │  Worker 2   │  │  Worker N   │              │
│  │  - kubelet  │  │  - kubelet  │  │  - kubelet  │              │
│  │  - workloads│  │  - workloads│  │  - workloads│              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  Node Discovery: mDNS / DHCP-based registration                 │
│  Storage: Longhorn replicated across nodes                      │
│  Networking: Cilium with cross-node routing                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Upgrade Path from Single-Node:**

```
┌─────────────────────────────────────────────────────────────────┐
│               Single to Multi-Node Upgrade Path                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Starting State: Single-Node Appliance                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Node 1 (control-plane + worker)                         │    │
│  │  - etcd (single instance)                                │    │
│  │  - All platform workloads                                │    │
│  │  - Local storage (Longhorn single-replica)               │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Step 1: Enable Enterprise Mode (license upgrade)               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Dashboard shows "Add Node" option                       │    │
│  │  Node enrollment token generated                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Step 2: Boot new nodes from enrollment ISO                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  New Node:                                               │    │
│  │  1. Boots enrollment ISO                                 │    │
│  │  2. Discovers existing cluster via mDNS                  │    │
│  │  3. Presents enrollment token                            │    │
│  │  4. Receives Talos config from existing node             │    │
│  │  5. Installs and joins cluster                           │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Step 3: Automatic cluster reconfiguration                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  If 3 nodes: etcd becomes HA (automatic)                 │    │
│  │  Longhorn replicates data to new nodes                   │    │
│  │  Workloads redistributed for balance                     │    │
│  │  Control plane roles assigned                            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Tier 3: Platform-Only Architecture (v1.1+)

```
┌─────────────────────────────────────────────────────────────────┐
│                 Platform-Only (BYOK) Architecture                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Customer's Existing Kubernetes Cluster                          │
│  (EKS, GKE, AKS, k3s, RKE2, kubeadm, etc.)                      │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Customer Managed:                                       │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │    │
│  │  │ K8s Control │  │ Networking  │  │ Storage         │  │    │
│  │  │ Plane       │  │ (CNI)       │  │ (CSI)           │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  CTO Platform (Helm Install)                        ││    │
│  │  │                                                     ││    │
│  │  │  Required:                                          ││    │
│  │  │  - ArgoCD (if not present)                          ││    │
│  │  │  - Vault (or external secrets operator)             ││    │
│  │  │  - CNPG (PostgreSQL operator)                       ││    │
│  │  │  - CTO Agent Controller                             ││    │
│  │  │  - CTO MCP Server                                   ││    │
│  │  │  - CTO Agents                                       ││    │
│  │  │                                                     ││    │
│  │  │  Optional:                                          ││    │
│  │  │  - MinIO (or use existing S3)                       ││    │
│  │  │  - Observability (or use existing)                  ││    │
│  │  │  - Cert-Manager (or use existing)                   ││    │
│  │  │  - Ingress (or use existing)                        ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Deployment Options:                                             │
│  1. Helm: helm install cto 5dlabs/cto-platform                  │
│  2. GitOps: Point ArgoCD at our chart repo                      │
│  3. Operator: CTO Operator manages full lifecycle               │
│                                                                  │
│  Prerequisites Check:                                            │
│  - Kubernetes 1.28+                                              │
│  - Default StorageClass                                          │
│  - Ingress controller                                            │
│  - 16GB+ allocatable memory                                      │
│  - Network policies support (optional but recommended)           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Platform-Only Installation Flow:**

```bash
# Option 1: Helm install
helm repo add 5dlabs https://charts.5dlabs.io
helm install cto 5dlabs/cto-platform \
  --namespace cto-system \
  --create-namespace \
  --values my-values.yaml

# Option 2: GitOps (ArgoCD)
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: cto-platform
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://charts.5dlabs.io
    chart: cto-platform
    targetRevision: 1.0.0
    helm:
      values: |
        license:
          key: "your-license-key"
        ingress:
          host: cto.example.com
  destination:
    server: https://kubernetes.default.svc
    namespace: cto-system
EOF
```

### GPU Support (v1.2)

- NVIDIA GPU Operator integration
- Automatic driver installation via Talos extensions
- GPU scheduling and resource management
- CUDA workload support

### Hardware-Level Monitoring (v1.2+)

Deep integration with physical hardware for proactive issue detection and alerting.

```
┌─────────────────────────────────────────────────────────────────┐
│                 Hardware Monitoring Architecture                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Physical Hardware Layer                                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────────┐  │    │
│  │  │  CPU    │ │  RAM    │ │  Disks  │ │  Network      │  │    │
│  │  │ Sensors │ │ ECC     │ │ SMART   │ │  Interfaces   │  │    │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └───────┬───────┘  │    │
│  │       │           │           │              │           │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────────┐  │    │
│  │  │  Fans   │ │  PSU    │ │  IPMI/  │ │  Temperature  │  │    │
│  │  │  RPM    │ │ Status  │ │  iDRAC  │ │  Sensors      │  │    │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └───────┬───────┘  │    │
│  │       │           │           │              │           │    │
│  └───────┼───────────┼───────────┼──────────────┼───────────┘    │
│          │           │           │              │                │
│          └───────────┴───────────┴──────────────┘                │
│                              │                                   │
│                              ▼                                   │
│  Talos Hardware Integration                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Talos provides:                                         │    │
│  │  • /sys/class/hwmon/* sensors                           │    │
│  │  • smartctl for disk SMART data                         │    │
│  │  • IPMI tools (if hardware supports)                    │    │
│  │  • Network interface statistics                         │    │
│  │  • CPU thermal throttling detection                     │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Hardware Monitor Agent                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Collectors:                                             │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌───────────────────┐  │    │
│  │  │ SMART       │ │ Sensors     │ │ IPMI/BMC          │  │    │
│  │  │ Collector   │ │ Collector   │ │ Collector         │  │    │
│  │  │             │ │             │ │                   │  │    │
│  │  │ • Disk      │ │ • CPU temp  │ │ • Power supply    │  │    │
│  │  │   health    │ │ • Fan RPM   │ │ • Chassis temp    │  │    │
│  │  │ • Bad       │ │ • Voltage   │ │ • Hardware events │  │    │
│  │  │   sectors   │ │ • Power     │ │ • SEL logs        │  │    │
│  │  │ • Wear      │ │             │ │                   │  │    │
│  │  │   level     │ │             │ │                   │  │    │
│  │  └─────────────┘ └─────────────┘ └───────────────────┘  │    │
│  │                                                          │    │
│  │  Analysis Engine:                                        │    │
│  │  • Trend analysis (disk wear prediction)                │    │
│  │  • Anomaly detection (unusual temperatures)             │    │
│  │  • Failure prediction (SMART pre-fail attributes)       │    │
│  │  • Correlation (multiple symptoms = likely cause)       │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Alert & Response System                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Alert Levels:                                           │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ 🟢 INFO     │ Normal operation, logged only         ││    │
│  │  │ 🟡 WARNING  │ Attention needed soon                 ││    │
│  │  │ 🟠 DEGRADED │ Redundancy lost, action needed        ││    │
│  │  │ 🔴 CRITICAL │ Immediate action required             ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  User Response Options:                                  │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │                                                     ││    │
│  │  │  ⚠️ DISK WARNING: /dev/nvme0n1                      ││    │
│  │  │                                                     ││    │
│  │  │  SMART predicts disk failure in ~30 days            ││    │
│  │  │  Current wear level: 87%                            ││    │
│  │  │  Reallocated sectors: 12 (increasing)               ││    │
│  │  │                                                     ││    │
│  │  │  Recommended: Replace disk within 2 weeks           ││    │
│  │  │                                                     ││    │
│  │  │  [View Details] [Order Replacement] [Contact Support]│    │
│  │  │                                                     ││    │
│  │  │  ☐ I'll handle this myself                          ││    │
│  │  │  ☐ Request support assistance ($X)                  ││    │
│  │  │  ☐ Remind me in 7 days                              ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              │ (if opted-in)                     │
│                              ▼                                   │
│  Phone-Home Telemetry (Privacy-Respecting)                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Privacy Settings (User Control):                        │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │                                                     ││    │
│  │  │  Telemetry Preferences                              ││    │
│  │  │                                                     ││    │
│  │  │  ● Full telemetry (recommended)                     ││    │
│  │  │    Hardware health, platform metrics, alerts        ││    │
│  │  │    Enables proactive support                        ││    │
│  │  │                                                     ││    │
│  │  │  ○ Limited telemetry                                ││    │
│  │  │    Critical alerts only, no metrics                 ││    │
│  │  │                                                     ││    │
│  │  │  ○ No telemetry (air-gapped mode)                   ││    │
│  │  │    All data stays local, no phone-home             ││    │
│  │  │    Manual updates required                          ││    │
│  │  │                                                     ││    │
│  │  │  [Save Preferences]                                 ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  What We Receive (Full Telemetry):                       │    │
│  │  ✓ Hardware health scores                               │    │
│  │  ✓ Disk SMART summaries (not contents)                  │    │
│  │  ✓ Temperature trends                                    │    │
│  │  ✓ Component failure predictions                        │    │
│  │  ✓ Platform version and health                          │    │
│  │                                                          │    │
│  │  What We NEVER Receive:                                  │    │
│  │  ✗ File contents or user data                           │    │
│  │  ✗ Application code or configs                          │    │
│  │  ✗ Credentials or secrets                               │    │
│  │  ✗ Network traffic or logs                              │    │
│  │  ✗ Personal information                                 │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  5D Labs Support Backend                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Proactive Support Actions:                              │    │
│  │  • Alert support team to critical issues                │    │
│  │  • Predict failures before customer notices             │    │
│  │  • Reach out proactively: "We noticed your disk..."     │    │
│  │  • Pre-ship replacement parts for enterprise customers  │    │
│  │  • Aggregate data improves predictions for everyone     │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Hardware Metrics Collected:**

| Component | Metric | Alert Threshold |
|-----------|--------|-----------------|
| **Disk (SMART)** | Reallocated sectors | > 0 new in 24h |
| | Wear level (SSD) | < 20% remaining |
| | Pending sectors | > 0 |
| | Temperature | > 60°C sustained |
| | Read/write errors | > 0 uncorrectable |
| **CPU** | Temperature | > 85°C |
| | Throttling events | Any occurrence |
| | MCE (Machine Check) | Any occurrence |
| **Memory** | ECC errors | > 0 correctable/day |
| | Uncorrectable errors | Any occurrence |
| **PSU** | Voltage rails | Outside ±5% |
| | Fan failure | RPM = 0 |
| **Chassis** | Ambient temperature | > 35°C |
| | Fan RPM | Below threshold |
| | Intrusion detection | Door open event |

**Proactive Support Workflow:**

```
Hardware Issue Detected
         │
         ▼
    ┌─────────┐
    │ Alert   │──────────────────────────────────────┐
    │ Level?  │                                      │
    └────┬────┘                                      │
         │                                           │
    ┌────┴────┬────────────┬────────────┐           │
    ▼         ▼            ▼            ▼           │
  INFO    WARNING      DEGRADED     CRITICAL        │
    │         │            │            │           │
    │         │            │            │           │
    ▼         ▼            ▼            ▼           │
  Log      Dashboard    Dashboard    Dashboard      │
  only     warning +    warning +    + Email +      │
           email        email        SMS/Phone      │
                           │            │           │
                           │            │           │
                           ▼            ▼           │
                    ┌─────────────────────┐         │
                    │ Phone-Home         │◄────────┘
                    │ (if opted-in)      │ (all levels
                    └─────────┬──────────┘  if full
                              │             telemetry)
                              ▼
                    ┌─────────────────────┐
                    │ 5D Labs receives:   │
                    │ • Alert type        │
                    │ • Hardware summary  │
                    │ • Customer tier     │
                    └─────────┬───────────┘
                              │
                    ┌─────────┴───────────┐
                    │                     │
                    ▼                     ▼
              Enterprise            Standard
              Customer              Customer
                    │                     │
                    ▼                     ▼
              Proactive           Available if
              outreach +          they request
              ship parts          support
```

### Agent Builder (v1.2+)

A visual interface for creating custom AI agents with specific capabilities, 
prompts, and tool access. Can be offered as a premium subscription feature.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Agent Builder Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Agent Builder UI                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Create New Agent                                        │    │
│  │  ════════════════                                        │    │
│  │                                                          │    │
│  │  Basic Info                                              │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ Name:        [Security Auditor              ]      ││    │
│  │  │ Description: [Reviews code for security issues...] ││    │
│  │  │ Icon:        [🔒 ▼]                                 ││    │
│  │  │ Category:    [Security ▼]                          ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  Base Template                                           │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ ○ Start from scratch                               ││    │
│  │  │ ● Clone existing agent                             ││    │
│  │  │   └─ [Rex (Backend) ▼]                             ││    │
│  │  │ ○ Import from file                                 ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  System Prompt                                           │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ You are a security-focused code reviewer.          ││    │
│  │  │                                                     ││    │
│  │  │ Your responsibilities:                             ││    │
│  │  │ - Identify OWASP Top 10 vulnerabilities            ││    │
│  │  │ - Check for hardcoded secrets                      ││    │
│  │  │ - Review authentication/authorization logic        ││    │
│  │  │ - Flag SQL injection risks                         ││    │
│  │  │ - Check dependency vulnerabilities                 ││    │
│  │  │                                                     ││    │
│  │  │ Always explain WHY something is a security risk.   ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │  [✨ AI Enhance Prompt]                                  │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Tool Access                                             │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │                                                     ││    │
│  │  │  Git Provider                                       ││    │
│  │  │  ☑ Read files          ☑ Create branches          ││    │
│  │  │  ☑ Read PRs            ☑ Create PRs               ││    │
│  │  │  ☑ Comment on PRs      ☐ Merge PRs                ││    │
│  │  │  ☑ Read issues         ☐ Create issues            ││    │
│  │  │                                                     ││    │
│  │  │  MCP Tools                                          ││    │
│  │  │  ☑ File system (read)  ☐ File system (write)      ││    │
│  │  │  ☑ Code search         ☑ Grep                     ││    │
│  │  │  ☐ Shell execution     ☐ Database access          ││    │
│  │  │                                                     ││    │
│  │  │  External                                           ││    │
│  │  │  ☑ Web search          ☑ Documentation lookup     ││    │
│  │  │  ☐ Slack notifications ☐ Email                    ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  Triggers                                                │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  When should this agent run?                       ││    │
│  │  │                                                     ││    │
│  │  │  ☑ On PR opened                                    ││    │
│  │  │  ☑ On PR updated (new commits)                     ││    │
│  │  │  ☐ On commit to main                               ││    │
│  │  │  ☐ Scheduled (cron)  [________]                    ││    │
│  │  │  ☑ Manual trigger                                  ││    │
│  │  │  ☐ On issue created with label: [security]         ││    │
│  │  │                                                     ││    │
│  │  │  File filters (only trigger for these paths):      ││    │
│  │  │  [src/**, lib/**, !*.test.ts, !*.md            ]   ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  AI Configuration                                        │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  CLI:       [Cursor CLI ▼]                         ││    │
│  │  │  Model:     [Claude Sonnet 4 ▼]                    ││    │
│  │  │  Max tokens: [8192        ]                        ││    │
│  │  │  Temperature: [0.3] ──●────────                    ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  [Test Agent]  [Save as Draft]  [Deploy Agent]          │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Agent Definition Format:**

```yaml
# security-auditor.agent.yaml
apiVersion: cto.5dlabs.io/v1
kind: Agent
metadata:
  name: security-auditor
  labels:
    category: security
spec:
  displayName: "Security Auditor"
  description: "Reviews code for security vulnerabilities"
  icon: "🔒"
  
  # Base configuration
  base:
    template: rex  # Optional: inherit from existing agent
    
  # System prompt
  prompt: |
    You are a security-focused code reviewer.
    
    Your responsibilities:
    - Identify OWASP Top 10 vulnerabilities
    - Check for hardcoded secrets
    - Review authentication/authorization logic
    ...
    
  # Tool permissions
  tools:
    git:
      read: true
      write: false
      createPR: true
      mergePR: false
    mcp:
      filesystem:
        read: true
        write: false
      shell: false
      database: false
    external:
      webSearch: true
      documentation: true
      
  # Triggers
  triggers:
    - type: pullRequest
      events: [opened, synchronize]
      pathFilters:
        include: ["src/**", "lib/**"]
        exclude: ["*.test.ts", "*.md"]
    - type: manual
    
  # AI configuration  
  ai:
    cli: cursor
    model: claude-sonnet-4
    maxTokens: 8192
    temperature: 0.3
```

**Agent Marketplace (Future):**

```
┌─────────────────────────────────────────────────────────────────┐
│  Agent Marketplace                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [Search agents...]                    [My Agents] [Browse All] │
│                                                                  │
│  Featured                                                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │ 🔒          │ │ 📝          │ │ 🧪          │               │
│  │ Security    │ │ Doc Writer  │ │ Test Gen    │               │
│  │ Auditor     │ │             │ │             │               │
│  │             │ │ Auto-       │ │ Generates   │               │
│  │ OWASP &     │ │ generates   │ │ unit tests  │               │
│  │ CVE checks  │ │ docs from   │ │ for your    │               │
│  │             │ │ code        │ │ code        │               │
│  │ ⭐⭐⭐⭐⭐    │ │ ⭐⭐⭐⭐      │ │ ⭐⭐⭐⭐      │               │
│  │ By 5D Labs  │ │ By 5D Labs  │ │ Community   │               │
│  │ [Install]   │ │ [Install]   │ │ [Install]   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  Categories: [Security] [Documentation] [Testing] [DevOps]      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Subscription Tiers for Agent Builder:**

| Feature | Starter | Team | Business | Enterprise |
|---------|---------|------|----------|------------|
| Pre-built agents | 5 | 10 | Unlimited | Unlimited |
| Custom agents | 0 | 3 | 10 | Unlimited |
| Agent Builder UI | ❌ | ✅ | ✅ | ✅ |
| Import/Export | ❌ | ✅ | ✅ | ✅ |
| Marketplace access | View | Install | Install + Publish | Full |
| Private agents | ❌ | ❌ | ✅ | ✅ |
| Agent analytics | ❌ | Basic | Full | Full |

### Supported Languages & Platforms

CTO Platform can build applications across virtually any language, framework, 
and platform through specialized agents and configurable toolchains.

```
┌─────────────────────────────────────────────────────────────────┐
│              Supported Languages & Platforms Matrix              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  WEB APPLICATIONS                                                │
│  ════════════════                                                │
│                                                                  │
│  Frontend:                                                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │   Next.js   │ │    React    │ │    Vue.js   │               │
│  │   (MVP ✓)   │ │             │ │   (v1.2+)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │   Svelte    │ │   Angular   │ │    Astro    │               │
│  │   (v1.2+)   │ │   (v1.3+)   │ │   (v1.2+)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  Styling:                                                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │ Tailwind CSS│ │  shadcn/ui  │ │    CSS      │               │
│  │   (MVP ✓)   │ │   (MVP ✓)   │ │   Modules   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  MOBILE APPLICATIONS                                             │
│  ═══════════════════                                             │
│                                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │React Native │ │    Expo     │ │   Flutter   │               │
│  │   (v1.2+)   │ │   (v1.2+)   │ │   (v1.3+)   │               │
│  │             │ │  Preferred  │ │             │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  DESKTOP APPLICATIONS                                            │
│  ════════════════════                                            │
│                                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │  Electron   │ │    Tauri    │ │   Native    │               │
│  │   (v1.2+)   │ │   (v1.3+)   │ │   (v1.4+)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  BACKEND / API                                                   │
│  ═════════════                                                   │
│                                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │     Go      │ │    Rust     │ │   Node.js   │               │
│  │   (MVP ✓)   │ │   (MVP ✓)   │ │   (v1.1+)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │   Python    │ │    Java     │ │     .NET    │               │
│  │   (v1.2+)   │ │   (v1.3+)   │ │   (v1.3+)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  INFRASTRUCTURE / DEVOPS                                         │
│  ═══════════════════════                                         │
│                                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │ Kubernetes  │ │  Terraform  │ │    Helm     │               │
│  │   (MVP ✓)   │ │   (MVP ✓)   │ │   (MVP ✓)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│  ┌─────────────┐ ┌─────────────┐                               │
│  │   Docker    │ │  Ansible    │                               │
│  │   (MVP ✓)   │ │   (v1.2+)   │                               │
│  └─────────────┘ └─────────────┘                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Specialized Agents by Stack:**

```
┌─────────────────────────────────────────────────────────────────┐
│                   Agent Specializations                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Pre-Built Agents (v1.0 MVP)                                    │
│  ───────────────────────────                                    │
│                                                                  │
│  🤖 Rex         Backend specialist (Go, Rust, Node.js)          │
│  🔥 Blaze       Frontend specialist (React, Next.js, Tailwind)  │
│  🔐 Cypher      Security & auth specialist                      │
│  📊 DataMind    Database & data pipelines                       │
│  🛡️ Guardian    Platform health & self-healing                  │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  Additional Agents (v1.2+)                                      │
│  ─────────────────────────                                      │
│                                                                  │
│  📱 Mobile      React Native / Expo specialist                  │
│  🖥️ Desktop     Electron / Tauri specialist                     │
│  🐍 Pythonista  Python backend & ML specialist                  │
│  ☕ JavaForge   Java / Spring Boot specialist                   │
│  🦀 Rustacean   Rust systems specialist                         │
│  🌐 CloudOps    Infrastructure & Terraform specialist           │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  Custom Agents (Agent Builder)                                  │
│  ────────────────────────────                                   │
│                                                                  │
│  Users can create specialized agents for:                       │
│  • Specific frameworks (Laravel, Django, Rails)                 │
│  • Company-specific conventions                                 │
│  • Domain expertise (fintech, healthcare, gaming)               │
│  • Legacy codebases                                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Language Support Roadmap:**

| Version | Frontend | Mobile | Desktop | Backend | Infra |
|---------|----------|--------|---------|---------|-------|
| **v1.0 (MVP)** | Next.js, React | - | - | Go, Rust | K8s, Docker |
| **v1.1** | + Tailwind deep | - | - | + Node.js | + Terraform |
| **v1.2** | + Vue, Svelte, Astro | React Native, Expo | Electron | + Python | + Ansible |
| **v1.3** | + Angular | + Flutter | + Tauri | + Java, .NET | + Pulumi |
| **v1.4+** | Community | + Native iOS/Android | + Native | Community | Community |

### Tool Management System

A centralized system for managing AI tools, MCP servers, and CLI integrations
that agents can use.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Tool Management Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Tool Registry                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  Built-in Tools                              [Core] ││    │
│  │  │  ─────────────                                      ││    │
│  │  │  ✅ File System (read, write, search)              ││    │
│  │  │  ✅ Git Operations (clone, commit, PR)             ││    │
│  │  │  ✅ Terminal (shell commands)                      ││    │
│  │  │  ✅ Web Search (Firecrawl)                         ││    │
│  │  │  ✅ Browser Automation                             ││    │
│  │  │  ✅ Code Search (semantic, grep)                   ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  Platform Tools                          [Included] ││    │
│  │  │  ──────────────                                     ││    │
│  │  │  ✅ Kubernetes (kubectl, helm)                     ││    │
│  │  │  ✅ Database (psql, migrations)                    ││    │
│  │  │  ✅ Secrets (vault CLI)                            ││    │
│  │  │  ✅ Observability (promql, logql)                  ││    │
│  │  │  ✅ ArgoCD (sync, rollback)                        ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  AI/LLM Integrations                     [Add-on]  ││    │
│  │  │  ───────────────────                                ││    │
│  │  │  ○ Cursor CLI                                      ││    │
│  │  │  ○ Claude API (direct)                             ││    │
│  │  │  ○ OpenAI API                                      ││    │
│  │  │  ○ Anthropic API                                   ││    │
│  │  │  ○ Ollama (local models)                           ││    │
│  │  │  ○ OpenRouter                                      ││    │
│  │  │  [Configure API Keys]                              ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  External Services                       [Add-on]  ││    │
│  │  │  ─────────────────                                  ││    │
│  │  │  ○ GitHub API                                      ││    │
│  │  │  ○ GitLab API                                      ││    │
│  │  │  ○ Slack                                           ││    │
│  │  │  ○ Discord                                         ││    │
│  │  │  ○ Email (SMTP)                                    ││    │
│  │  │  [Add Integration]                                 ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  Project Management                      [Add-on]  ││    │
│  │  │  ──────────────────                                 ││    │
│  │  │  ✅ GitHub Projects  (Default)                     ││    │
│  │  │  ○ Linear           Modern, fast                   ││    │
│  │  │  ○ Jira             Enterprise standard            ││    │
│  │  │  ○ Asana            Team workflows                 ││    │
│  │  │  ○ Trello           Kanban boards                  ││    │
│  │  │  ○ Notion           All-in-one workspace           ││    │
│  │  │  ○ Monday.com       Work management                ││    │
│  │  │  ○ ClickUp          Feature-rich                   ││    │
│  │  │  ○ Shortcut         Developer-focused              ││    │
│  │  │  [Configure Project Management]                    ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Tool Management UI:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Settings > Tools & Integrations                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  AI Models                                                       │
│  ─────────                                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Default Model: [Claude Sonnet 4 ▼]                      │    │
│  │                                                          │    │
│  │ Configured Providers:                                    │    │
│  │ ┌────────────────────────────────────────────────────┐  │    │
│  │ │ ✅ Anthropic        API Key: ****...7x2F  [Edit]   │  │    │
│  │ │ ✅ Cursor CLI       Authenticated        [Reauth]  │  │    │
│  │ │ ○  OpenAI          Not configured       [Setup]   │  │    │
│  │ │ ○  Ollama (Local)  Not running          [Setup]   │  │    │
│  │ └────────────────────────────────────────────────────┘  │    │
│  │                                                          │    │
│  │ [+ Add Provider]                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  MCP Servers                                                     │
│  ───────────                                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Active MCP Servers:                                      │    │
│  │ ┌────────────────────────────────────────────────────┐  │    │
│  │ │ ✅ filesystem      Built-in              Running   │  │    │
│  │ │ ✅ git             Built-in              Running   │  │    │
│  │ │ ✅ kubernetes      Platform              Running   │  │    │
│  │ │ ✅ firecrawl       Web search            Running   │  │    │
│  │ │ ✅ browser         Automation            Running   │  │    │
│  │ │ ○  custom-tools    User-defined          Stopped   │  │    │
│  │ └────────────────────────────────────────────────────┘  │    │
│  │                                                          │    │
│  │ [+ Add MCP Server]                                       │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  [Advanced] Custom Tools (for power users)                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Advanced Tool Configuration (Power Users):**

```
┌─────────────────────────────────────────────────────────────────┐
│  Advanced > Custom Tools                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ⚠️ Advanced users only. Misconfiguration may affect agents.    │
│                                                                  │
│  Custom MCP Servers                                              │
│  ──────────────────                                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Add Custom MCP Server                                    │    │
│  │                                                          │    │
│  │ Name:        [my-custom-tools                    ]      │    │
│  │ Type:        ○ stdio  ● sse  ○ websocket               │    │
│  │ Command:     [npx -y @myorg/mcp-server           ]      │    │
│  │ Environment:                                             │    │
│  │   ┌────────────────────────────────────────────────┐    │    │
│  │   │ API_KEY=sk-xxx                                 │    │    │
│  │   │ BASE_URL=https://api.example.com              │    │    │
│  │   └────────────────────────────────────────────────┘    │    │
│  │                                                          │    │
│  │ Capabilities:                                            │    │
│  │ ☑ Allow in agents  ☑ Allow in workflows  ☐ Admin only  │    │
│  │                                                          │    │
│  │              [Test Connection]  [Save]                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Custom CLI Tools                                                │
│  ────────────────                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Register CLI for agent use:                              │    │
│  │                                                          │    │
│  │ ┌────────────────────────────────────────────────────┐  │    │
│  │ │ Name        Command              Path     Enabled  │  │    │
│  │ │ ──────────  ───────────────────  ───────  ───────  │  │    │
│  │ │ aws         aws                  system   ☑        │  │    │
│  │ │ gcloud      gcloud               system   ☐        │  │    │
│  │ │ az          az                   system   ☐        │  │    │
│  │ │ pulumi      pulumi               custom   ☑        │  │    │
│  │ │ custom-cli  /opt/bin/my-cli      custom   ☑        │  │    │
│  │ └────────────────────────────────────────────────────┘  │    │
│  │                                                          │    │
│  │ [+ Add CLI Tool]                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Import/Export                                                   │
│  ─────────────                                                   │
│  [Export Tool Config]  [Import Tool Config]                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Tool Configuration YAML:**

```yaml
# tools-config.yaml - Exportable tool configuration
apiVersion: cto.5dlabs.io/v1
kind: ToolConfiguration

aiProviders:
  default: anthropic
  providers:
    - name: anthropic
      enabled: true
      models:
        - claude-sonnet-4
        - claude-opus-4
      apiKey: ${ANTHROPIC_API_KEY}
    - name: cursor
      enabled: true
      authMethod: oauth
    - name: ollama
      enabled: false
      endpoint: http://localhost:11434

mcpServers:
  builtin:
    - filesystem
    - git
    - kubernetes
    - browser
  external:
    - name: firecrawl
      command: npx -y firecrawl-mcp
      env:
        FIRECRAWL_API_KEY: ${FIRECRAWL_KEY}
    - name: custom-tools
      command: /opt/mcp/custom-server
      capabilities:
        - agents
        - workflows

cliTools:
  - name: aws
    command: aws
    enabled: true
  - name: pulumi
    command: pulumi
    path: /usr/local/bin/pulumi
    enabled: true
```

### Project Management Integrations

CTO Platform integrates with popular project management tools through a unified 
abstraction layer. This allows agents to read tasks, update status, create issues,
and sync progress regardless of the underlying PM tool.

```
┌─────────────────────────────────────────────────────────────────┐
│              Project Management Integration Architecture         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                    CTO Platform Agents                           │
│                           │                                      │
│                           ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │            Unified Project Management API                │    │
│  │                                                          │    │
│  │  Common Operations:                                      │    │
│  │  • getTask(id)           • createTask(data)             │    │
│  │  • listTasks(filters)    • updateTask(id, data)         │    │
│  │  • getProject(id)        • addComment(taskId, text)     │    │
│  │  • listProjects()        • attachFile(taskId, file)     │    │
│  │  • getSprint/Iteration() • transitionStatus(id, status) │    │
│  │                                                          │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │                                   │
│              ┌───────────────┼───────────────┐                  │
│              │               │               │                  │
│              ▼               ▼               ▼                  │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐         │
│  │    Provider   │ │    Provider   │ │    Provider   │         │
│  │    Adapters   │ │    Adapters   │ │    Adapters   │         │
│  └───────┬───────┘ └───────┬───────┘ └───────┬───────┘         │
│          │                 │                 │                  │
│          ▼                 ▼                 ▼                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  GitHub   Linear   Jira   Asana   Trello   Notion  ...  │    │
│  │  Projects                                                │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Supported Project Management Tools:**

| Tool | Status | Best For | Key Features |
|------|--------|----------|--------------|
| **GitHub Projects** | MVP ✓ | Dev teams using GitHub | Native integration, issues sync |
| **Linear** | v1.1 | Modern startups | Fast API, cycles, triage |
| **Jira** | v1.1 | Enterprise | Workflows, JQL, Confluence |
| **Asana** | v1.2 | Cross-functional teams | Portfolios, workload |
| **Trello** | v1.2 | Simple kanban | Power-Ups, Butler |
| **Notion** | v1.2 | All-in-one workspace | Databases, docs |
| **Monday.com** | v1.3 | Work management | Automations, dashboards |
| **ClickUp** | v1.3 | Feature-rich | Docs, whiteboards |
| **Shortcut** | v1.3 | Developer teams | Stories, epics, iterations |
| **Azure Boards** | v1.3 | Microsoft shops | Azure DevOps integration |

**Project Management UI:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Settings > Project Management                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Active Integration                                              │
│  ──────────────────                                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ ┌──────────────────────────────────────────────────────┐│    │
│  │ │ 🐙 GitHub Projects                        [Active]  ││    │
│  │ │    Organization: acme-corp                          ││    │
│  │ │    Projects synced: 3                               ││    │
│  │ │    Last sync: 2 minutes ago                         ││    │
│  │ │                                                      ││    │
│  │ │    [Configure] [Disconnect] [Sync Now]              ││    │
│  │ └──────────────────────────────────────────────────────┘│    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Add Another Integration                                         │
│  ───────────────────────                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │    │
│  │  │ Linear  │ │  Jira   │ │  Asana  │ │ Trello  │       │    │
│  │  │         │ │         │ │         │ │         │       │    │
│  │  │[Connect]│ │[Connect]│ │[Connect]│ │[Connect]│       │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │    │
│  │                                                          │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │    │
│  │  │ Notion  │ │ Monday  │ │ ClickUp │ │Shortcut │       │    │
│  │  │         │ │         │ │         │ │         │       │    │
│  │  │[Connect]│ │[Connect]│ │[Connect]│ │[Connect]│       │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Sync Settings                                                   │
│  ─────────────                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Auto-sync interval:    [Every 5 minutes ▼]              │    │
│  │ Sync direction:        [● Bidirectional ○ Read-only]    │    │
│  │ Create tasks in:       [GitHub Projects ▼]              │    │
│  │ Status mapping:        [Configure Mappings]             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Status Mapping Configuration:**

Different PM tools have different status workflows. CTO Platform maps them:

```
┌─────────────────────────────────────────────────────────────────┐
│  Status Mapping: GitHub ↔ Jira                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  CTO Platform Status    GitHub Status       Jira Status         │
│  ───────────────────    ─────────────       ───────────         │
│  📥 Backlog         →   No Status       →   To Do               │
│  📋 Ready           →   Ready           →   Ready for Dev       │
│  🔄 In Progress     →   In Progress     →   In Progress         │
│  👀 Review          →   In Review       →   Code Review         │
│  ✅ Done            →   Done            →   Done                │
│  🚫 Blocked         →   Blocked         →   Blocked             │
│                                                                  │
│  [+ Add Custom Mapping]                                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Multi-Tool Sync (Enterprise):**

For enterprises using multiple tools across teams:

```
┌─────────────────────────────────────────────────────────────────┐
│             Multi-Tool Project Sync (Enterprise)                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Engineering               Product                 Leadership    │
│  ┌─────────────┐          ┌─────────────┐        ┌───────────┐ │
│  │   GitHub    │          │    Jira     │        │   Asana   │ │
│  │  Projects   │◄────────►│   Epics     │◄──────►│ Portfolio │ │
│  └─────────────┘          └─────────────┘        └───────────┘ │
│        │                         │                      │       │
│        │                         │                      │       │
│        └─────────────────────────┼──────────────────────┘       │
│                                  │                               │
│                                  ▼                               │
│                    ┌─────────────────────────┐                  │
│                    │   CTO Platform Hub      │                  │
│                    │   (Single source of     │                  │
│                    │    truth for agents)    │                  │
│                    └─────────────────────────┘                  │
│                                                                  │
│  Benefits:                                                       │
│  • Each team uses their preferred tool                          │
│  • Agents see unified view                                       │
│  • Auto-sync keeps everything aligned                           │
│  • No manual status updates across tools                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Project Management Configuration:**

```yaml
# project-management.yaml
apiVersion: cto.5dlabs.io/v1
kind: ProjectManagement

primary: github-projects

integrations:
  - name: github-projects
    type: github
    config:
      organization: acme-corp
      defaultProject: "Engineering Board"
    sync:
      enabled: true
      interval: 5m
      direction: bidirectional
      
  - name: linear-sync
    type: linear
    config:
      teamId: ENG
      workspace: acme
    sync:
      enabled: true
      interval: 5m
      direction: read-only  # Linear is source of truth

statusMapping:
  platform:
    - backlog
    - ready
    - in_progress
    - review
    - done
    - blocked
  mappings:
    github:
      backlog: "No Status"
      ready: "Ready"
      in_progress: "In Progress"
      review: "In Review"
      done: "Done"
      blocked: "Blocked"
    linear:
      backlog: "Backlog"
      ready: "Todo"
      in_progress: "In Progress"
      review: "In Review"
      done: "Done"
      blocked: "Blocked"
    jira:
      backlog: "To Do"
      ready: "Ready for Dev"
      in_progress: "In Progress"
      review: "Code Review"
      done: "Done"
      blocked: "Blocked"
```

### Notification & Communication Integrations

CTO Platform provides multi-channel notifications for monitoring agent progress,
receiving alerts, and even submitting new tasks/PRDs through messaging platforms.

```
┌─────────────────────────────────────────────────────────────────┐
│               Notification Channels Architecture                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Ways to Monitor & Interact with CTO Platform:                  │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Web UI    │  │ Mobile App  │  │    Slack    │             │
│  │             │  │   (React    │  │             │             │
│  │  Dashboard  │  │   Native)   │  │   Bot +     │             │
│  │  Full ctrl  │  │   Monitor   │  │   Channel   │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                     │
│         │    ┌───────────┼────────────────┤                     │
│         │    │           │                │                     │
│         ▼    ▼           ▼                ▼                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  CTO Platform API                        │   │
│  │                                                          │   │
│  │  • Real-time events (WebSocket)                         │   │
│  │  • Push notifications                                    │   │
│  │  • Webhook dispatching                                   │   │
│  │  • Chat command processing                              │   │
│  └─────────────────────────────────────────────────────────┘   │
│         │                │                │                     │
│         ▼                ▼                ▼                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Discord   │  │    Email    │  │  Webhooks   │             │
│  │             │  │             │  │             │             │
│  │   Bot +     │  │   Digests   │  │   Custom    │             │
│  │   Server    │  │   Alerts    │  │   Integr.   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Supported Notification Channels:**

| Channel | Status | Capabilities | Best For |
|---------|--------|--------------|----------|
| **Slack** | MVP ✓ | Bot, slash commands, threads | Teams using Slack |
| **Discord** | MVP ✓ | Bot, slash commands, channels | Dev communities |
| **Web UI** | MVP ✓ | Full dashboard, real-time | Primary interface |
| **Mobile App** | v1.1 | Push notifications, monitoring | On-the-go |
| **Email** | v1.1 | Digests, critical alerts | Async updates |
| **MS Teams** | v1.2 | Bot, cards, tabs | Enterprise |
| **Webhooks** | v1.1 | Custom integrations | Power users |

**Slack Integration:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Slack Bot Capabilities                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  📢 NOTIFICATIONS (Receive)                                     │
│  ──────────────────────────                                     │
│  • Task started / completed / failed                            │
│  • PR opened / merged / needs review                            │
│  • Deployment status (staging, production)                      │
│  • Agent questions (needs human input)                          │
│  • System alerts (resource usage, errors)                       │
│                                                                  │
│  💬 COMMANDS (Send)                                             │
│  ──────────────────                                             │
│  /cto status              - Current platform status             │
│  /cto tasks               - List active tasks                   │
│  /cto task <id>           - Get task details                    │
│  /cto new "description"   - Create new task                     │
│  /cto prd                 - Submit a PRD (opens modal)          │
│  /cto deploy <env>        - Trigger deployment                  │
│  /cto logs <service>      - Get recent logs                     │
│  /cto ask "question"      - Ask the platform a question         │
│                                                                  │
│  📝 PRD SUBMISSION (via Slack)                                  │
│  ─────────────────────────────                                  │
│  User can submit PRDs directly from Slack:                      │
│                                                                  │
│  1. Type /cto prd                                               │
│  2. Modal opens with PRD template                               │
│  3. Fill in requirements                                        │
│  4. Submit → Creates task, agents start working                 │
│                                                                  │
│  Or: Paste PRD text in #cto-requests channel                    │
│      Bot parses and creates tasks automatically                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Slack Notification Examples:**

```
┌─────────────────────────────────────────────────────────────────┐
│  #engineering                                                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  🤖 CTO Platform                                    2:34 PM     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ✅ Task Completed                                        │   │
│  │                                                          │   │
│  │ **User Authentication API**                              │   │
│  │ Task #42 completed successfully                          │   │
│  │                                                          │   │
│  │ 📊 Summary:                                              │   │
│  │ • 12 files changed                                       │   │
│  │ • 847 lines added                                        │   │
│  │ • All tests passing                                      │   │
│  │ • PR #156 ready for review                               │   │
│  │                                                          │   │
│  │ [View PR] [View Task] [Deploy to Staging]                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  🤖 CTO Platform                                    2:45 PM     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ❓ Agent Question                                        │   │
│  │                                                          │   │
│  │ Rex needs clarification on Task #43:                     │   │
│  │                                                          │   │
│  │ "Should the payment API support both Stripe and         │   │
│  │ PayPal, or just Stripe for MVP?"                         │   │
│  │                                                          │   │
│  │ [Just Stripe] [Both] [Reply with details...]             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Discord Integration:**

```
┌─────────────────────────────────────────────────────────────────┐
│                   Discord Bot Capabilities                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Similar to Slack, but with Discord-specific features:          │
│                                                                  │
│  📁 CHANNEL STRUCTURE                                           │
│  ────────────────────                                           │
│  #cto-status       - Platform health, deployments               │
│  #cto-tasks        - Task updates, completions                  │
│  #cto-alerts       - Critical alerts (@ mentions)               │
│  #cto-requests     - Submit new tasks/PRDs                      │
│  #cto-logs         - Verbose logging (optional)                 │
│                                                                  │
│  🎮 SLASH COMMANDS                                              │
│  ─────────────────                                              │
│  Same as Slack: /cto status, /cto tasks, /cto prd, etc.        │
│                                                                  │
│  🔔 ROLE MENTIONS                                               │
│  ────────────────                                               │
│  @DevOps   - Infrastructure alerts                              │
│  @Backend  - API/backend task updates                           │
│  @Frontend - UI task updates                                    │
│  @All      - Critical/blocking issues                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Multi-Channel Monitoring:**

```
┌─────────────────────────────────────────────────────────────────┐
│              Unified Monitoring Experience                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Same Information, Multiple Channels:                           │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │               Task #42 Completed                        │    │
│  │               User Auth API                              │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                     │                                           │
│      ┌──────────────┼──────────────┬──────────────┐            │
│      │              │              │              │            │
│      ▼              ▼              ▼              ▼            │
│  ┌────────┐   ┌────────┐   ┌────────┐   ┌────────┐            │
│  │  Web   │   │ Mobile │   │ Slack  │   │Discord │            │
│  │        │   │        │   │        │   │        │            │
│  │ 🔔 Badge│   │📱 Push │   │💬 Msg  │   │💬 Msg  │            │
│  │ + Toast│   │ Notif  │   │        │   │        │            │
│  └────────┘   └────────┘   └────────┘   └────────┘            │
│                                                                  │
│  User Preference:                                                │
│  "I want Slack for task updates,                                │
│   push notifications for critical alerts,                       │
│   email digest for daily summary"                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Notification Settings UI:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Settings > Notifications                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Connected Channels                                              │
│  ──────────────────                                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ ✅ Slack       acme-corp.slack.com    [Configure]       │    │
│  │ ✅ Discord     Acme Dev Server        [Configure]       │    │
│  │ ○  Email       Not configured         [Setup]           │    │
│  │ ○  MS Teams    Not configured         [Setup]           │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Notification Preferences                                        │
│  ────────────────────────                                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Web    Mobile  Slack  Discord  Email │    │
│  │                    ───    ──────  ─────  ───────  ───── │    │
│  │ Task completed     ☑      ☐       ☑      ☐        ☐     │    │
│  │ Task failed        ☑      ☑       ☑      ☑        ☑     │    │
│  │ PR ready           ☑      ☐       ☑      ☐        ☐     │    │
│  │ Deploy success     ☑      ☑       ☑      ☐        ☐     │    │
│  │ Deploy failed      ☑      ☑       ☑      ☑        ☑     │    │
│  │ Agent question     ☑      ☑       ☑      ☑        ☐     │    │
│  │ System alert       ☑      ☑       ☐      ☐        ☑     │    │
│  │ Daily digest       ☐      ☐       ☐      ☐        ☑     │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  PRD Submission Channels                                         │
│  ───────────────────────                                         │
│  Allow PRD/task submission via:                                  │
│  ☑ Web UI                                                        │
│  ☑ Mobile App                                                    │
│  ☑ Slack (/cto prd or #cto-requests channel)                    │
│  ☑ Discord (/cto prd or #cto-requests channel)                  │
│  ☐ Email (parse PRD from email body)                            │
│                                                                  │
│  [Save Preferences]                                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Notification Configuration:**

```yaml
# notifications.yaml
apiVersion: cto.5dlabs.io/v1
kind: NotificationConfig

channels:
  slack:
    enabled: true
    workspace: acme-corp
    botToken: ${SLACK_BOT_TOKEN}
    channels:
      status: "#cto-status"
      tasks: "#cto-tasks"
      alerts: "#cto-alerts"
      requests: "#cto-requests"
    features:
      slashCommands: true
      prdSubmission: true
      interactiveButtons: true

  discord:
    enabled: true
    guildId: "123456789"
    botToken: ${DISCORD_BOT_TOKEN}
    channels:
      status: "cto-status"
      tasks: "cto-tasks"
      alerts: "cto-alerts"
    features:
      slashCommands: true
      roleMentions: true

  email:
    enabled: true
    smtp:
      host: smtp.sendgrid.net
      port: 587
      auth: ${SMTP_AUTH}
    from: "cto@acme.com"
    digest:
      enabled: true
      schedule: "0 9 * * 1-5"  # 9am weekdays

preferences:
  defaults:
    taskCompleted: [web, slack]
    taskFailed: [web, mobile, slack, discord, email]
    prReady: [web, slack]
    deploySuccess: [web, mobile, slack]
    deployFailed: [web, mobile, slack, discord, email]
    agentQuestion: [web, mobile, slack, discord]
    systemAlert: [web, mobile, email]
```

### Cost Analysis Dashboard

A comprehensive dashboard for tracking token usage, model costs, and budget 
management across projects. Token usage is the primary cost driver and the 
main focus of this system.

```
┌─────────────────────────────────────────────────────────────────┐
│                   Cost Analysis Architecture                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                      Agent Requests                              │
│                           │                                      │
│                           ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Token Metering Layer                    │    │
│  │                                                          │    │
│  │  Every LLM call is tracked:                             │    │
│  │  • Input tokens                                          │    │
│  │  • Output tokens                                         │    │
│  │  • Model used                                            │    │
│  │  • Project context                                       │    │
│  │  • Agent/task context                                    │    │
│  │  • Timestamp                                             │    │
│  │                                                          │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   Usage Database                         │    │
│  │                                                          │    │
│  │  token_usage:                                            │    │
│  │  ├── project_id                                          │    │
│  │  ├── task_id                                             │    │
│  │  ├── agent_id                                            │    │
│  │  ├── model                                               │    │
│  │  ├── input_tokens                                        │    │
│  │  ├── output_tokens                                       │    │
│  │  ├── cost_usd                                            │    │
│  │  └── timestamp                                           │    │
│  │                                                          │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │                                   │
│              ┌───────────────┼───────────────┐                  │
│              │               │               │                  │
│              ▼               ▼               ▼                  │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐         │
│  │   Dashboard   │ │    Alerts     │ │   Reports     │         │
│  │   Real-time   │ │   Budget      │ │   Export      │         │
│  │   Analytics   │ │   Warnings    │ │   CSV/PDF     │         │
│  └───────────────┘ └───────────────┘ └───────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Cost Dashboard UI:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Cost Analysis                                         [Export] │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  This Month                                               │   │
│  │  ═══════════                                              │   │
│  │                                                           │   │
│  │  💰 Total Cost        📊 Tokens Used       📈 vs Last Mo │   │
│  │  ┌──────────────┐    ┌──────────────┐    ┌────────────┐  │   │
│  │  │   $847.32    │    │   12.4M      │    │   +23%     │  │   │
│  │  │              │    │   tokens     │    │   ▲        │  │   │
│  │  └──────────────┘    └──────────────┘    └────────────┘  │   │
│  │                                                           │   │
│  │  Budget: $1,000/mo                    ████████░░ 85%     │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Cost by Model                                                   │
│  ─────────────                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Claude Sonnet 4    ████████████████████░░░  $512.40     │   │
│  │  Claude Opus 4      ████████░░░░░░░░░░░░░░░  $284.20     │   │
│  │  Claude Haiku       ██░░░░░░░░░░░░░░░░░░░░░   $42.50     │   │
│  │  GPT-4o             █░░░░░░░░░░░░░░░░░░░░░░    $8.22     │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Cost by Project                                                 │
│  ───────────────                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Project              Tasks    Tokens     Cost    Model  │   │
│  │  ──────────────────   ─────    ──────     ────    ────── │   │
│  │  🔹 acme-webapp         47    5.2M     $412.30   Sonnet  │   │
│  │  🔹 payment-api         23    3.1M     $248.40   Opus    │   │
│  │  🔹 mobile-app          18    2.8M     $142.20   Sonnet  │   │
│  │  🔹 admin-dashboard     12    1.3M      $44.42   Haiku   │   │
│  │                                                           │   │
│  │  [View All Projects]                                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Per-Project Model Selection:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Project Settings > acme-webapp > Model Configuration           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Default Model for This Project                                  │
│  ──────────────────────────────                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Current: Claude Sonnet 4                               │    │
│  │                                                          │    │
│  │  [Claude Sonnet 4 ▼]                                    │    │
│  │                                                          │    │
│  │  ┌────────────────────────────────────────────────────┐ │    │
│  │  │ ● Claude Sonnet 4     $3/$15 per 1M tokens  [Best] │ │    │
│  │  │ ○ Claude Opus 4       $15/$75 per 1M tokens        │ │    │
│  │  │ ○ Claude Haiku        $0.25/$1.25 per 1M tokens    │ │    │
│  │  │ ○ GPT-4o              $2.50/$10 per 1M tokens      │ │    │
│  │  │ ○ GPT-4o-mini         $0.15/$0.60 per 1M tokens    │ │    │
│  │  │ ○ Ollama (Local)      Free (self-hosted)           │ │    │
│  │  └────────────────────────────────────────────────────┘ │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Per-Agent Override (Optional)                                   │
│  ─────────────────────────────                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Agent        Default Model     Override       Reason   │    │
│  │  ──────────   ─────────────     ────────       ──────── │    │
│  │  🤖 Rex       Sonnet 4          [Opus 4 ▼]     Complex  │    │
│  │  🔥 Blaze     Sonnet 4          [Default ▼]    -        │    │
│  │  🔐 Cypher    Sonnet 4          [Opus 4 ▼]     Security │    │
│  │  🛡️ Guardian  Sonnet 4          [Haiku ▼]      Simple   │    │
│  │                                                          │    │
│  │  💡 Tip: Use Opus for complex tasks, Haiku for routine  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Budget Controls                                                 │
│  ───────────────                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Monthly budget for this project:  [$500         ]      │    │
│  │                                                          │    │
│  │  When budget exceeded:                                   │    │
│  │  ○ Alert only (continue running)                        │    │
│  │  ● Switch to cheaper model (Haiku)                      │    │
│  │  ○ Pause agents (require approval)                      │    │
│  │                                                          │    │
│  │  Alert thresholds:                                       │    │
│  │  ☑ 50% - Slack notification                             │    │
│  │  ☑ 75% - Slack + Email                                  │    │
│  │  ☑ 90% - Slack + Email + Mobile push                    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  [Save Configuration]                                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Token Usage Details:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Token Usage > acme-webapp > Task #42                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Task: Implement user authentication API                        │
│  Status: Completed                                               │
│  Duration: 2h 34m                                                │
│                                                                  │
│  Token Summary                                                   │
│  ─────────────                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Total Tokens:  847,293                                   │   │
│  │  ├── Input:     612,847  (72%)                           │   │
│  │  └── Output:    234,446  (28%)                           │   │
│  │                                                           │   │
│  │  Total Cost:    $8.42                                     │   │
│  │  Model:         Claude Sonnet 4                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Breakdown by Agent                                              │
│  ──────────────────                                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Agent      Calls   Input     Output    Cost    % Total  │   │
│  │  ─────────  ─────   ─────     ──────    ────    ───────  │   │
│  │  🤖 Rex      34    412K      156K     $5.68     67.4%   │   │
│  │  🔐 Cypher    8    142K       62K     $2.04     24.2%   │   │
│  │  🔥 Blaze     6     58K       16K     $0.70      8.4%   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Token Timeline                                                  │
│  ──────────────                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Tokens                                                   │   │
│  │  150K ┤                    ╭──╮                           │   │
│  │       │        ╭───╮      │  │   ╭──╮                    │   │
│  │  100K ┤   ╭───╮│   │ ╭───╮│  │   │  │                    │   │
│  │       │   │   ││   │ │   ││  │   │  │                    │   │
│  │   50K ┤╭──│   ││   │ │   ││  │╭──│  │╭──╮                │   │
│  │       │   │   ││   │ │   ││  ││  │  ││  │                │   │
│  │     0 ┼───┴───┴┴───┴─┴───┴┴──┴┴──┴──┴┴──┴───────────────│   │
│  │       10am    11am    12pm    1pm    2pm                  │   │
│  │                                                           │   │
│  │  ■ Rex  ■ Cypher  ■ Blaze                                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  [Export CSV] [View Raw Logs]                                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Model Pricing Reference:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Model Pricing (Updated Automatically)                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Provider    Model              Input/1M    Output/1M   Speed   │
│  ──────────  ─────────────────  ────────    ─────────   ─────   │
│                                                                  │
│  Anthropic                                                       │
│  ├─ Claude Opus 4              $15.00      $75.00      ███░░   │
│  ├─ Claude Sonnet 4            $3.00       $15.00      ████░   │
│  └─ Claude Haiku               $0.25       $1.25       █████   │
│                                                                  │
│  OpenAI                                                          │
│  ├─ GPT-4o                     $2.50       $10.00      ████░   │
│  ├─ GPT-4o-mini                $0.15       $0.60       █████   │
│  └─ o1                         $15.00      $60.00      ██░░░   │
│                                                                  │
│  Google                                                          │
│  ├─ Gemini 1.5 Pro             $1.25       $5.00       ████░   │
│  └─ Gemini 1.5 Flash           $0.075      $0.30       █████   │
│                                                                  │
│  Local (Ollama)                                                  │
│  ├─ Llama 3.3 70B              $0.00       $0.00       ███░░   │
│  ├─ Qwen 2.5 72B               $0.00       $0.00       ███░░   │
│  └─ Mistral Large              $0.00       $0.00       ███░░   │
│                                                                  │
│  Last updated: 2 hours ago  [Refresh Prices]                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Quality vs Cost Intelligence:**

The system tracks not just token costs, but also **effective cost** - accounting 
for rework, bugs, and task complexity. A cheaper model that produces bugs can 
end up costing more than an expensive model that gets it right the first time.

```
┌─────────────────────────────────────────────────────────────────┐
│              Quality vs Cost Analysis System                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Metrics Tracked Per Model:                                     │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  📊 First-Pass Success Rate                             │    │
│  │     % of tasks completed without rework                 │    │
│  │                                                          │    │
│  │  🔄 Rework Rate                                         │    │
│  │     % of tasks requiring fixes after initial completion │    │
│  │                                                          │    │
│  │  🐛 Bug Introduction Rate                               │    │
│  │     Bugs found per 1000 lines of generated code        │    │
│  │                                                          │    │
│  │  ⏱️ Time to Completion                                  │    │
│  │     Average time from task start to PR merge           │    │
│  │                                                          │    │
│  │  💰 Effective Cost                                      │    │
│  │     Token cost + (Rework cost × Rework probability)    │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│                                                                  │
│  Task Complexity Assessment:                                    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  SIMPLE              MODERATE           COMPLEX          │    │
│  │  ────────            ────────           ───────          │    │
│  │  • Config changes    • New endpoints    • Architecture   │    │
│  │  • Text updates      • CRUD operations  • Security-crit  │    │
│  │  • Simple UI tweaks  • Standard UI      • Data migration │    │
│  │  • Documentation     • Unit tests       • Integrations   │    │
│  │  • Dependency bumps  • Bug fixes        • Performance    │    │
│  │                                                          │    │
│  │  Haiku OK ✓          Sonnet ideal       Opus preferred   │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Quality-Cost Matrix Dashboard:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Quality vs Cost Analysis                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Model Performance (Your Data - Last 30 Days)                   │
│  ────────────────────────────────────────────                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Model          Token    First-  Rework   Effective  ROI │   │
│  │                 Cost     Pass    Rate     Cost            │   │
│  │  ─────────────  ──────   ──────  ──────   ──────────  ─── │   │
│  │                                                           │   │
│  │  Claude Opus    $15/75   97%     3%       $16.20     ⭐⭐⭐ │   │
│  │  (complex)      /1M tok                   per task        │   │
│  │                                                           │   │
│  │  Claude Sonnet  $3/15    92%     8%       $4.80      ⭐⭐⭐ │   │
│  │  (moderate)     /1M tok                   per task        │   │
│  │                                                           │   │
│  │  Claude Haiku   $0.25    78%     22%      $1.20      ⭐⭐  │   │
│  │  (simple only)  /1M tok                   per task        │   │
│  │                                                           │   │
│  │  Haiku on       $0.25    41%     59%      $8.40      ⚠️   │   │
│  │  complex tasks  /1M tok          ⚠️        per task       │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ⚠️ Warning: Using Haiku on complex tasks costs MORE than      │
│     Opus due to 59% rework rate (avg 3.2 attempts to complete) │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  Cost Visualization (by complexity)                              │
│  ──────────────────────────────────                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Effective    │                              ╭── Haiku   │   │
│  │  Cost ($)     │                         ╭────╯            │   │
│  │               │                    ╭────╯                 │   │
│  │    $20 ─┤     │               ╭────╯                      │   │
│  │               │          ╭────╯                           │   │
│  │    $15 ─┤     │     ╭────╯                   ╭── Sonnet  │   │
│  │               │╭────╯               ╭────────╯            │   │
│  │    $10 ─┤     ╭╯           ╭────────╯                     │   │
│  │              ╱│   ╭────────╯               ╭── Opus      │   │
│  │     $5 ─┤  ╱  ╭───╯           ╭────────────╯             │   │
│  │        ╱╭──╯╭─╯  ╭────────────╯                           │   │
│  │     $0 ┼──────────────────────────────────────────────    │   │
│  │        Simple     Moderate      Complex                   │   │
│  │                                                           │   │
│  │  Takeaway: Haiku wins for simple, Sonnet for moderate,   │   │
│  │            Opus for complex (despite higher token cost)   │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Smart Model Selection (Auto-Detect Complexity):**

```
┌─────────────────────────────────────────────────────────────────┐
│                 Intelligent Model Selection                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  When a new task arrives, the system analyzes:                  │
│                                                                  │
│  1. PRD/Description Analysis                                    │
│     ├── Keyword detection (security, migration, refactor)      │
│     ├── Scope indicators (files affected, dependencies)        │
│     └── Historical similarity to past tasks                    │
│                                                                  │
│  2. Codebase Context                                            │
│     ├── Complexity of affected files                           │
│     ├── Test coverage requirements                              │
│     └── Integration touchpoints                                 │
│                                                                  │
│  3. Risk Assessment                                              │
│     ├── Production impact                                       │
│     ├── Security sensitivity                                    │
│     └── Data handling                                           │
│                                                                  │
│                          ▼                                      │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  📋 Task: "Add email field validation"                  │    │
│  │                                                          │    │
│  │  Complexity Assessment: SIMPLE                          │    │
│  │  ├── Single file change                                 │    │
│  │  ├── Standard regex validation                          │    │
│  │  ├── No security implications                           │    │
│  │  └── Similar to 12 past tasks (94% success with Haiku) │    │
│  │                                                          │    │
│  │  Recommended: Claude Haiku                              │    │
│  │  Estimated cost: $0.08                                   │    │
│  │  Confidence: 96%                                         │    │
│  │                                                          │    │
│  │  [Use Haiku] [Override to Sonnet] [Override to Opus]    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  📋 Task: "Implement OAuth2 with PKCE flow"             │    │
│  │                                                          │    │
│  │  Complexity Assessment: COMPLEX                         │    │
│  │  ├── Multiple file changes (8+ files)                   │    │
│  │  ├── Security-critical (authentication)                 │    │
│  │  ├── External integration (OAuth provider)              │    │
│  │  └── Similar tasks: 67% success Sonnet, 94% success Opus│    │
│  │                                                          │    │
│  │  Recommended: Claude Opus                               │    │
│  │  Estimated cost: $12.40                                  │    │
│  │  Confidence: 89%                                         │    │
│  │                                                          │    │
│  │  ⚠️ Using Sonnet would likely require 1.4 attempts     │    │
│  │     (est. $8.20 × 1.4 = $11.48, similar to Opus)        │    │
│  │                                                          │    │
│  │  [Use Opus] [Override to Sonnet] [Override to Haiku]    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Model Selection Settings:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Settings > Model Selection Strategy                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Auto-Select Mode                                                │
│  ────────────────                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ● Cost-Optimized (Recommended)                         │    │
│  │    System selects cheapest model likely to succeed      │    │
│  │    on first attempt. Balances cost vs rework risk.     │    │
│  │                                                          │    │
│  │  ○ Quality-First                                        │    │
│  │    Always use best available model. Minimizes rework    │    │
│  │    but higher upfront costs.                            │    │
│  │                                                          │    │
│  │  ○ Budget-Constrained                                   │    │
│  │    Stay within budget, accept potential rework.         │    │
│  │    Best for non-critical projects.                      │    │
│  │                                                          │    │
│  │  ○ Manual Only                                          │    │
│  │    Always ask for model selection. No auto-routing.    │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Complexity Thresholds                                           │
│  ─────────────────────                                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Simple tasks (auto-route to Haiku):                    │    │
│  │  ☑ Config/env changes                                   │    │
│  │  ☑ Documentation updates                                │    │
│  │  ☑ Dependency version bumps                             │    │
│  │  ☑ Simple text/copy changes                             │    │
│  │                                                          │    │
│  │  Force Opus for:                                         │    │
│  │  ☑ Any task with "security" in description              │    │
│  │  ☑ Database migrations                                   │    │
│  │  ☑ Authentication/authorization changes                 │    │
│  │  ☑ Payment/billing code                                  │    │
│  │  ☑ Tasks affecting 5+ files                             │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Learning Mode                                                   │
│  ─────────────                                                   │
│  ☑ Track outcomes to improve future recommendations             │
│  ☑ Alert when a model consistently underperforms               │
│  ☑ Suggest model changes based on accumulated data             │
│                                                                  │
│  [Save Settings]                                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Rework Tracking:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Rework Analysis > This Month                                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Rework Events                                                   │
│  ─────────────                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Task         Model    Attempts  Rework Cost   Root Cause│   │
│  │  ───────────  ──────   ────────  ───────────   ──────────│   │
│  │                                                           │   │
│  │  #42 Auth     Sonnet   2         $8.40         Edge case │   │
│  │  #38 API      Haiku    4         $3.20         Complexity│   │
│  │  #35 UI       Haiku    3         $1.80         Complexity│   │
│  │  #31 DB       Sonnet   2         $12.20        Schema    │   │
│  │                                                           │   │
│  │  Total rework cost this month: $25.60                    │   │
│  │  (4.2% of total spend)                                   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Insight: Tasks #38 and #35 used Haiku but were moderate       │
│  complexity. Switching to Sonnet would have cost $4.80 each    │
│  vs $3.20 and $1.80 in rework. Net savings: $0.60              │
│                                                                  │
│  Recommendation: Tighten Haiku eligibility criteria            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Telemetry-Driven Quality Feedback Loop:**

The telemetry system continuously monitors for errors and quality issues, feeding 
data back into model selection. If the QA/code quality agent spends excessive time 
fixing issues, that's a strong signal the implementation model wasn't sufficient.

```
┌─────────────────────────────────────────────────────────────────┐
│              Telemetry → Quality → Cost Feedback Loop            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Implementation                        │    │
│  │                    (Rex, Blaze, etc.)                   │    │
│  │                         │                                │    │
│  │                         ▼                                │    │
│  │  ┌─────────────────────────────────────────────────┐    │    │
│  │  │              Code Quality Check                  │    │    │
│  │  │              (Cypher Agent)                      │    │    │
│  │  │                                                  │    │    │
│  │  │  • Linting errors                               │    │    │
│  │  │  • Type errors                                  │    │    │
│  │  │  • Test failures                                │    │    │
│  │  │  • Security vulnerabilities                     │    │    │
│  │  │  • Code smell detection                         │    │    │
│  │  │  • Complexity analysis                          │    │    │
│  │  └──────────────────────┬──────────────────────────┘    │    │
│  │                         │                                │    │
│  │           ┌─────────────┴─────────────┐                 │    │
│  │           │                           │                 │    │
│  │           ▼                           ▼                 │    │
│  │     ┌──────────┐              ┌──────────────┐          │    │
│  │     │  Pass ✓  │              │  Issues ⚠️   │          │    │
│  │     │          │              │              │          │    │
│  │     │ Ship it! │              │ Fix needed   │          │    │
│  │     └──────────┘              └──────┬───────┘          │    │
│  │                                      │                  │    │
│  │                                      ▼                  │    │
│  │                          ┌─────────────────────┐        │    │
│  │                          │  Telemetry Records  │        │    │
│  │                          │  ─────────────────  │        │    │
│  │                          │  • Error type       │        │    │
│  │                          │  • Severity         │        │    │
│  │                          │  • Time to fix      │        │    │
│  │                          │  • Fix token cost   │        │    │
│  │                          │  • Model used       │        │    │
│  │                          │  • Task complexity  │        │    │
│  │                          └─────────┬───────────┘        │    │
│  │                                    │                    │    │
│  └────────────────────────────────────┼────────────────────┘    │
│                                       │                         │
│                                       ▼                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Quality Intelligence Engine                 │    │
│  │                                                          │    │
│  │  Aggregates telemetry to learn:                         │    │
│  │  • Which models produce most errors by task type        │    │
│  │  • Error patterns (e.g., Haiku misses edge cases)      │    │
│  │  • True cost = tokens + QA time + fix time              │    │
│  │  • Optimal model routing based on historical data       │    │
│  │                                                          │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Model Selection (Next Task)                 │    │
│  │                                                          │    │
│  │  "Tasks like this with Haiku → 45% QA fix rate         │    │
│  │   Upgrading to Sonnet for this task type"               │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Quality Metrics Dashboard:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Code Quality Telemetry                                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  QA Agent Activity (This Week)                                   │
│  ─────────────────────────────                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  📊 Tasks Reviewed:     47                               │   │
│  │  ✅ First-Pass Clean:   38 (81%)                         │   │
│  │  🔧 Required Fixes:      9 (19%)                         │   │
│  │                                                           │   │
│  │  ⏱️ Avg QA Time:        4.2 min (clean) / 18 min (fixes)│   │
│  │  💰 QA Token Cost:      $42.30                           │   │
│  │  💰 Fix Token Cost:     $28.70                           │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Error Breakdown by Type                                         │
│  ───────────────────────                                         │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Error Type          Count   Avg Fix Cost   Model        │   │
│  │  ──────────────────  ─────   ────────────   ─────────    │   │
│  │  🐛 Logic errors       4     $4.20          Haiku (3)    │   │
│  │  ⚠️ Type mismatches     3     $1.80          Mixed        │   │
│  │  🔒 Security issues     1     $8.40          Sonnet       │   │
│  │  📝 Missing tests       1     $2.10          Haiku        │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ⚠️ Pattern Detected: 75% of logic errors from Haiku           │
│     on tasks classified as "moderate" complexity                │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  Error Rate by Model (Last 30 Days)                             │
│  ──────────────────────────────────                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Model          Tasks   Clean   Fix Rate   Avg Fix Cost  │   │
│  │  ─────────────  ─────   ─────   ────────   ────────────  │   │
│  │  Claude Opus      23    22       4%        $6.20         │   │
│  │  Claude Sonnet    89    78      12%        $3.40         │   │
│  │  Claude Haiku     41    27      34%        $2.80         │   │
│  │  Ollama Local     12     9      25%        $0 (local)    │   │
│  │                                                           │   │
│  │  💡 Haiku's 34% fix rate on moderate tasks makes it     │   │
│  │     MORE expensive than Sonnet when you factor in QA    │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Real-Time Quality Alerts:**

```
┌─────────────────────────────────────────────────────────────────┐
│  🔔 Quality Alert                                     Just now  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  ⚠️ High Error Rate Detected                             │   │
│  │                                                           │   │
│  │  Project: payment-api                                     │   │
│  │  Model: Claude Haiku                                      │   │
│  │                                                           │   │
│  │  Last 5 tasks:                                            │   │
│  │  • Task #51: 3 errors, 2 fix cycles ($4.20)              │   │
│  │  • Task #52: 5 errors, 3 fix cycles ($6.80)              │   │
│  │  • Task #53: 2 errors, 1 fix cycle ($2.10)               │   │
│  │  • Task #54: 4 errors, 2 fix cycles ($5.40)              │   │
│  │  • Task #55: 6 errors, 4 fix cycles ($9.20)              │   │
│  │                                                           │   │
│  │  Total QA overhead: $27.70 (vs $4.80 Haiku token cost)   │   │
│  │                                                           │   │
│  │  If using Sonnet: Est. $24.00 tokens, ~$3.60 QA overhead │   │
│  │  Net savings with Sonnet: ~$5.90 per 5 tasks             │   │
│  │                                                           │   │
│  │  [Switch to Sonnet] [Investigate Tasks] [Dismiss]        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**True Cost Calculation:**

```
┌─────────────────────────────────────────────────────────────────┐
│              True Cost Formula                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  True Cost = Implementation Tokens                              │
│            + QA Review Tokens                                    │
│            + Fix Tokens × P(fix needed)                         │
│            + Re-QA Tokens × P(fix needed)                       │
│            + (Recursive if multiple fix cycles)                 │
│                                                                  │
│  ───────────────────────────────────────────────────────────── │
│                                                                  │
│  Example: "Add user profile endpoint"                           │
│                                                                  │
│  With Haiku ($0.25/$1.25 per 1M):                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Implementation:    50K tokens      $0.06                │   │
│  │  QA Review:         20K tokens      $0.03                │   │
│  │  P(fix needed):     34%                                   │   │
│  │  Fix (if needed):   30K tokens      $0.04                │   │
│  │  Re-QA:             15K tokens      $0.02                │   │
│  │  P(2nd fix):        15%                                   │   │
│  │  ─────────────────────────────────────────────────────── │   │
│  │  Expected cost:     $0.06 + $0.03 + 0.34×($0.04+$0.02)   │   │
│  │                     + 0.15×($0.04+$0.02)                 │   │
│  │                   = $0.12                                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  With Sonnet ($3/$15 per 1M):                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Implementation:    40K tokens      $0.12                │   │
│  │  QA Review:         15K tokens      $0.05                │   │
│  │  P(fix needed):     8%                                    │   │
│  │  Fix (if needed):   20K tokens      $0.06                │   │
│  │  Re-QA:             10K tokens      $0.03                │   │
│  │  P(2nd fix):        2%                                    │   │
│  │  ─────────────────────────────────────────────────────── │   │
│  │  Expected cost:     $0.12 + $0.05 + 0.08×($0.06+$0.03)   │   │
│  │                     + 0.02×($0.06+$0.03)                 │   │
│  │                   = $0.18                                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  For this simple task: Haiku wins ($0.12 vs $0.18)             │
│  But for complex tasks: Sonnet's lower P(fix) wins             │
│                                                                  │
│  The system learns these probabilities from YOUR telemetry     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Telemetry Configuration:**

```yaml
# quality-telemetry.yaml
apiVersion: cto.5dlabs.io/v1
kind: QualityTelemetry

collection:
  enabled: true
  
  # Track these quality signals
  signals:
    - lint_errors
    - type_errors  
    - test_failures
    - security_vulnerabilities
    - code_complexity
    - cyclomatic_complexity
    
  # Track QA agent activity
  qaMetrics:
    - review_duration
    - issues_found
    - fix_cycles
    - fix_token_cost
    
analysis:
  # Update model performance stats
  modelPerformance:
    enabled: true
    windowDays: 30
    minSamples: 10  # Need 10 tasks before making recommendations
    
  # Detect patterns
  patternDetection:
    enabled: true
    alertThreshold: 0.30  # Alert if error rate > 30%
    
  # Feed into model selection
  modelRouting:
    enabled: true
    updateFrequency: daily
    
alerts:
  highErrorRate:
    threshold: 0.25
    window: "5 tasks"
    channels: [slack, web]
    
  costAnomaly:
    threshold: 2.0  # 2x expected cost
    channels: [slack, email]
    
  modelUnderperforming:
    threshold: 0.20  # 20% worse than baseline
    channels: [web]

reporting:
  weeklyDigest: true
  includeRecommendations: true
```

### Opt-In Anonymized Telemetry (Phone Home)

Users can opt-in to share anonymized metrics with CTO Platform to help improve
cost optimization and model recommendations across all customers.

```
┌─────────────────────────────────────────────────────────────────┐
│              Anonymized Telemetry Architecture                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Customer Instance                        CTO Platform Cloud    │
│  ─────────────────                        ──────────────────    │
│                                                                  │
│  ┌─────────────────────┐                                        │
│  │  Local Telemetry    │                                        │
│  │  (Full Detail)      │                                        │
│  │                     │                                        │
│  │  • Project names    │                                        │
│  │  • Task descriptions│                                        │
│  │  • Code content     │◄── NEVER SHARED                       │
│  │  • File paths       │                                        │
│  │  • User info        │                                        │
│  └──────────┬──────────┘                                        │
│             │                                                    │
│             ▼                                                    │
│  ┌─────────────────────┐                                        │
│  │  Anonymization      │                                        │
│  │  Layer              │                                        │
│  │                     │                                        │
│  │  • Strip PII        │                                        │
│  │  • Hash identifiers │                                        │
│  │  • Aggregate stats  │                                        │
│  │  • Remove content   │                                        │
│  └──────────┬──────────┘                                        │
│             │                                                    │
│             │ (Opt-in only)                                     │
│             ▼                                                    │
│  ┌─────────────────────┐      ┌─────────────────────────────┐  │
│  │  Anonymized Metrics │─────►│  CTO Insights Database      │  │
│  │                     │      │                             │  │
│  │  • Model + task type│      │  Aggregates from all        │  │
│  │  • Success/fail     │      │  opt-in customers           │  │
│  │  • Token counts     │      │                             │  │
│  │  • Error categories │      │  Powers:                    │  │
│  │  • Fix cycle count  │      │  • Global recommendations   │  │
│  │  • Complexity score │      │  • Model benchmarks         │  │
│  │                     │      │  • Best practice detection  │  │
│  └─────────────────────┘      └─────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**What We Collect (Opt-In Only):**

```
┌─────────────────────────────────────────────────────────────────┐
│  Anonymized Telemetry Data                                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ✅ SHARED (if opted-in)           ❌ NEVER SHARED              │
│  ─────────────────────────          ──────────────────────────  │
│                                                                  │
│  • Model used                       • Source code               │
│  • Task complexity (1-10)           • Project/file names        │
│  • Task category (auth, api, ui)    • Task descriptions         │
│  • Token counts (in/out)            • Error messages (content)  │
│  • Success/failure                  • User/org identity         │
│  • Error type (category only)       • IP addresses              │
│  • Fix cycles needed                • API keys                  │
│  • Time to completion               • Business logic            │
│  • Language/framework               • Customer data             │
│  • Platform version                 • Internal URLs             │
│                                                                  │
│  Example anonymized record:                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ {                                                         │   │
│  │   "instance_id": "a1b2c3d4",  // Random, rotates monthly │   │
│  │   "model": "claude-sonnet-4",                            │   │
│  │   "task_type": "api_endpoint",                           │   │
│  │   "complexity": 6,                                        │   │
│  │   "language": "typescript",                               │   │
│  │   "framework": "nextjs",                                  │   │
│  │   "input_tokens": 45230,                                  │   │
│  │   "output_tokens": 12847,                                 │   │
│  │   "success": true,                                        │   │
│  │   "fix_cycles": 1,                                        │   │
│  │   "error_types": ["type_error"],                         │   │
│  │   "qa_time_seconds": 142                                  │   │
│  │ }                                                         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Opt-In Settings UI:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Settings > Privacy & Telemetry                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Help Improve CTO Platform                                       │
│  ─────────────────────────                                       │
│                                                                  │
│  Share anonymized usage data to help us improve model           │
│  recommendations and cost optimization for everyone.            │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ☑ Share anonymized metrics with CTO Platform           │    │
│  │                                                          │    │
│  │    What's shared:                                        │    │
│  │    ✓ Model performance statistics                       │    │
│  │    ✓ Task complexity and success rates                  │    │
│  │    ✓ Token usage patterns                               │    │
│  │    ✓ Error categories (not content)                     │    │
│  │                                                          │    │
│  │    What's NEVER shared:                                  │    │
│  │    ✗ Source code or file contents                       │    │
│  │    ✗ Project names or descriptions                      │    │
│  │    ✗ Personal or company information                    │    │
│  │    ✗ API keys or credentials                            │    │
│  │                                                          │    │
│  │  [View sample data] [View privacy policy]               │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Sharing Level                                                   │
│  ─────────────                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ○ Off - No data shared                                 │    │
│  │                                                          │    │
│  │  ● Basic - Model performance only                       │    │
│  │    (model, success/fail, token counts)                  │    │
│  │                                                          │    │
│  │  ○ Enhanced - Include task patterns                     │    │
│  │    (+ complexity, error types, framework)               │    │
│  │                                                          │    │
│  │  ○ Full - All anonymized metrics                        │    │
│  │    (+ timing, fix cycles, detailed categories)          │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Benefits of Sharing                                             │
│  ───────────────────                                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  🎁 Customers who opt-in receive:                       │    │
│  │                                                          │    │
│  │  • Global benchmarks (how do you compare?)              │    │
│  │  • Early access to improved recommendations             │    │
│  │  • Community-powered model rankings                     │    │
│  │  • Best practice insights from aggregate data           │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  [Save Preferences]                                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**How Aggregate Data Helps Everyone:**

```
┌─────────────────────────────────────────────────────────────────┐
│              Global Insights from Aggregate Telemetry            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  With data from 1,000+ opt-in instances, we can provide:       │
│                                                                  │
│  1. GLOBAL MODEL BENCHMARKS                                     │
│  ───────────────────────────                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Model          Avg Success   Best For          Avoid    │   │
│  │  ─────────────  ───────────   ────────────────  ──────── │   │
│  │  Claude Opus    96.2%         Security, complex Multi-file│  │
│  │  Claude Sonnet  91.4%         General, APIs     Simple   │   │
│  │  Claude Haiku   72.8%         Config, docs      Logic    │   │
│  │  GPT-4o         89.1%         Frontend, UI      Backend  │   │
│  │  Llama 3.3 70B  68.4%         Simple tasks      Complex  │   │
│  │                                                           │   │
│  │  Based on 847,000 tasks across all opt-in customers      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  2. FRAMEWORK-SPECIFIC RECOMMENDATIONS                          │
│  ─────────────────────────────────────                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  "For Next.js API routes, Sonnet outperforms Opus        │   │
│  │   (93% vs 94% success, but 5x cheaper). However, for     │   │
│  │   Next.js middleware, Opus has 12% higher success rate." │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  3. COST OPTIMIZATION PATTERNS                                  │
│  ─────────────────────────────                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Discovered patterns:                                     │   │
│  │  • Teams using prompt caching save avg 34% on tokens     │   │
│  │  • Haiku on "simple" tasks saves $4.20/task avg          │   │
│  │  • Splitting large tasks into smaller ones: +18% success │   │
│  │  • Adding examples to prompts: +12% first-pass rate      │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  4. EARLY WARNING SYSTEM                                        │
│  ───────────────────────                                        │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  "We've detected Claude Sonnet 4.1 has 8% lower success  │   │
│  │   on TypeScript generics. Rolling back recommendation    │   │
│  │   until Anthropic addresses this."                       │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**"How Do I Compare?" Dashboard (Opt-In Customers):**

```
┌─────────────────────────────────────────────────────────────────┐
│  Benchmarks (Compared to Similar Teams)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Your team compared to similar-sized teams using CTO Platform   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  Metric              You      Avg       Percentile       │   │
│  │  ──────────────────  ───      ───       ──────────       │   │
│  │  First-pass rate     89%      84%       Top 25%  ⭐      │   │
│  │  Cost per task       $2.40    $3.10     Top 15%  ⭐      │   │
│  │  Fix cycles/task     1.2      1.4       Top 30%          │   │
│  │  QA overhead         12%      18%       Top 10%  ⭐      │   │
│  │                                                           │   │
│  │  💡 You're more cost-efficient than 85% of similar teams │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Improvement Opportunities (Based on Top Performers)            │
│  ───────────────────────────────────────────────────             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                                           │   │
│  │  📈 Top 10% teams use Haiku for 40% of tasks (you: 22%) │   │
│  │     Potential savings: $180/month                        │   │
│  │     [See which tasks qualify]                            │   │
│  │                                                           │   │
│  │  📈 Top teams enable prompt caching (you: disabled)      │   │
│  │     Avg savings: 28% on token costs                      │   │
│  │     [Enable prompt caching]                              │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Telemetry Phone-Home Configuration:**

```yaml
# phone-home-telemetry.yaml
apiVersion: cto.5dlabs.io/v1
kind: PhoneHomeTelemetry

enabled: true  # User opted-in
level: enhanced  # basic | enhanced | full

# What to share
sharing:
  modelPerformance: true
  taskCategories: true
  errorTypes: true
  frameworkStats: true
  timingData: true
  tokenCounts: true
  
# Privacy settings  
privacy:
  # Rotate instance ID monthly (prevent long-term tracking)
  instanceIdRotation: monthly
  
  # Strip any detected PII before sending
  piiDetection: true
  
  # Aggregate locally before sending (reduces granularity)
  localAggregation: hourly
  
# Data transmission
transmission:
  endpoint: https://telemetry.ctoplatform.io/v1/ingest
  batchSize: 100
  interval: 1h
  
  # Retry with exponential backoff
  retries: 3
  
  # Show user what's being sent
  auditLog: true
  
# User can always view what's being sent
auditDashboard:
  enabled: true
  retentionDays: 30

# Allow user to delete their data
dataRights:
  exportEnabled: true
  deleteEnabled: true  # GDPR compliance
```

**What CTO Platform Does With This Data:**

```
┌─────────────────────────────────────────────────────────────────┐
│              CTO Platform Insights Pipeline                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Aggregate Data from Opt-In Customers                           │
│                    │                                             │
│                    ▼                                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   Analysis Engine                        │    │
│  │                                                          │    │
│  │  • Model performance by task type                       │    │
│  │  • Framework-specific patterns                          │    │
│  │  • Error rate trends                                     │    │
│  │  • Cost optimization opportunities                      │    │
│  │  • New model evaluation                                  │    │
│  │                                                          │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │                                   │
│              ┌───────────────┼───────────────┐                  │
│              │               │               │                  │
│              ▼               ▼               ▼                  │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐         │
│  │  Updated      │ │  Blog Posts   │ │  Model        │         │
│  │  Defaults     │ │  & Reports    │ │  Rankings     │         │
│  │  for All      │ │  (Public)     │ │  Page         │         │
│  │  Customers    │ │               │ │               │         │
│  └───────────────┘ └───────────────┘ └───────────────┘         │
│                                                                  │
│  Examples:                                                       │
│  • "Default to Sonnet for auth tasks (97% success rate)"       │
│  • "Recommend Haiku for config changes (saves 82%)"            │
│  • "New model X performs 5% better than Y on frontend"         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Cost Optimization Recommendations:**

```
┌─────────────────────────────────────────────────────────────────┐
│  💡 Cost Optimization Insights                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 🎯 Recommendation: Switch Guardian to Haiku              │   │
│  │                                                           │   │
│  │ Guardian agent runs routine health checks that don't     │   │
│  │ require Sonnet's capabilities. Switching to Haiku would  │   │
│  │ save ~$45/month with no quality impact.                  │   │
│  │                                                           │   │
│  │ Current: $52/mo (Sonnet) → Proposed: $7/mo (Haiku)       │   │
│  │ Quality impact: None (100% success rate on simple tasks) │   │
│  │                                                           │   │
│  │ [Apply This Change] [Dismiss]                            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ⚠️ Alert: Haiku struggling on payment-api project        │   │
│  │                                                           │   │
│  │ Last 5 tasks using Haiku had 60% rework rate.            │   │
│  │ These tasks are too complex for Haiku.                   │   │
│  │                                                           │   │
│  │ Haiku cost: $2.40 + $4.80 rework = $7.20 effective      │   │
│  │ Sonnet cost: $4.80 (likely first-pass success)          │   │
│  │                                                           │   │
│  │ Recommendation: Use Sonnet as default for this project  │   │
│  │                                                           │   │
│  │ [Switch to Sonnet] [Keep Haiku] [Analyze Tasks]          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 🎯 Recommendation: Enable prompt caching                 │   │
│  │                                                           │   │
│  │ Your projects have 67% prompt overlap. Enabling prompt   │   │
│  │ caching could reduce costs by ~30%.                      │   │
│  │                                                           │   │
│  │ Estimated savings: $180/month                            │   │
│  │                                                           │   │
│  │ [Enable Caching] [Learn More]                            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 🎯 Recommendation: Use local models for admin-dashboard  │   │
│  │                                                           │   │
│  │ This project has simple tasks suitable for local models. │   │
│  │ Running Llama 3.3 70B locally would eliminate API costs. │   │
│  │ Historical data shows 89% first-pass success for similar │   │
│  │ simple tasks with local models.                          │   │
│  │                                                           │   │
│  │ Current: $44/mo → Proposed: $0/mo (local compute only)   │   │
│  │                                                           │   │
│  │ [Configure Local Model] [Dismiss]                        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Cost Configuration YAML:**

```yaml
# cost-config.yaml
apiVersion: cto.5dlabs.io/v1
kind: CostConfiguration

global:
  defaultModel: claude-sonnet-4
  monthlyBudget: 1000
  budgetActions:
    at50Percent: [slack]
    at75Percent: [slack, email]
    at90Percent: [slack, email, mobile]
    atLimit: switchToHaiku  # or: alertOnly, pauseAgents

models:
  - id: claude-opus-4
    provider: anthropic
    inputCostPer1M: 15.00
    outputCostPer1M: 75.00
    enabled: true
  - id: claude-sonnet-4
    provider: anthropic
    inputCostPer1M: 3.00
    outputCostPer1M: 15.00
    enabled: true
    default: true
  - id: claude-haiku
    provider: anthropic
    inputCostPer1M: 0.25
    outputCostPer1M: 1.25
    enabled: true
    fallback: true
  - id: ollama-llama-3.3
    provider: local
    inputCostPer1M: 0
    outputCostPer1M: 0
    enabled: true

projects:
  acme-webapp:
    defaultModel: claude-sonnet-4
    monthlyBudget: 500
    agentOverrides:
      rex: claude-opus-4      # Complex backend work
      cypher: claude-opus-4   # Security-critical
      guardian: claude-haiku  # Simple monitoring
      
  admin-dashboard:
    defaultModel: ollama-llama-3.3  # Use local model
    monthlyBudget: 0  # Free (local)

reporting:
  dailyDigest: true
  weeklyReport: true
  exportFormat: [csv, pdf]
  retentionDays: 90
```

**Token Tracking Schema:**

```sql
-- Token usage tracking schema
CREATE TABLE token_usage (
  id UUID PRIMARY KEY,
  timestamp TIMESTAMPTZ NOT NULL,
  
  -- Context
  project_id VARCHAR(255) NOT NULL,
  task_id VARCHAR(255),
  agent_id VARCHAR(255) NOT NULL,
  
  -- Model info
  provider VARCHAR(50) NOT NULL,
  model VARCHAR(100) NOT NULL,
  
  -- Token counts
  input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  cached_tokens INTEGER DEFAULT 0,
  
  -- Cost (calculated at time of request)
  input_cost_usd DECIMAL(10, 6) NOT NULL,
  output_cost_usd DECIMAL(10, 6) NOT NULL,
  total_cost_usd DECIMAL(10, 6) NOT NULL,
  
  -- Metadata
  request_type VARCHAR(50),  -- 'completion', 'embedding', etc.
  latency_ms INTEGER,
  
  -- Indexes for fast queries
  INDEX idx_project_timestamp (project_id, timestamp),
  INDEX idx_model_timestamp (model, timestamp),
  INDEX idx_agent_timestamp (agent_id, timestamp)
);

-- Aggregated daily stats for fast dashboard queries
CREATE TABLE daily_cost_summary (
  date DATE NOT NULL,
  project_id VARCHAR(255) NOT NULL,
  model VARCHAR(100) NOT NULL,
  
  total_requests INTEGER,
  total_input_tokens BIGINT,
  total_output_tokens BIGINT,
  total_cost_usd DECIMAL(12, 2),
  
  PRIMARY KEY (date, project_id, model)
);
```

### Frontend Template Management (v1.3+)

A system for managing UI designs and component libraries using shadcn/ui, 
enabling consistent frontend development across projects.

```
┌─────────────────────────────────────────────────────────────────┐
│                Frontend Template Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Design System Management                                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Your Design Systems                                     │    │
│  │  ═══════════════════                                     │    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ 🎨 Acme Corp Design System              [Default]  ││    │
│  │  │    Based on: shadcn/ui                             ││    │
│  │  │    Theme: Custom (acme-brand)                      ││    │
│  │  │    Components: 45 customized                       ││    │
│  │  │    Last updated: 2 days ago                        ││    │
│  │  │    [Edit] [Duplicate] [Export]                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ 🌙 Dark Admin Theme                                ││    │
│  │  │    Based on: shadcn/ui                             ││    │
│  │  │    Theme: Dark mode focus                          ││    │
│  │  │    Components: 32 customized                       ││    │
│  │  │    Last updated: 1 week ago                        ││    │
│  │  │    [Edit] [Duplicate] [Export]                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  [+ Create New Design System]                           │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Design System Editor                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Acme Corp Design System                                │    │
│  │  ┌──────────┬──────────┬──────────┬──────────┐         │    │
│  │  │ Theme    │Components│ Layouts  │ Export   │         │    │
│  │  └──────────┴──────────┴──────────┴──────────┘         │    │
│  │                                                          │    │
│  │  Theme Configuration                                     │    │
│  │  ───────────────────                                     │    │
│  │                                                          │    │
│  │  Colors                                                  │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ Primary:    [#3B82F6] ████                         ││    │
│  │  │ Secondary:  [#10B981] ████                         ││    │
│  │  │ Accent:     [#F59E0B] ████                         ││    │
│  │  │ Background: [#FFFFFF] ████                         ││    │
│  │  │ Foreground: [#1F2937] ████                         ││    │
│  │  │ Muted:      [#F3F4F6] ████                         ││    │
│  │  │ Border:     [#E5E7EB] ████                         ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  Typography                                              │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ Font Family: [Inter ▼]                             ││    │
│  │  │ Heading Font: [Cal Sans ▼]                         ││    │
│  │  │ Mono Font: [JetBrains Mono ▼]                      ││    │
│  │  │                                                     ││    │
│  │  │ Scale:                                             ││    │
│  │  │ H1: 36px / 2.25rem    Body: 16px / 1rem           ││    │
│  │  │ H2: 30px / 1.875rem   Small: 14px / 0.875rem      ││    │
│  │  │ H3: 24px / 1.5rem     XS: 12px / 0.75rem          ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  Border Radius                                           │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │ None ○  SM ○  MD ●  LG ○  Full ○                   ││    │
│  │  │ Preview: [████]  radius: 0.5rem                    ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Component Library Management:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Component Browser                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  shadcn/ui Components                    [Search components...] │
│                                                                  │
│  Categories: [All] [Forms] [Data Display] [Layout] [Navigation] │
│                                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │   Button    │ │    Card     │ │   Dialog    │               │
│  │  ┌──────┐   │ │ ┌────────┐  │ │  ┌──────┐   │               │
│  │  │Click │   │ │ │ Title  │  │ │  │Modal │   │               │
│  │  └──────┘   │ │ │ Body   │  │ │  │      │   │               │
│  │             │ │ └────────┘  │ │  └──────┘   │               │
│  │ ✅ Installed │ │ ✅ Installed │ │ ✅ Installed │               │
│  │ [Customize] │ │ [Customize] │ │ [Customize] │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │ Data Table  │ │   Sidebar   │ │   Charts    │               │
│  │ ┌─────────┐ │ │ ┌──┬─────┐  │ │    📊       │               │
│  │ │ │ │ │ │ │ │ │ │  │     │  │ │   ╱╲       │               │
│  │ │ │ │ │ │ │ │ │ │  │     │  │ │  ╱  ╲      │               │
│  │ └─────────┘ │ │ └──┴─────┘  │ │ ╱    ╲     │               │
│  │ ☐ Not added │ │ ✅ Installed │ │ ☐ Not added │               │
│  │ [Add]       │ │ [Customize] │ │ [Add]       │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Page Templates:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Page Templates                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Ready-to-use page layouts for common patterns                  │
│                                                                  │
│  ┌─────────────────────┐  ┌─────────────────────┐              │
│  │ ┌─────────────────┐ │  │ ┌───┬─────────────┐ │              │
│  │ │     Header      │ │  │ │   │             │ │              │
│  │ ├─────────────────┤ │  │ │ S │   Content   │ │              │
│  │ │                 │ │  │ │ i │             │ │              │
│  │ │   Dashboard     │ │  │ │ d │             │ │              │
│  │ │   Grid Layout   │ │  │ │ e │             │ │              │
│  │ │                 │ │  │ │   │             │ │              │
│  │ └─────────────────┘ │  │ └───┴─────────────┘ │              │
│  │                      │  │                      │              │
│  │ Dashboard Template   │  │ Sidebar Layout       │              │
│  │ [Use Template]       │  │ [Use Template]       │              │
│  └─────────────────────┘  └─────────────────────┘              │
│                                                                  │
│  ┌─────────────────────┐  ┌─────────────────────┐              │
│  │ ┌─────────────────┐ │  │ ┌─────────────────┐ │              │
│  │ │ Login / Sign Up │ │  │ │    Settings     │ │              │
│  │ │                 │ │  │ │ ┌────┬────────┐ │ │              │
│  │ │  [Email     ]   │ │  │ │ │Tab │Content │ │ │              │
│  │ │  [Password  ]   │ │  │ │ │Tab │        │ │ │              │
│  │ │  [  Login   ]   │ │  │ │ └────┴────────┘ │ │              │
│  │ └─────────────────┘ │  │ └─────────────────┘ │              │
│  │                      │  │                      │              │
│  │ Auth Pages           │  │ Settings Page        │              │
│  │ [Use Template]       │  │ [Use Template]       │              │
│  └─────────────────────┘  └─────────────────────┘              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Integration with Agents (Blaze):**

```
┌─────────────────────────────────────────────────────────────────┐
│  Design System → Agent Integration                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  When Blaze (frontend agent) generates UI code:                 │
│                                                                  │
│  1. Reads active design system configuration                    │
│  2. Uses project's installed shadcn components                  │
│  3. Applies correct theme variables                             │
│  4. Follows typography and spacing rules                        │
│  5. Generates consistent, on-brand UI                           │
│                                                                  │
│  Agent Prompt Injection:                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ [Automatically added to Blaze's context]                │    │
│  │                                                          │    │
│  │ ## Design System: Acme Corp                             │    │
│  │                                                          │    │
│  │ Use these shadcn/ui components:                         │    │
│  │ - Button, Card, Dialog, Input, Select...                │    │
│  │                                                          │    │
│  │ Theme colors:                                            │    │
│  │ - Primary: hsl(221.2 83.2% 53.3%)                       │    │
│  │ - Use CSS variables: --primary, --secondary, etc.       │    │
│  │                                                          │    │
│  │ Typography:                                              │    │
│  │ - Font: Inter for body, Cal Sans for headings           │    │
│  │ - Use Tailwind classes: text-sm, text-base, etc.        │    │
│  │                                                          │    │
│  │ Always use existing components before creating new.     │    │
│  │ Follow the established patterns in /components/ui/      │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Design System Export:**

```yaml
# design-system.yaml (exportable)
name: Acme Corp Design System
version: 1.0.0
base: shadcn/ui

theme:
  colors:
    primary: "221.2 83.2% 53.3%"
    secondary: "210 40% 96.1%"
    accent: "210 40% 96.1%"
    background: "0 0% 100%"
    foreground: "222.2 84% 4.9%"
    # ... all CSS variables
    
  typography:
    fontFamily:
      sans: ["Inter", "sans-serif"]
      heading: ["Cal Sans", "sans-serif"]
      mono: ["JetBrains Mono", "monospace"]
    scale:
      h1: "2.25rem"
      h2: "1.875rem"
      # ...
      
  borderRadius: "0.5rem"
  
components:
  installed:
    - button
    - card
    - dialog
    - input
    # ...
  customized:
    button:
      variants:
        primary:
          className: "bg-primary hover:bg-primary/90"
        # custom variants
        
templates:
  - dashboard
  - auth
  - settings
```

### Git Provider Flexibility (v1.2+)

Support for multiple Git providers beyond GitHub, especially for enterprises 
with self-hosted requirements.

```
┌─────────────────────────────────────────────────────────────────┐
│                  Git Provider Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Supported Providers                                             │
│  ───────────────────                                             │
│                                                                  │
│  v1.0 (MVP):                                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  🐙 GitHub.com                                          │    │
│  │     • Full integration                                  │    │
│  │     • PRs, Issues, Actions                              │    │
│  │     • OAuth authentication                              │    │
│  │     • Webhooks                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  v1.2 (Stretch):                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  🏢 GitHub Enterprise Server                            │    │
│  │     • Self-hosted GitHub                                │    │
│  │     • Same API, custom domain                           │    │
│  │     • Enterprise SSO integration                        │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  🦊 GitLab (Cloud & Self-Hosted)                        │    │
│  │     • gitlab.com                                        │    │
│  │     • GitLab CE/EE self-hosted                          │    │
│  │     • MRs, Issues, CI/CD                                │    │
│  │     • Different API structure                           │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  v1.3+ (Future):                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  🍵 Gitea / Forgejo                                     │    │
│  │     • Lightweight self-hosted                           │    │
│  │     • GitHub-compatible API                             │    │
│  │     • Popular for air-gapped                            │    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │  🪣 Bitbucket (Cloud & Server)                          │    │
│  │     • Atlassian ecosystem                               │    │
│  │     • Popular in enterprise                             │    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │  🌊 Azure DevOps                                        │    │
│  │     • Microsoft ecosystem                               │    │
│  │     • Azure Repos                                       │    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │  📦 Codeberg                                            │    │
│  │     • Non-profit, Forgejo-based                         │    │
│  │     • Open source focused                               │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Provider Abstraction Layer:**

```
┌─────────────────────────────────────────────────────────────────┐
│                  Git Provider Abstraction                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  CTO Platform Agents                                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Rex, Blaze, Cypher, etc.                               │    │
│  │                                                          │    │
│  │  Use unified Git interface:                             │    │
│  │  • create_pull_request()                                │    │
│  │  • get_file_contents()                                  │    │
│  │  • post_comment()                                       │    │
│  │  • create_branch()                                      │    │
│  │  • list_pull_requests()                                 │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Git Provider Interface                  │    │
│  │                                                          │    │
│  │  trait GitProvider {                                    │    │
│  │    fn create_pr(&self, ...) -> Result<PullRequest>;    │    │
│  │    fn get_file(&self, ...) -> Result<FileContent>;     │    │
│  │    fn post_comment(&self, ...) -> Result<Comment>;     │    │
│  │    fn get_diff(&self, ...) -> Result<Diff>;            │    │
│  │    fn merge_pr(&self, ...) -> Result<()>;              │    │
│  │  }                                                      │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│         ┌────────────────────┼────────────────────┐             │
│         │                    │                    │             │
│         ▼                    ▼                    ▼             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐       │
│  │   GitHub    │     │   GitLab    │     │   Gitea     │       │
│  │   Adapter   │     │   Adapter   │     │   Adapter   │       │
│  │             │     │             │     │             │       │
│  │ • REST API  │     │ • REST API  │     │ • REST API  │       │
│  │ • GraphQL   │     │ • GraphQL   │     │ • (GH compat)│      │
│  │ • Webhooks  │     │ • Webhooks  │     │ • Webhooks  │       │
│  └─────────────┘     └─────────────┘     └─────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Provider Configuration UI:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Git Provider Settings                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Select Provider:                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ ● GitHub.com                                            │    │
│  │ ○ GitHub Enterprise                                     │    │
│  │ ○ GitLab.com                                            │    │
│  │ ○ GitLab Self-Hosted                                    │    │
│  │ ○ Gitea / Forgejo                                       │    │
│  │ ○ Bitbucket                                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  GitHub.com Configuration:                                       │
│                                                                  │
│  Organization/User:                                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ acme-corp                                                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Authentication:                                                 │
│  ○ GitHub App (Recommended)                                     │
│    App ID: [____________]  Private Key: [Upload]               │
│                                                                  │
│  ○ Personal Access Token                                        │
│    Token: [••••••••••••••••••••] [Test Connection]             │
│                                                                  │
│  Repositories:                                                   │
│  ○ All repositories                                             │
│  ● Selected repositories                                        │
│    ┌─────────────────────────────────────────────────────┐     │
│    │ ☑ acme-corp/backend                                 │     │
│    │ ☑ acme-corp/frontend                                │     │
│    │ ☐ acme-corp/docs                                    │     │
│    │ ☐ acme-corp/infrastructure                          │     │
│    └─────────────────────────────────────────────────────┘     │
│                                                                  │
│                              [Save Configuration]               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Self-Hosted Provider Configuration:**

```
┌─────────────────────────────────────────────────────────────────┐
│  GitLab Self-Hosted Configuration                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Instance URL:                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ https://gitlab.acme.internal                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  API Version: v4 (auto-detected)                                │
│                                                                  │
│  Authentication:                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Personal Access Token                                    │    │
│  │ ┌─────────────────────────────────────────────────────┐ │    │
│  │ │ ••••••••••••••••••••••••••••••                      │ │    │
│  │ └─────────────────────────────────────────────────────┘ │    │
│  │                                                          │    │
│  │ Required scopes: api, read_repository, write_repository │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  TLS Certificate:                                                │
│  ○ Use system CA bundle                                         │
│  ● Custom CA certificate                                        │
│    [Upload CA cert]                                             │
│  ○ Skip TLS verification (⚠️ not recommended)                   │
│                                                                  │
│  Connection Status: ✅ Connected                                │
│  GitLab Version: 16.8.1-ee                                      │
│                                                                  │
│                    [Test Connection]  [Save]                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Feature Parity Matrix:**

| Feature | GitHub | GitHub Ent | GitLab | Gitea | Bitbucket |
|---------|--------|------------|--------|-------|-----------|
| Pull/Merge Requests | ✅ | ✅ | ✅ | ✅ | ✅ |
| Code Review Comments | ✅ | ✅ | ✅ | ✅ | ✅ |
| Branch Protection | ✅ | ✅ | ✅ | ✅ | ✅ |
| Webhooks | ✅ | ✅ | ✅ | ✅ | ✅ |
| Status Checks | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue Integration | ✅ | ✅ | ✅ | ✅ | ✅ |
| OAuth/SSO | ✅ | ✅ | ✅ | 🟡 | ✅ |
| GitHub Actions equiv | ✅ | ✅ | ✅ CI | ✅ | ✅ Pipes |
| GraphQL API | ✅ | ✅ | ✅ | ❌ | ❌ |

**Air-Gapped Git (Bundled Gitea):**

For fully air-gapped deployments, option to include bundled Gitea:

```
┌─────────────────────────────────────────────────────────────────┐
│  Air-Gapped Mode: Bundled Git Server                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ☑ Include bundled Gitea instance                               │
│                                                                  │
│  This will deploy a local Gitea server on your platform.       │
│  Useful for:                                                     │
│  • Air-gapped environments                                      │
│  • Development/testing without external dependencies            │
│  • Complete data sovereignty                                    │
│                                                                  │
│  Access: https://git.cto-platform.local                        │
│  Storage: Uses platform's MinIO for LFS                        │
│                                                                  │
│  Note: You can still connect external providers in addition    │
│  to the bundled Gitea instance.                                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Cross-Tier Compatibility

All tiers use the same CTO Platform Core components, ensuring:
- Consistent API and user experience across tiers
- Agents and workflows work identically
- Configuration is portable between tiers
- Upgrades preserve user data and settings

### Mobile App (v1.2+)

A React Native mobile app for monitoring and managing your CTO Platform remotely.

```
┌─────────────────────────────────────────────────────────────────┐
│                    CTO Mobile App Architecture                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Mobile App (React Native)             │    │
│  │                    iOS + Android                         │    │
│  │                                                          │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │                                                     ││    │
│  │  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐          ││    │
│  │  │  │ 🏠  │ │ 📊  │ │ 🤖  │ │ ⚠️  │ │ ⚙️  │          ││    │
│  │  │  │Home │ │Stats│ │Agent│ │Alert│ │Sett.│          ││    │
│  │  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘          ││    │
│  │  │                                                     ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │                                                          │    │
│  │  Features:                                               │    │
│  │  • Dashboard - Platform health at a glance              │    │
│  │  • Agents - View agent status, trigger runs             │    │
│  │  • Alerts - Push notifications for issues               │    │
│  │  • PRD Builder - Create/edit PRDs on the go            │    │
│  │  • Deployments - Monitor ArgoCD syncs                   │    │
│  │  • Logs - Quick log viewer                              │    │
│  │  • Remote access - Secure tunnel to platform            │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              │ Secure Connection                 │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Connection Methods                    │    │
│  │                                                          │    │
│  │  1. Direct (Local Network)                              │    │
│  │     Phone → https://cto.local:6443                      │    │
│  │     (When on same network as platform)                  │    │
│  │                                                          │    │
│  │  2. Cloudflare Tunnel (Remote)                          │    │
│  │     Phone → Cloudflare → Platform                       │    │
│  │     (Secure remote access from anywhere)                │    │
│  │                                                          │    │
│  │  3. VPN (Enterprise)                                    │    │
│  │     Phone → Corporate VPN → Platform                    │    │
│  │     (For enterprises with existing VPN)                 │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Mobile App Screens:**

```
┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
│ ≡  CTO Platform   │  │ ← Agent Status    │  │ ← Alerts          │
├───────────────────┤  ├───────────────────┤  ├───────────────────┤
│                   │  │                   │  │                   │
│ Platform Health   │  │ Rex               │  │ 🔴 Critical (1)   │
│ ████████████ 94%  │  │ ────────────────  │  │                   │
│                   │  │ Status: ✅ Idle   │  │ ┌───────────────┐ │
│ ┌───────────────┐ │  │ Last run: 2h ago  │  │ │ Disk Warning  │ │
│ │ 🤖 5 Agents   │ │  │ Model: Sonnet 4   │  │ │ nvme0n1 @ 87% │ │
│ │    Running    │ │  │                   │  │ │ 2 min ago     │ │
│ └───────────────┘ │  │ Recent Tasks:     │  │ └───────────────┘ │
│                   │  │ ┌───────────────┐ │  │                   │
│ ┌───────────────┐ │  │ │ PR #142       │ │  │ 🟡 Warning (3)   │
│ │ 📊 Resources  │ │  │ │ ✅ Completed  │ │  │                   │
│ │ CPU: 45%      │ │  │ └───────────────┘ │  │ ┌───────────────┐ │
│ │ RAM: 62%      │ │  │ ┌───────────────┐ │  │ │ Cert Expiring │ │
│ │ Disk: 34%     │ │  │ │ PR #141       │ │  │ │ in 7 days     │ │
│ └───────────────┘ │  │ │ ✅ Completed  │ │  │ └───────────────┘ │
│                   │  │ └───────────────┘ │  │                   │
│ ┌───────────────┐ │  │                   │  │ 🟢 Info (12)     │
│ │ ⚠️ 1 Alert    │ │  │ [Trigger Run]     │  │                   │
│ │    Active     │ │  │                   │  │                   │
│ └───────────────┘ │  │                   │  │                   │
│                   │  │                   │  │                   │
├───────────────────┤  └───────────────────┘  └───────────────────┘
│ 🏠  📊  🤖  ⚠️  ⚙️│
└───────────────────┘

Home Screen          Agent Detail          Alerts List
```

**Push Notifications:**

```
┌─────────────────────────────────────────┐
│ 🔴 CTO Platform Alert                   │
│                                         │
│ Critical: Database pod OOMKilled        │
│ Guardian is attempting auto-recovery... │
│                                         │
│              [View] [Dismiss]           │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│ ✅ CTO Platform                         │
│                                         │
│ PR #142 merged successfully             │
│ Rex completed review in 3m 24s          │
│                                         │
│              [View PR] [Dismiss]        │
└─────────────────────────────────────────┘
```

### PRD Builder (v1.3+)

An AI-assisted feature that takes a Product Requirements Document and orchestrates 
the full implementation through the agent system.

```
┌─────────────────────────────────────────────────────────────────┐
│                      PRD Builder Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  User Input                                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  PRD: User Authentication System                        │    │
│  │  ════════════════════════════════                       │    │
│  │                                                          │    │
│  │  ## Overview                                             │    │
│  │  Build a complete authentication system with:           │    │
│  │  - Email/password login                                 │    │
│  │  - OAuth (Google, GitHub)                               │    │
│  │  - Password reset flow                                  │    │
│  │  - Session management                                   │    │
│  │                                                          │    │
│  │  ## Requirements                                         │    │
│  │  - Use BetterAuth library                               │    │
│  │  - PostgreSQL for user storage                          │    │
│  │  - React frontend components                            │    │
│  │  - API rate limiting                                    │    │
│  │                                                          │    │
│  │  ## Acceptance Criteria                                  │    │
│  │  - Users can register and login                         │    │
│  │  - OAuth flow works for Google/GitHub                   │    │
│  │  - Password reset emails are sent                       │    │
│  │  - Sessions expire after 24 hours                       │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  PRD Analyzer (AI-Powered)                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  1. Parse PRD into structured requirements              │    │
│  │  2. Identify technical components needed                │    │
│  │  3. Break down into implementable tasks                 │    │
│  │  4. Determine task dependencies                         │    │
│  │  5. Assign appropriate agents to each task              │    │
│  │  6. Estimate effort and timeline                        │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Generated Implementation Plan                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  📋 Implementation Plan                                  │    │
│  │  ═══════════════════════                                │    │
│  │                                                          │    │
│  │  Phase 1: Database & Backend (Rex)                      │    │
│  │  ├─ Task 1.1: Create user schema migrations             │    │
│  │  ├─ Task 1.2: Set up BetterAuth configuration          │    │
│  │  ├─ Task 1.3: Implement auth API endpoints             │    │
│  │  └─ Task 1.4: Add rate limiting middleware             │    │
│  │                                                          │    │
│  │  Phase 2: Frontend Components (Blaze)                   │    │
│  │  ├─ Task 2.1: Create login/register forms              │    │
│  │  ├─ Task 2.2: Build OAuth button components            │    │
│  │  ├─ Task 2.3: Password reset flow UI                   │    │
│  │  └─ Task 2.4: Session management hooks                 │    │
│  │                                                          │    │
│  │  Phase 3: Integration & Testing (Rex + Blaze)          │    │
│  │  ├─ Task 3.1: Connect frontend to backend              │    │
│  │  ├─ Task 3.2: E2E tests for auth flows                 │    │
│  │  └─ Task 3.3: Security review                          │    │
│  │                                                          │    │
│  │  Estimated time: 4-6 hours                              │    │
│  │  PRs to be created: 3-5                                 │    │
│  │                                                          │    │
│  │  [Approve & Start] [Edit Plan] [Cancel]                 │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Orchestration Engine                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  Execution Flow:                                         │    │
│  │                                                          │    │
│  │  ┌──────────┐    ┌──────────┐    ┌──────────┐          │    │
│  │  │ Task 1.1 │───►│ Task 1.2 │───►│ Task 1.3 │          │    │
│  │  │   Rex    │    │   Rex    │    │   Rex    │          │    │
│  │  └──────────┘    └──────────┘    └────┬─────┘          │    │
│  │                                       │                 │    │
│  │       ┌───────────────────────────────┘                 │    │
│  │       │                                                 │    │
│  │       ▼                                                 │    │
│  │  ┌──────────┐    ┌──────────┐    ┌──────────┐          │    │
│  │  │ Task 2.1 │───►│ Task 2.2 │───►│ Task 2.3 │          │    │
│  │  │  Blaze   │    │  Blaze   │    │  Blaze   │          │    │
│  │  └──────────┘    └──────────┘    └────┬─────┘          │    │
│  │                                       │                 │    │
│  │       ┌───────────────────────────────┘                 │    │
│  │       ▼                                                 │    │
│  │  ┌──────────────────────────────────────────┐          │    │
│  │  │ Task 3.1: Integration (Rex + Blaze)      │          │    │
│  │  └──────────────────────────────────────────┘          │    │
│  │                                                          │    │
│  │  Progress: ████████████░░░░░░░░ 60%                     │    │
│  │  Current: Task 2.2 - OAuth button components            │    │
│  │  Agent: Blaze                                           │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  Output: Pull Requests                                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │  ✅ PR #143: feat(auth): Add user schema and migrations │    │
│  │     Merged 2 hours ago                                  │    │
│  │                                                          │    │
│  │  ✅ PR #144: feat(auth): BetterAuth configuration       │    │
│  │     Merged 1 hour ago                                   │    │
│  │                                                          │    │
│  │  🔄 PR #145: feat(auth): Login/register components      │    │
│  │     In review - waiting for approval                    │    │
│  │                                                          │    │
│  │  ⏳ PR #146: feat(auth): OAuth integration              │    │
│  │     Pending - blocked by #145                           │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**PRD Builder UI (Web + Mobile):**

```
┌─────────────────────────────────────────────────────────────────┐
│  CTO Platform > PRD Builder                                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ New PRD                                            [AI ✨]│    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │                                                          │    │
│  │ Title: User Authentication System                       │    │
│  │                                                          │    │
│  │ ┌─────────────────────────────────────────────────────┐ │    │
│  │ │ ## Overview                                         │ │    │
│  │ │ Build a complete authentication system with...      │ │    │
│  │ │                                                     │ │    │
│  │ │ ## Requirements                                     │ │    │
│  │ │ - Email/password login                              │ │    │
│  │ │ - OAuth (Google, GitHub)                            │ │    │
│  │ │ |                                                   │ │    │
│  │ └─────────────────────────────────────────────────────┘ │    │
│  │                                                          │    │
│  │ [📎 Attach files]  [🎨 Add mockups]  [📋 Use template]  │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  AI Suggestions:                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ 💡 Consider adding:                                      │    │
│  │    • Two-factor authentication (2FA)                    │    │
│  │    • Account lockout after failed attempts              │    │
│  │    • Audit logging for auth events                      │    │
│  │                                                          │    │
│  │    [Add to PRD]  [Dismiss]                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────┐  ┌─────────────────────────────┐     │
│  │  Save Draft          │  │  Analyze & Create Plan  ▶   │     │
│  └──────────────────────┘  └─────────────────────────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**PRD Templates:**

```
Available Templates:
├── Authentication System
├── Payment Integration  
├── Admin Dashboard
├── API Endpoint (CRUD)
├── Background Job Worker
├── Email Notification System
├── File Upload Service
├── Search Implementation
├── Reporting Dashboard
├── Mobile App Feature
└── Custom (blank)

Each template includes:
• Pre-filled sections with common requirements
• Suggested acceptance criteria
• Estimated complexity
• Recommended agent assignments
```

**Mobile PRD Builder:**

```
┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
│ ← New PRD         │  │ ← Implementation  │  │ ← Progress        │
├───────────────────┤  ├───────────────────┤  ├───────────────────┤
│                   │  │                   │  │                   │
│ Title:            │  │ Auth System       │  │ Auth System       │
│ ┌───────────────┐ │  │ ════════════════  │  │ ════════════════  │
│ │ Auth System   │ │  │                   │  │                   │
│ └───────────────┘ │  │ Plan Generated ✅ │  │ ████████████░░░░  │
│                   │  │                   │  │ 75% Complete      │
│ Use Template:     │  │ 8 tasks           │  │                   │
│ ┌───────────────┐ │  │ 3 agents          │  │ Current Task:     │
│ │ Authentication│ │  │ ~4 hours          │  │ ┌───────────────┐ │
│ │     ▼         │ │  │                   │  │ │ OAuth buttons │ │
│ └───────────────┘ │  │ Tasks:            │  │ │ Blaze 🤖      │ │
│                   │  │ ✅ Schema         │  │ │ In progress   │ │
│ Description:      │  │ ✅ BetterAuth     │  │ └───────────────┘ │
│ ┌───────────────┐ │  │ ✅ API routes     │  │                   │
│ │ Build user    │ │  │ 🔄 Login forms    │  │ PRs:              │
│ │ auth with     │ │  │ ⏳ OAuth btns     │  │ ✅ #143 Merged    │
│ │ email login   │ │  │ ⏳ Reset flow     │  │ ✅ #144 Merged    │
│ │ and OAuth...  │ │  │ ⏳ Integration    │  │ 🔄 #145 Review    │
│ │               │ │  │ ⏳ Tests          │  │                   │
│ └───────────────┘ │  │                   │  │ [View Details]    │
│                   │  │ [Start] [Edit]    │  │                   │
│ [AI ✨ Enhance]   │  │                   │  │                   │
│                   │  │                   │  │                   │
│ [Save] [Analyze]  │  │                   │  │                   │
├───────────────────┤  └───────────────────┘  └───────────────────┘
│ 🏠  📋  🤖  ⚠️  ⚙️│
└───────────────────┘

Create PRD          Review Plan           Monitor Progress
(Voice input too!)
```

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2025-11-27 | 5D Labs | Initial architecture draft |
