# Bare metal providers for Talos Linux Kubernetes deployments

**Hivelocity and Zenlayer emerge as top choices** for vendor-agnostic Kubernetes platforms requiring PXE or custom ISO boot—both offer native iPXE API support with official Terraform providers and global datacenter presence. Voltage Park delivers the best GPU value at **$1.99/hour per H100** with confirmed PXE capability. The bare metal market is undergoing significant transformation: Equinix Metal's announced sunset by 2026 creates opportunity, while Metal3 achieved CNCF Incubating status in August 2025, establishing Cluster API as the emerging standard for multi-provider orchestration.

---

## Providers with native iPXE API support offer the smoothest Talos integration

The critical differentiator for Talos Linux deployment is whether providers support **iPXE via their provisioning API** versus requiring manual IPMI console access for custom ISO mounting. Native iPXE eliminates manual intervention entirely.

**Hivelocity** (Tampa, Florida) stands out with the most developer-friendly implementation. Their API accepts custom iPXE scripts at provisioning time—either as a hosted URL or inline content. The official Terraform provider (`hivelocity/hivelocity`) supports device provisioning, SSH key management, and spec-based product filtering. Pricing starts around **$62/month** for instant deployment servers with hourly billing available. Datacenter coverage spans **36+ locations** including Seoul, with GPU servers available on request. Notably, Hivelocity acquired Heficed in 2022, adding Johannesburg, Lagos, and São Paulo to their footprint.

**Zenlayer** (Los Angeles) matches this with native iPXE support plus exceptional emerging market coverage—**50+ locations across 6 continents** including strong presence in China, India, Brazil, Indonesia, and Southeast Asia. Their API documentation at `docs.console.zenlayer.com/api-reference/bare-metal-cloud/bmc` covers instance creation, reinstallation, and VPC management. The iPXE workflow supports both hosted scripts and netboot.xyz integration. IPMI console access is available for monitoring boot processes. Pricing is subscription-based with VIP hourly billing options.

**Servers.com** (Luxembourg) offers Dell PowerEdge servers with iDRAC, enabling custom ISO mounting via virtual media over CIFS/HTTP/HTTPS. Their Terraform provider integrates L2 segments, out-of-band management, and rescue mode. While not true iPXE-over-API, the combination of iDRAC access with API-controlled rescue mode provides a workable Talos deployment path for enterprise Dell hardware preferences.

---

## GPU-focused providers deliver H100 access with varying Talos compatibility

**Voltage Park** emerged as the standout for AI workloads requiring Talos compatibility. Running exclusively Dell PowerEdge XE9680 servers with **NVIDIA HGX H100 80GB SXM5** GPUs, they offer industry-leading pricing at **$1.99/hour per GPU**. Critically, the Imbue case study confirms successful MAAS/PXE-based Talos deployment on Voltage Park infrastructure. The REST API (`cloud-api.voltagepark.com`) supports deploy, manage, and cancel operations with Bearer token authentication. InfiniBand networking at **3,200 Gbps** enables GPU cluster interconnects. Deployment takes 15 minutes (30 with their managed Kubernetes add-on). However, no official Terraform provider exists yet.

**Gcore** (Luxembourg) provides the most complete automation story for GPU bare metal. Their HashiCorp-verified Terraform provider includes a `gcore_baremetal` resource with documented examples. Custom ISO support is explicitly confirmed—users can upload and deploy their own images. GPU options include A100, H100, H200, and upcoming GB200 configurations. Per-minute billing keeps costs predictable. The **15+ global datacenters** integrate with their CDN and DDoS protection products.

**OpenMetal** (USA) runs OpenStack Ironic underneath, providing native PXE and IPMI support through familiar APIs. Their **8x H100 configurations** target serious AI workloads with full BIOS and driver control. SGX/TDX-capable servers enable confidential computing use cases. Monthly billing only (no hourly) with 1+ year contract preferences. US and Netherlands datacenter locations.

**Crusoe Cloud** (Denver, founded 2018) offers the broadest GPU selection including H100, H200, GB200, L40S, A100, and AMD MI300X. Their sustainability focus—**90% lower carbon footprint** via stranded energy—appeals to enterprises with environmental commitments. Terraform and CLI support are documented, and their Managed Kubernetes (CMK) product simplifies deployment. PXE/custom ISO support requires direct verification; their managed provisioning approach may limit custom OS flexibility.

---

## Regional providers fill coverage gaps across APAC, Middle East, and Africa

| Provider | Regions | PXE/ISO | API Quality | GPU | K8s Readiness |
|----------|---------|---------|-------------|-----|---------------|
| **i3D.net** (Ubisoft) | APAC, ME, Africa, SA | FlexMetal API | Excellent | Yes | Proven at scale |
| **Zenlayer** | 50+ global | Native iPXE | Excellent | Yes | Documented Talos |
| **Hivelocity** | 36+ global | Native iPXE | Excellent | Yes | Terraform provider |
| **RedSwitches** | Singapore | IPMI/KVM | Basic | Yes | Good |
| **Dataplugs** | Japan, Hong Kong | IPMI/KVM | Basic | Limited | Manual |
| **Virtual Servers SA** | South Africa | IPMI | Basic | No | Manual |

**i3D.net** (Rotterdam, owned by Ubisoft) deserves special attention. Their FlexMetal API powers **200+ Kubernetes clusters** at Ubisoft with a **99.97% success rate at 12 deployments per second**—the strongest bare metal Kubernetes case study found. Datacenters span Tokyo, Singapore, Hong Kong, Sydney, Dubai, Fujairah, Johannesburg, and São Paulo. Talos OS native integration and Terraform support are documented. The 26+ Tbps private backbone enables multi-cloud integration with AWS, GCP, Azure, and AliCloud.

For **Africa-specific deployments**: Hivelocity (Johannesburg, Lagos), i3D.net (Johannesburg), and local providers like Virtual Servers SA (Cape Town, Johannesburg) and Rackzar offer IPMI-based custom OS capability. Virtual Servers SA provides full IPMI/KVM access from R1,499/month (~$80 USD) with unmetered traffic and NAPAfrica peering.

For **Middle East**: Zenlayer and i3D.net both serve Dubai. i3D.net additionally operates in Fujairah with direct Etisalat peering.

For **South America**: São Paulo coverage from Hivelocity, i3D.net, Zenlayer, and Melbicom (Lithuania-based but with IX.br integration).

---

## Industry standards converge on Cluster API for multi-provider orchestration

The bare metal provisioning landscape has consolidated around four key projects, with **Cluster API emerging as the integration layer** for multi-provider Kubernetes deployments.

**Metal3** achieved CNCF Incubating status in August 2025 with 57 contributing organizations including Fujitsu, IKEA, SUSE, Ericsson, and Red Hat. It wraps OpenStack Ironic in Kubernetes-native CRDs, representing physical servers as `BareMetalHost` resources. The Cluster API Provider Metal3 (CAPM3) enables declarative cluster provisioning. Talos Linux works through custom OS images. Best suited for owned/on-premises hardware rather than cloud provider integration.

**Tinkerbell** (CNCF Sandbox, open-sourced by Equinix Metal) powers thousands of daily provisions. Its workflow-driven architecture uses Smee (DHCP), Tootles (metadata), and Hook (LinuxKit installation environment). The CAPT provider enables Cluster API integration. Native Talos support exists—Talos config passes via the metadata service.

**MAAS** (Canonical) remains the most mature option at nearly 10 years old. It delivers the **fastest OS installation times in the industry** through optimized image-based provisioning. BMC support includes IPMI, AMT, and Redfish. Spectro Cloud's `cluster-api-provider-maas` provides full Cluster API capabilities. PhoenixNAP integrated MAAS for near-instant OS installation.

**Redfish** (DMTF standard, since 2014) has replaced IPMI as the industry-standard BMC protocol. All major vendors support it: Dell iDRAC, HPE iLO, Supermicro, Lenovo XCC. Metal3, Ironic, and Sidero Omni all use Redfish drivers. This standardization enables true vendor-agnostic provisioning.

---

## New market entrants and enterprise vendor offerings expand options

**Ubicloud** (Y Combinator W24, $16M seed round January 2024) represents a new category: open-source IaaS running on bare metal providers. Founded by the ex-Microsoft/Citus Data team, it delivers VMs **3x cheaper than AWS** by layering on top of Hetzner, Leaseweb, and AWS Bare Metal. GitHub Actions runners run **10x cheaper**. Both managed service and self-hosted options exist. Kubernetes support is planned. This model abstracts underlying bare metal providers while maintaining cost advantages.

**HPE GreenLake for Private Cloud Enterprise** (expanded 2023-2024) offers API-driven bare metal through `client.greenlake.hpe.com/api/metal` with an official Terraform provider (`hpegl_metal_host`). VMs, containers, and bare metal share a single compute/storage pool with pay-per-use pricing. Custom images are supported for OS installation. Requires HPE relationship but provides enterprise-grade support.

**Dell APEX Compute** (announced Dell World 2023) delivers bare metal servers as subscription service deployable in datacenters, edge locations, or Equinix colocation. CPU/GPU and OS/hypervisor choices are customer-specified. Monthly subscription pricing with APEX Console provisioning.

**Equinix Metal sunset announcement**: In late 2024, Equinix announced plans to sunset their Metal product by 2026. This creates significant market opportunity and migration pressure for current users. Tinkerbell remains available as the open-source provisioning foundation.

---

## Provider compatibility matrix for Talos Linux requirements

| Provider | iPXE/PXE API | Custom ISO | REST API | Terraform | GPU (H100/A100) | IPMI/KVM | Hourly Billing |
|----------|-------------|------------|----------|-----------|-----------------|----------|----------------|
| **Hivelocity** | ✅ Native | ✅ | ✅ Excellent | ✅ Official | ✅ | ✅ | ✅ |
| **Zenlayer** | ✅ Native | ✅ | ✅ Excellent | ✅ | ⚠️ Limited | ✅ | VIP only |
| **Voltage Park** | ✅ MAAS | ✅ | ✅ | ❌ | ✅ H100 | SSH | ✅ |
| **Gcore** | ❌ | ✅ Explicit | ✅ | ✅ Verified | ✅ H100/H200 | VNC | ✅ Per-minute |
| **OpenMetal** | ✅ Ironic | ✅ | ✅ OpenStack | ✅ Via OS | ✅ H100/A100 | ✅ | ❌ Monthly |
| **i3D.net** | ✅ FlexMetal | ✅ | ✅ | ✅ | ✅ | ✅ | Contact |
| **Servers.com** | ❌ | ✅ iDRAC | ✅ | ✅ Official | Contact | ✅ | ✅ |
| **HOSTKEY** | ❌ | ✅ IPMI | Basic | ❌ | ✅ H100/A100 | ✅ | ✅ €1.53/hr |
| **Crusoe Cloud** | ⚠️ Verify | ⚠️ Verify | ✅ | ✅ | ✅ All types | Managed | ✅ |
| **GMI Cloud** | ⚠️ Verify | ⚠️ Verify | ✅ | ❌ | ✅ H100/H200 | Managed | ✅ |

---

## Recommendations for vendor-agnostic Kubernetes platform architecture

**For full automation with Talos Linux**, prioritize providers with native iPXE API support:

1. **Hivelocity** as primary—official Terraform provider, native iPXE, 36+ locations, proven automation
2. **Zenlayer** for emerging markets—native iPXE, 50+ locations, strong APAC/LATAM/Africa presence  
3. **i3D.net** for proven Kubernetes scale—200+ cluster deployments, FlexMetal API, global coverage

**For GPU-intensive AI/ML workloads**:

1. **Voltage Park** for H100 value ($1.99/hr) with confirmed PXE capability via MAAS
2. **Gcore** for Terraform automation + H100/H200/GB200 availability
3. **OpenMetal** for OpenStack-native environments with H100 clusters

**For multi-provider abstraction**, implement a Cluster API-based control plane:
- Use **Metal3** or **MAAS** with respective Cluster API providers
- Deploy **Sidero Omni** ($10-50/node/month) for unified multi-cluster management across providers
- Leverage **Tinkerbell** patterns for custom provisioning workflows

**Providers requiring further validation** before implementation: Crusoe Cloud and GMI Cloud offer compelling GPU options but need direct confirmation of custom OS boot capability. Their managed provisioning approach may limit Talos deployment flexibility.

The combination of Hivelocity + Zenlayer + Voltage Park would provide global coverage, iPXE automation, and competitive GPU pricing while maintaining full Talos Linux compatibility through native API-based provisioning.