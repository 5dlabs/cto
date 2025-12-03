# Bare Metal Provider Research

> Provider research with iPXE/PXE support, APIs, pricing, and global coverage for Talos Linux deployment.

## Critical Requirements

For CTO Platform bare metal automation, providers must support:

| Requirement | Priority | Notes |
|-------------|----------|-------|
| PXE boot OR Custom ISO | **Non-negotiable** | Required for Talos Linux deployment |
| REST API | **Non-negotiable** | Programmatic provisioning |
| Server actions (reboot, power) | High | Lifecycle management |
| IPMI/KVM access | High | Boot monitoring, troubleshooting |
| Hourly billing | Medium | Cost optimization for variable workloads |
| Terraform provider | Medium | Infrastructure as Code integration |

---

## Existing Provider Coverage

Already documented/implemented:
- **Latitude.sh** (implemented)
- Hetzner
- OVHcloud
- Vultr
- Cherry Servers
- Scaleway
- PhoenixNAP
- DigitalOcean
- Linode/Akamai
- Equinix Metal (sunsetting June 30, 2026)

---

## New Provider Research

### Tier 1: Primary Automation Targets

#### Hivelocity (Tampa, FL)

**Why it matters:** Native iPXE API support with official Terraform provider - ideal for automation.

| Attribute | Details |
|-----------|---------|
| **iPXE Support** | Native API support (inline scripts or hosted URLs) |
| **Locations** | 36+ global including Seoul, Johannesburg, Lagos, São Paulo |
| **Terraform** | Official provider: `hivelocity/hivelocity` |
| **Pricing** | ~$62/month instant deployment, hourly billing available |
| **API Docs** | developers.hivelocity.net/docs/custom-ipxe |
| **Notes** | Acquired Heficed (2022) for expanded footprint |

**Recommendation:** Primary automation target for broad geographic coverage.

---

#### Zenlayer (Los Angeles)

**Why it matters:** Strongest emerging markets presence with native iPXE.

| Attribute | Details |
|-----------|---------|
| **iPXE Support** | Native iPXE + netboot.xyz integration |
| **Locations** | 50+ across 6 continents (strong China, India, Brazil, Indonesia, SEA) |
| **API** | docs.console.zenlayer.com/api-reference/bare-metal-cloud/bmc |
| **IPMI** | Console access for boot monitoring |
| **Billing** | Subscription-based with VIP hourly billing |

**Recommendation:** Essential for APAC, LATAM, and emerging market coverage.

---

#### i3D.net (Rotterdam, Netherlands)

**Why it matters:** Proven K8s scale with documented Talos OS integration.

| Attribute | Details |
|-----------|---------|
| **Parent** | Ubisoft-owned |
| **Scale** | FlexMetal API powers 200+ K8s clusters at Ubisoft |
| **Reliability** | 99.97% success rate at 12 deployments/second |
| **Locations** | Tokyo, Singapore, Hong Kong, Sydney, Dubai, Fujairah, Johannesburg, São Paulo |
| **Talos** | Documented Talos OS integration |
| **Network** | 26+ Tbps private backbone with AWS/GCP/Azure/AliCloud integration |

**Recommendation:** Proven K8s scale - ideal for enterprise deployments.

---

### Tier 2: GPU/AI Workload Specialists

#### Voltage Park (GPU-focused)

**Why it matters:** Industry-leading H100 pricing for AI workloads.

| Attribute | Details |
|-----------|---------|
| **Hardware** | Dell PowerEdge XE9680 with NVIDIA HGX H100 80GB SXM5 |
| **GPU Pricing** | **$1.99/hour per H100** (industry-leading) |
| **PXE Support** | Confirmed PXE/MAAS deployment (Imbue case study) |
| **Network** | InfiniBand: 3,200 Gbps for GPU clusters |
| **API** | cloud-api.voltagepark.com |
| **Deploy Time** | 15 minutes (30 with managed K8s) |
| **Terraform** | No official provider yet |

**Recommendation:** Primary target for AI/ML workloads requiring GPU clusters.

---

#### Gcore (Luxembourg)

**Why it matters:** HashiCorp-verified Terraform with explicit custom ISO support.

| Attribute | Details |
|-----------|---------|
| **Terraform** | HashiCorp-verified provider with `gcore_baremetal` resource |
| **ISO Support** | Explicit custom ISO support |
| **GPUs** | A100, H100, H200, GB200 (upcoming) |
| **Billing** | Per-minute billing |
| **Locations** | 15+ global datacenters |
| **Extras** | CDN/DDoS integration |

**Recommendation:** Strong choice for IaC-first deployments with GPU needs.

---

#### OpenMetal (USA)

**Why it matters:** OpenStack Ironic underneath means native PXE/IPMI support.

| Attribute | Details |
|-----------|---------|
| **Foundation** | OpenStack Ironic (native PXE/IPMI) |
| **GPU Configs** | 8x H100 configurations |
| **Security** | SGX/TDX-capable for confidential computing |
| **Billing** | Monthly only, 1+ year contracts preferred |
| **Locations** | US and Netherlands |

**Recommendation:** Enterprise-grade for serious AI workloads with compliance requirements.

---

### Regional Coverage Gap Analysis

| Region | Existing Coverage | New Providers |
|--------|-------------------|---------------|
| **Africa** | Limited | Hivelocity (Johannesburg, Lagos), i3D.net (Johannesburg) |
| **Middle East** | Limited | Zenlayer, i3D.net (Dubai, Fujairah) |
| **South America** | Limited | Hivelocity, i3D.net, Zenlayer (São Paulo) |
| **Southeast Asia** | Moderate | Zenlayer (Indonesia, SEA), i3D.net (Singapore, Hong Kong) |
| **China** | Limited | Zenlayer (strong presence) |

---

## Provider Compatibility Matrix

| Provider | iPXE/PXE API | Custom ISO | REST API | Terraform | GPU | IPMI/KVM | Hourly |
|----------|--------------|------------|----------|-----------|-----|----------|--------|
| **Hivelocity** | ✅ Native | ✅ | ✅ | ✅ Official | ❌ | ✅ | ✅ |
| **Zenlayer** | ✅ Native | ✅ | ✅ | ❌ | ✅ | ✅ | VIP only |
| **i3D.net** | ✅ FlexMetal | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ |
| **Voltage Park** | ✅ PXE/MAAS | ⚠️ | ✅ | ❌ | ✅ H100 | ✅ | ✅ |
| **Gcore** | ⚠️ | ✅ Explicit | ✅ | ✅ Verified | ✅ H100/H200 | ✅ | ✅ Per-min |
| **OpenMetal** | ✅ Ironic | ✅ | ✅ | ❌ | ✅ 8xH100 | ✅ | ❌ Monthly |

---

## Industry Standards & Orchestration

### Metal3 (CNCF Incubating)
- **Status:** CNCF Incubating (August 2025)
- **Community:** 57 contributing organizations
- **Use Case:** Kubernetes-native bare metal provisioning

### Tinkerbell (CNCF Sandbox)
- **Status:** CNCF Sandbox
- **Scale:** Powers thousands of daily provisions
- **Use Case:** Workflow-based bare metal provisioning

### MAAS (Canonical)
- **Maturity:** 10 years
- **Claim:** Fastest OS installation
- **Use Case:** Data center automation

### Redfish (DMTF Standard)
- **Status:** Replacing IPMI
- **Support:** Dell iDRAC, HPE iLO, Supermicro, Lenovo XCC
- **Use Case:** Modern server management API

### Cluster API
- **Status:** Emerging standard
- **Use Case:** Multi-provider Kubernetes orchestration

---

## Pricing Comparison

### General Compute

| Provider | Entry Config | Monthly | Hourly |
|----------|--------------|---------|--------|
| Hivelocity | Basic | ~$62 | Available |
| Latitude.sh | c2.small | $85 | $0.14 |
| Hetzner | AX41 | €39 | ❌ |
| OVHcloud | Rise-1 | €55 | ❌ |

### GPU Compute (H100)

| Provider | Config | Hourly | Monthly Equivalent |
|----------|--------|--------|-------------------|
| **Voltage Park** | H100 SXM5 | **$1.99** | ~$1,433 |
| Gcore | H100 | ~$3.50 | ~$2,520 |
| Lambda Labs | H100 | ~$1.89 | ~$1,361 |
| AWS | p5.48xlarge (8x H100) | $98.32 | ~$70,790 |

**Key Insight:** Bare metal H100 pricing is **~50x cheaper** than equivalent AWS instances.

---

## Implementation Priority

### Phase 1: Core Automation
1. **Hivelocity** - Official Terraform, native iPXE, 36+ locations
2. **Latitude.sh** - Already implemented, expand coverage

### Phase 2: Emerging Markets
3. **Zenlayer** - Native iPXE, 50+ locations, APAC/LATAM/Africa
4. **i3D.net** - 200+ cluster deployments, FlexMetal API

### Phase 3: GPU/AI Workloads
5. **Voltage Park** - $1.99/hr H100, proven PXE
6. **Gcore** - Terraform + H100/H200/GB200
7. **OpenMetal** - H100 clusters, confidential computing

---

## API Integration Notes

### Hivelocity iPXE Example

```bash
# Deploy with custom iPXE script
curl -X POST "https://core.hivelocity.net/api/v2/compute/servers" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "product_id": "123",
    "location_id": "TPA1",
    "ipxe_script_url": "https://boot.talos.dev/ipxe"
  }'
```

### Gcore Terraform Example

```hcl
resource "gcore_baremetal" "talos_node" {
  name       = "talos-worker-1"
  flavor_id  = "bm1-infrastructure-small"
  region_id  = 1
  
  image_id   = "custom-talos-image-id"
  
  interface {
    type       = "external"
    network_id = gcore_network.main.id
  }
}
```

### Voltage Park API

```bash
# Check available H100 capacity
curl -X GET "https://cloud-api.voltagepark.com/v1/instances/available" \
  -H "Authorization: Bearer $API_KEY"
```

---

## Risk Assessment

### Equinix Metal Discontinuation (June 30, 2026)

**Impact:** Major market disruption - many enterprises currently on Equinix Metal will need alternatives.

**Opportunity:** Position CTO Platform as migration destination with:
- Multi-provider abstraction
- Equivalent or better coverage
- Automated migration tooling

### Provider Stability Concerns

| Provider | Risk Level | Notes |
|----------|------------|-------|
| Hivelocity | Low | Established, profitable, growing |
| Zenlayer | Low | VC-backed, strong growth |
| i3D.net | Low | Ubisoft-owned, stable |
| Voltage Park | Medium | Newer entrant, GPU-focused |
| Gcore | Low | Established, CDN revenue base |
| OpenMetal | Medium | Smaller, niche focus |

---

## Recommendations Summary

1. **Primary automation:** Hivelocity (official Terraform, native iPXE, 36+ locations)
2. **Emerging markets:** Zenlayer (native iPXE, 50+ locations, APAC/LATAM/Africa)
3. **Proven K8s scale:** i3D.net (200+ cluster deployments, FlexMetal API)
4. **GPU/AI workloads:** Voltage Park ($1.99/hr H100), Gcore (Terraform + H100/H200/GB200), OpenMetal (H100 clusters)
