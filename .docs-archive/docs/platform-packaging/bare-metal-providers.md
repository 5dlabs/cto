# CTO Platform: Bare Metal Provider Guide

## Overview

CTO Platform supports **two deployment models** - customers can choose based on their 
needs, expertise, and existing infrastructure:

```
┌─────────────────────────────────────────────────────────────────┐
│                 CTO Platform Deployment Options                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  OPTION A: Bare Metal Cloud Providers (Recommended for MVP)     │
│  ─────────────────────────────────────────────────────────────  │
│  Deploy on Hetzner, OVHcloud, Vultr, Latitude.sh, etc.         │
│                                                                  │
│  ✅ No upfront hardware investment                              │
│  ✅ Instant provisioning (minutes)                              │
│  ✅ Consistent, tested hardware specs                           │
│  ✅ API-driven automation                                       │
│  ✅ Hourly/monthly billing                                      │
│  ✅ Still 70-80% cheaper than AWS/GCP/Azure                     │
│                                                                  │
│  Best for: Most customers, quick start, predictable costs       │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  OPTION B: Self-Hosted (Own Hardware)                           │
│  ─────────────────────────────────────────────────────────────  │
│  Deploy on your own physical servers                            │
│                                                                  │
│  ✅ Full data sovereignty                                       │
│  ✅ Air-gapped deployments possible                             │
│  ✅ Use existing hardware investments                           │
│  ✅ Maximum cost savings long-term                              │
│  ✅ Complete control                                            │
│                                                                  │
│  Best for: Enterprises with existing infra, regulated           │
│  industries, air-gapped requirements, colocation customers      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Why Two Options?

| Factor | Bare Metal Cloud | Self-Hosted |
|--------|------------------|-------------|
| **Upfront Cost** | $0 | $2,500-100,000+ |
| **Time to Deploy** | 15 minutes | Days-weeks |
| **Hardware Expertise** | Not needed | Required |
| **Air-Gapped** | Not possible | ✅ Full support |
| **Data Location** | Provider DC | Your facility |
| **Long-term Cost** | Monthly recurring | Lower after payback |
| **Maintenance** | Provider handles | You handle |

**Our Recommendation:** Start with bare metal cloud for MVP/validation, 
then offer self-hosted for enterprises with specific requirements.

---

## Option A: Bare Metal Cloud Providers

---

## Recommended Providers (Tier 1)

### 🇩🇪 Hetzner (Germany)

**Why #1**: Best price-to-performance ratio, extremely popular with self-hosters, 
excellent Kubernetes community support, 100% green energy.

**Locations:** Germany (Falkenstein, Nuremberg), Finland (Helsinki)

| Plan | CPU | RAM | Storage | Network | Price/mo |
|------|-----|-----|---------|---------|----------|
| **AX42** | AMD Ryzen 7 PRO 8700GE (8c/16t) | 64 GB DDR5 ECC | 2x 512GB NVMe | 1 Gbit/s | **€46.52** (~$50) |
| **AX52** | AMD Ryzen 7 7700 (8c/16t) | 64 GB DDR5 | 2x 1TB NVMe | 1 Gbit/s | **€59.66** (~$65) |
| **AX102** | AMD Ryzen 9 7950X3D (16c/32t) | 128 GB DDR5 ECC | 2x 1.92TB NVMe | 1 Gbit/s | **€104.11** (~$115) |
| **AX162-R** | AMD EPYC 9454P (48c/96t) | 256 GB DDR5 ECC | 2x 3.84TB NVMe | 1 Gbit/s | **€199.21** (~$220) |

**CTO Platform Recommendation:**
- **Single Node (MVP)**: AX52 - €60/mo (~$65)
- **3-Node Cluster**: 3x AX52 - €180/mo (~$195)
- **Enterprise**: AX162-R - €200/mo (~$220)

**Additional Features:**
- Unlimited bandwidth (included)
- DDoS protection included
- vSwitch for VLAN networking
- Storage Box (backup) from €3.20/mo
- No setup fee

**API:** Full API for automated provisioning
**Talos Support:** Excellent - widely tested, community support

---

### 🇫🇷 OVHcloud (France)

**Why**: Major European provider, global presence, unmetered bandwidth, 
cost-effective at scale.

**Locations:** 40 data centers across Europe, Americas, Asia-Pacific

| Plan | CPU | RAM | Storage | Network | Price/mo |
|------|-----|-----|---------|---------|----------|
| **Rise-1** | Intel Xeon-E 2388G (8c/16t) | 32 GB DDR4 | 2x 512GB NVMe | 1 Gbit/s | **~$70** |
| **Rise-2** | Intel Xeon-E 2388G (8c/16t) | 64 GB DDR4 | 2x 960GB NVMe | 1 Gbit/s | **~$95** |
| **Advance-1** | AMD EPYC 4344P (8c/16t) | 64 GB DDR5 | 2x 960GB NVMe | 1 Gbit/s | **~$130** |
| **Scale-1** | AMD EPYC 9124 (16c/32t) | 128 GB DDR5 | 2x 960GB NVMe | 25 Gbit/s | **~$295** |

**CTO Platform Recommendation:**
- **Single Node**: Rise-2 - ~$95/mo
- **3-Node Cluster**: 3x Rise-1 - ~$210/mo

**Additional Features:**
- Unmetered bandwidth
- Anti-DDoS protection
- vRack (private networking)
- Global presence

**API:** OVH API for provisioning
**Talos Support:** Good - documented

---

### 🇺🇸 Vultr (US)

**Why**: US-based, simple pricing, good developer experience, 32 global locations.

**Locations:** 32 cities globally (Americas, Europe, Asia, Australia)

| Plan | CPU | RAM | Storage | Network | Price/mo |
|------|-----|-----|---------|---------|----------|
| **Intel E-2286G** | Intel E-2286G (6c/12t) | 32 GB | 2x 480GB SSD | 10 Gbit/s | **$120** |
| **Intel E-2288G** | Intel E-2288G (8c/16t) | 64 GB | 2x 960GB SSD | 10 Gbit/s | **$185** |
| **AMD EPYC 7443P** | AMD EPYC 7443P (24c/48t) | 128 GB | 2x 960GB NVMe | 10 Gbit/s | **$350** |
| **AMD EPYC 7443P** | AMD EPYC 7443P (24c/48t) | 256 GB | 2x 1.92TB NVMe | 10 Gbit/s | **$470** |

**CTO Platform Recommendation:**
- **Single Node**: E-2288G - $185/mo
- **3-Node Cluster**: 3x E-2286G - $360/mo

**Additional Features:**
- 10 Gbit/s network included
- Hourly billing available
- Excellent API
- GPU instances available

**API:** REST API, Terraform provider
**Talos Support:** Good

---

### 🇺🇸 Latitude.sh (US)

**Why**: Modern platform, ex-Packet/Equinix Metal team, API-first design, 
competitive pricing, good for Kubernetes.

**Locations:** US, Europe, South America, Asia

| Plan | CPU | RAM | Storage | Network | Price/mo |
|------|-----|-----|---------|---------|----------|
| **c2.medium.x86** | Intel Xeon E-2378G (8c/16t) | 32 GB | 960GB NVMe | 10 Gbit/s | **~$180** |
| **c3.medium.x86** | AMD EPYC 7313P (16c/32t) | 64 GB | 2x 960GB NVMe | 10 Gbit/s | **~$320** |
| **m3.large.x86** | AMD EPYC 7443P (24c/48t) | 256 GB | 2x 3.84TB NVMe | 25 Gbit/s | **~$650** |

**CTO Platform Recommendation:**
- **Single Node**: c2.medium.x86 - ~$180/mo
- **3-Node Cluster**: 3x c2.medium.x86 - ~$540/mo

**Additional Features:**
- Metal as a Service (MaaS) API
- VLAN support
- Terraform provider
- Great for K8s workloads

**API:** Excellent, modern REST API
**Talos Support:** Excellent - actively tested

---

## Other Notable Providers (Tier 2)

### 🇱🇹 Cherry Servers (Lithuania)

**Starting at:** $107/mo
**Best for:** Europe, customization
**Pros:** Fully customizable, DevOps integrations, 24/7 support
**Cons:** Smaller footprint

### 🇫🇷 Scaleway (France)

**Starting at:** €10/mo (Dedibox)
**Best for:** Budget European deployments
**Pros:** ARM-based servers, Apple Silicon for CI/CD
**Cons:** Limited US presence

### 🇺🇸 PhoenixNAP (US)

**Starting at:** $89/mo
**Best for:** US enterprises, disaster recovery
**Pros:** Multiple US locations, Kubernetes-focused
**Cons:** Higher pricing

### 🇺🇸 DigitalOcean (US)

**Starting at:** $150/mo
**Best for:** Developers familiar with DO ecosystem
**Pros:** Simple UI, good documentation
**Cons:** Limited bare metal options

### 🇺🇸 Linode/Akamai (US)

**Starting at:** $156/mo (Dedicated CPU)
**Best for:** Existing Linode customers
**Pros:** Global CDN integration
**Cons:** Technically "dedicated VMs" not bare metal

---

## Price Comparison Matrix

### Single Node (64GB RAM, NVMe, suitable for CTO Platform)

| Provider | Location | Monthly Cost | Hourly | Network |
|----------|----------|--------------|--------|---------|
| **Hetzner AX52** | 🇩🇪 Germany | **$65** | $0.09 | 1 Gbit |
| OVHcloud Rise-2 | 🇫🇷 France | $95 | N/A | 1 Gbit |
| Cherry Servers | 🇱🇹 Lithuania | $107 | Yes | 1 Gbit |
| Vultr | 🇺🇸 US | $185 | $0.25 | 10 Gbit |
| Latitude.sh | 🇺🇸 US | $180 | $0.25 | 10 Gbit |
| DigitalOcean | 🇺🇸 US | $150+ | Yes | 10 Gbit |
| PhoenixNAP | 🇺🇸 US | $89+ | Yes | 1 Gbit |

### 3-Node Cluster (HA Kubernetes)

| Provider | Config | Monthly Cost | vs AWS (3x m5.xlarge) |
|----------|--------|--------------|----------------------|
| **Hetzner** | 3x AX52 | **$195** | 85% savings |
| OVHcloud | 3x Rise-1 | $210 | 84% savings |
| Vultr | 3x E-2286G | $360 | 72% savings |
| Latitude | 3x c2.medium | $540 | 58% savings |
| AWS | 3x m5.xlarge | $1,296 | Baseline |

---

## CTO Platform Certified Configurations

### Starter Tier (Single Node)

**Hetzner AX52** - **€60/mo (~$65)**
```
CPU:      AMD Ryzen 7 7700 (8 cores / 16 threads)
RAM:      64 GB DDR5
Storage:  2x 1TB NVMe (RAID 1)
Network:  1 Gbit/s unmetered
Location: Germany (Falkenstein, Nuremberg) or Finland (Helsinki)
```

**Suitable for:**
- 5-15 developers
- Development/staging environments
- Small production workloads

### Team Tier (3-Node Cluster)

**3x Hetzner AX52** - **€180/mo (~$195)**
```
Per Node:
  CPU:      AMD Ryzen 7 7700 (8 cores / 16 threads)
  RAM:      64 GB DDR5
  Storage:  2x 1TB NVMe

Total Cluster:
  CPU:      24 cores / 48 threads
  RAM:      192 GB
  Storage:  6TB NVMe (replicated via Longhorn)
```

**Suitable for:**
- 15-50 developers
- Production workloads
- High availability requirements

### Business Tier (5-Node Cluster)

**5x Hetzner AX102** - **€520/mo (~$575)**
```
Per Node:
  CPU:      AMD Ryzen 9 7950X3D (16 cores / 32 threads)
  RAM:      128 GB DDR5 ECC
  Storage:  2x 1.92TB NVMe

Total Cluster:
  CPU:      80 cores / 160 threads
  RAM:      640 GB
  Storage:  19TB NVMe (replicated)
```

**Suitable for:**
- 50-150 developers
- Heavy workloads
- AI/ML training

### Enterprise Tier (10-Node Cluster)

**10x Hetzner AX162-R** - **€2,000/mo (~$2,200)**
```
Per Node:
  CPU:      AMD EPYC 9454P (48 cores / 96 threads)
  RAM:      256 GB DDR5 ECC
  Storage:  2x 3.84TB NVMe

Total Cluster:
  CPU:      480 cores / 960 threads
  RAM:      2.5 TB
  Storage:  76TB NVMe (replicated)
```

**Suitable for:**
- 150+ developers
- Enterprise production
- Multi-tenant platforms

---

## Cost Comparison: Bare Metal Cloud vs AWS

### Small Team (Single Node)

| Item | AWS (m5.2xlarge) | Hetzner AX52 | Savings |
|------|------------------|--------------|---------|
| Compute | $280/mo | $65/mo | 77% |
| Storage (500GB) | $50/mo | Included | 100% |
| Data Transfer (2TB) | $180/mo | Included | 100% |
| **Total** | **$510/mo** | **$65/mo** | **87%** |
| **Annual** | **$6,120** | **$780** | **$5,340** |

### Mid-Size Team (3-Node Cluster)

| Item | AWS (3x m5.2xlarge) | Hetzner 3x AX52 | Savings |
|------|---------------------|-----------------|---------|
| Compute | $840/mo | $195/mo | 77% |
| Storage (2TB) | $200/mo | Included | 100% |
| Data Transfer (5TB) | $450/mo | Included | 100% |
| Load Balancer | $25/mo | Included* | 100% |
| **Total** | **$1,515/mo** | **$195/mo** | **87%** |
| **Annual** | **$18,180** | **$2,340** | **$15,840** |

*MetalLB included with CTO Platform

### Enterprise (10-Node Cluster)

| Item | AWS (10x m5.4xlarge) | Hetzner 10x AX162-R | Savings |
|------|----------------------|---------------------|---------|
| Compute | $5,600/mo | $2,200/mo | 61% |
| Storage (20TB) | $2,000/mo | Included | 100% |
| Data Transfer (20TB) | $1,800/mo | Included | 100% |
| **Total** | **$9,400/mo** | **$2,200/mo** | **77%** |
| **Annual** | **$112,800** | **$26,400** | **$86,400** |

---

## Deployment Automation

### Hetzner Robot API

```bash
# Example: Automated server ordering via API
curl -X POST "https://robot-ws.your-server.de/server/order" \
  -u "$HETZNER_USER:$HETZNER_PASS" \
  -d "product_id=AX52" \
  -d "datacenter=FSN1" \
  -d "authorized_key=$SSH_KEY"
```

### Terraform Provider (Hetzner)

```hcl
# main.tf - Provision CTO Platform nodes
provider "hcloud" {
  token = var.hetzner_token
}

resource "hcloud_server" "cto_node" {
  count       = 3
  name        = "cto-node-${count.index + 1}"
  server_type = "ax52"
  image       = "talos"  # Custom Talos image
  location    = "fsn1"
  
  ssh_keys = [hcloud_ssh_key.default.id]
}

resource "hcloud_network" "cto_network" {
  name     = "cto-platform"
  ip_range = "10.0.0.0/8"
}
```

### Latitude.sh Terraform

```hcl
provider "latitude" {
  auth_token = var.latitude_token
}

resource "latitude_server" "cto_node" {
  count            = 3
  hostname         = "cto-node-${count.index + 1}"
  plan             = "c2.medium.x86"
  operating_system = "talos_v1.8"
  site             = "NY1"
}
```

---

## Network Architecture

### Single Provider Setup (Hetzner)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hetzner Data Center (FSN1)                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Internet                                                        │
│      │                                                           │
│      ▼                                                           │
│  ┌─────────┐                                                     │
│  │ Floating │  Failover IP for HA                               │
│  │   IP     │                                                    │
│  └────┬────┘                                                     │
│       │                                                          │
│  ┌────┴────────────────────────────────────────────────────┐    │
│  │                    vSwitch (VLAN)                        │    │
│  └──┬──────────────┬──────────────┬──────────────┬─────────┘    │
│     │              │              │              │               │
│  ┌──┴──┐        ┌──┴──┐        ┌──┴──┐        ┌──┴──┐          │
│  │Node1│        │Node2│        │Node3│        │Node4│          │
│  │AX52 │        │AX52 │        │AX52 │        │AX52 │          │
│  │64GB │        │64GB │        │64GB │        │64GB │          │
│  └─────┘        └─────┘        └─────┘        └─────┘          │
│                                                                  │
│  Private Network: 10.0.0.0/24                                   │
│  Pod Network: 10.244.0.0/16 (Cilium)                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Multi-Region Setup

```
┌─────────────────────────────────────────────────────────────────┐
│                     Multi-Region Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│           Cloudflare / Global DNS                               │
│                      │                                           │
│        ┌─────────────┼─────────────┐                            │
│        │             │             │                            │
│        ▼             ▼             ▼                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │ Hetzner  │  │  OVH     │  │ Latitude │                      │
│  │ (EU)     │  │  (EU)    │  │  (US)    │                      │
│  │ FSN1     │  │  GRA     │  │  NY1     │                      │
│  │          │  │          │  │          │                      │
│  │ 3 Nodes  │  │ 3 Nodes  │  │ 3 Nodes  │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
│       │             │             │                             │
│       └─────────────┴─────────────┘                             │
│                     │                                           │
│              WireGuard Mesh                                     │
│          (Cross-DC connectivity)                                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## GPU Options (AI/ML Workloads)

### Hetzner (Limited GPU)

Currently no dedicated GPU servers, but:
- Can use Hetzner Cloud GPU (GTX 1080 instances)
- Better to use Vultr or Latitude for GPU

### Vultr GPU

| GPU | vCPU | RAM | Storage | Price/hr |
|-----|------|-----|---------|----------|
| NVIDIA A40 | 12 | 120 GB | 1.4TB NVMe | $1.99 |
| NVIDIA A100 (40GB) | 24 | 240 GB | 2.8TB NVMe | $3.50 |
| NVIDIA H100 (80GB) | 26 | 240 GB | 2.8TB NVMe | $5.50 |

### Lambda Labs (GPU Specialist)

| GPU | vCPU | RAM | Storage | Price/hr |
|-----|------|-----|---------|----------|
| NVIDIA A10 | 30 | 200 GB | 1.4TB NVMe | $0.75 |
| NVIDIA A100 (40GB) | 30 | 200 GB | 1.4TB NVMe | $1.29 |
| NVIDIA H100 (80GB) | 26 | 200 GB | 3TB NVMe | $2.49 |

---

## Provider Selection Guide

### Choose Hetzner If:
- ✅ Budget is primary concern
- ✅ EU location is acceptable/preferred
- ✅ GDPR compliance needed
- ✅ Green energy matters
- ✅ Community/self-hoster ecosystem

### Choose OVHcloud If:
- ✅ Need global presence
- ✅ Unmetered bandwidth critical
- ✅ EU data sovereignty
- ✅ vRack networking needed

### Choose Vultr If:
- ✅ Need US-based provider
- ✅ Want GPU options
- ✅ Need many global locations
- ✅ Hourly billing important

### Choose Latitude.sh If:
- ✅ Modern API-first experience
- ✅ Coming from Equinix Metal
- ✅ Need premium US locations
- ✅ Advanced networking features

---

## Implementation Plan

### Phase 1: MVP (Hetzner-first)

1. **Create Hetzner account automation**
   - Document signup process
   - API key generation
   - SSH key management

2. **Build Talos image for Hetzner**
   - Custom Talos image with Hetzner drivers
   - Upload to Hetzner Image service
   - Test automated installation

3. **Terraform module for single-node**
   - One-click deployment
   - Network configuration
   - DNS setup

### Phase 2: Multi-Provider Support

1. **Add OVHcloud support**
2. **Add Vultr support**
3. **Add Latitude.sh support**
4. **Create provider-agnostic abstraction layer**

### Phase 3: Advanced Features

1. **Multi-region deployments**
2. **Cross-provider networking (WireGuard mesh)**
3. **GPU node support**
4. **Automated failover between providers**

---

## Customer Quick Start

### Step 1: Choose Provider & Plan

```
┌─────────────────────────────────────────────────────────────────┐
│  CTO Platform - Choose Your Infrastructure                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Budget-Friendly (EU):                                          │
│  ● Hetzner AX52 - €60/mo ($65)                                 │
│    Best value, Germany/Finland                                  │
│                                                                  │
│  Global Presence:                                                │
│  ● OVHcloud Rise - $95/mo                                       │
│    40 data centers worldwide                                    │
│                                                                  │
│  US-Based:                                                       │
│  ● Vultr Bare Metal - $185/mo                                   │
│    32 US & global locations                                     │
│                                                                  │
│  Premium US:                                                     │
│  ● Latitude.sh - $180/mo                                        │
│    Modern platform, great API                                   │
│                                                                  │
│                              [Compare Providers →]              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Step 2: Automated Deployment

```bash
# Using CTO Platform CLI
cto-platform deploy \
  --provider hetzner \
  --plan ax52 \
  --location fsn1 \
  --nodes 1 \
  --license YOUR_LICENSE_KEY
```

### Step 3: Access Platform

```
Your CTO Platform is ready!

Dashboard: https://cto.your-domain.com
API: https://api.cto.your-domain.com

Time to deploy: ~15 minutes
Monthly cost: €60 ($65)
```

---

## Option B: Self-Hosted (Own Hardware)

For customers who prefer to run on their own infrastructure.

### Certified Hardware (Self-Hosted)

| Tier | Recommended Server | Specs | Approx. Cost |
|------|-------------------|-------|--------------|
| **Starter** | Dell PowerEdge R660 | 8c, 64GB, 1TB NVMe | $2,500 (refurb) |
| **Team** | 3x Dell PowerEdge R660 | 24c total, 192GB | $7,500 (refurb) |
| **Business** | Dell PowerEdge R760 | 32c, 256GB, 4TB NVMe | $8,000 |
| **Enterprise** | HPE ProLiant DL360 Gen11 | 48c, 512GB, 8TB NVMe | $15,000+ |

### Self-Hosted Requirements

**Minimum Hardware:**
```
CPU:      8+ cores (x86_64, ARM64 in v1.3+)
RAM:      64 GB minimum
Storage:  500GB NVMe minimum (1TB+ recommended)
Network:  1 Gbit/s minimum (10GbE for multi-node)
```

**Network Requirements:**
```
- Static IP or DHCP reservation
- Outbound internet (for updates, unless air-gapped)
- Port 6443 (K8s API)
- Port 443/80 (HTTP/S)
```

### Self-Hosted Installation Methods

```
┌─────────────────────────────────────────────────────────────────┐
│              Self-Hosted Installation Options                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. USB Boot (Recommended)                                      │
│     Download ISO → Flash to USB → Boot server → Follow wizard   │
│                                                                  │
│  2. PXE Boot                                                    │
│     Set up PXE server → Network boot → Automated install        │
│     (Good for provisioning multiple servers)                    │
│                                                                  │
│  3. BMC/IPMI Remote                                             │
│     Mount ISO via iDRAC/iLO → Virtual media boot               │
│     (No physical access needed)                                 │
│                                                                  │
│  4. Air-Gapped ISO                                              │
│     Download air-gapped ISO (~30GB) → Fully offline install    │
│     (For disconnected environments)                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Self-Hosted vs Bare Metal Cloud: Cost Analysis

**5-Year TCO Comparison (3-Node Cluster):**

| Year | Bare Metal Cloud | Self-Hosted | Notes |
|------|------------------|-------------|-------|
| Year 0 | $0 | $7,500 | Hardware purchase |
| Year 1 | $2,340 | $600 | Cloud: $195/mo, Self: power/colo |
| Year 2 | $2,340 | $600 | |
| Year 3 | $2,340 | $600 | |
| Year 4 | $2,340 | $3,600 | Self: hardware refresh |
| Year 5 | $2,340 | $600 | |
| **Total** | **$11,700** | **$13,500** | Cloud wins slightly |

**Key Insight:** Bare metal cloud is actually cost-competitive with self-hosted 
when you factor in hardware depreciation, maintenance, and power costs.

**Self-hosted makes sense when:**
- You already own the hardware
- Air-gapped requirement
- Regulatory/data sovereignty requirements
- Colocation with fixed costs
- Very long deployment horizon (7+ years)

---

## Hybrid Deployments

Customers can mix both options:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hybrid Deployment Example                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Production (Self-Hosted)              Dev/Staging (Cloud)      │
│  ┌─────────────────────┐              ┌─────────────────────┐  │
│  │ Customer Data Center│              │ Hetzner Cloud       │  │
│  │                     │              │                     │  │
│  │ 5x Dell PowerEdge   │◄─────────────│ 2x AX52             │  │
│  │ Air-gapped          │  Sync        │ Internet-connected  │  │
│  │ Sensitive data      │              │ Testing             │  │
│  └─────────────────────┘              └─────────────────────┘  │
│                                                                  │
│  Benefits:                                                       │
│  • Production stays on-prem for compliance                      │
│  • Dev/staging in cloud for agility                             │
│  • Same CTO Platform, same experience                           │
│  • Lower cloud costs (only dev workloads)                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Summary: Which Option to Choose?

| Scenario | Recommendation |
|----------|----------------|
| **Startup / Small team** | Bare Metal Cloud (Hetzner) |
| **Testing / POC** | Bare Metal Cloud |
| **Enterprise pilot** | Bare Metal Cloud |
| **Enterprise production** | Either (based on requirements) |
| **Regulated industry** | Self-Hosted or Private Cloud |
| **Air-gapped / Disconnected** | Self-Hosted |
| **Existing hardware** | Self-Hosted |
| **Global presence needed** | Bare Metal Cloud (multi-region) |

---

*Last updated: November 2024*
*For CTO Platform deployment planning*

