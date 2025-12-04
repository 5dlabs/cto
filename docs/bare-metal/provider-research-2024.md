# Bare Metal & Cloud Provider Research

> Comprehensive research for CTO Platform multi-provider support

## Executive Summary

This document evaluates providers for the CTO Platform's infrastructure provisioning capabilities. The goal is to support **5+ bare metal providers**, the **big 3 cloud providers**, and **at-home appliance** deployment.

### Key Requirements

For Talos Linux bootstrap, providers must support:

| Requirement | Priority | Notes |
|------------|----------|-------|
| **iPXE/Custom OS** | Critical | Required for Talos installation |
| **REST API** | Critical | Programmatic provisioning |
| **Server Actions** | High | Power on/off, reboot |
| **Reinstall API** | High | OS reinstallation with custom iPXE |
| **IPMI/KVM** | Medium | Out-of-band management |
| **GPU Support** | Medium | For AI/ML workloads |
| **Private Networks** | Medium | Cluster networking |

---

## Bare Metal Providers (5 Recommended)

### 1. Latitude.sh ✅ (Currently Implemented)

**Status**: Production-ready implementation in `crates/metal/src/providers/latitude/`

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐⭐ Excellent |
| **iPXE Support** | ✅ Full support via reinstall API |
| **GPU Availability** | ✅ H100, L40S |
| **Pricing** | $$ Mid-range ($180/mo 3-node) |
| **Regions** | 15+ global locations |
| **Documentation** | Excellent - REST API + Go SDK |

**API Endpoints**:
- `POST /servers` - Deploy server
- `POST /servers/{id}/reinstall` - iPXE reinstall
- `POST /servers/{id}/actions` - Power control
- `POST /servers/{id}/ipmi` - IPMI credentials

**Pros**:
- Best API design for automation
- Ex-Packet team (enterprise reliability)
- Hourly billing for POC-friendly testing
- Strong GPU availability

**Cons**:
- Higher pricing than budget providers
- Smaller footprint than hyperscalers

---

### 2. Vultr

**API Documentation**: https://www.vultr.com/api/#tag/baremetal

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐ Good |
| **iPXE Support** | ✅ Custom iPXE scripts supported |
| **GPU Availability** | ✅ H100, A100 |
| **Pricing** | $$ Mid-range ($120/mo 3-node) |
| **Regions** | 25+ global locations |
| **Documentation** | Good - REST API |

**Key API Endpoints**:
```
POST /v2/bare-metals          # Create bare metal instance
GET  /v2/bare-metals          # List instances
GET  /v2/bare-metals/{id}     # Get instance
DELETE /v2/bare-metals/{id}   # Delete instance
POST /v2/bare-metals/{id}/reinstall  # Reinstall OS
POST /v2/bare-metals/{id}/start      # Power on
POST /v2/bare-metals/{id}/halt       # Power off
POST /v2/bare-metals/{id}/reboot     # Reboot
```

**iPXE Support**:
```json
{
  "region": "ewr",
  "plan": "vbm-4c-32gb",
  "os_id": 159,  // Custom iPXE
  "script_id": "cb676a46-66fd-4dfb-b839-443f2e6c0b60",
  "ipxe_chain_url": "https://example.com/boot.ipxe"
}
```

**Pros**:
- Strong GPU availability (H100/A100)
- Good geographic coverage
- Competitive pricing
- Established provider

**Cons**:
- API slightly less polished than Latitude
- Some features require support tickets

**Implementation Priority**: HIGH - Good balance of features/price

---

### 3. Hetzner

**API Documentation**: https://docs.hetzner.cloud/ (Cloud) + https://robot.hetzner.com/doc/webservice/en.html (Dedicated)

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐ Good (Robot API) |
| **iPXE Support** | ✅ Via rescue mode + installimage |
| **GPU Availability** | ⚠️ Limited |
| **Pricing** | $ Budget-friendly ($65/mo 3-node) |
| **Regions** | Europe + US East |
| **Documentation** | Good - REST API |

**Two APIs**:
1. **Hetzner Cloud API** - VPS/Cloud servers
2. **Hetzner Robot API** - Dedicated servers (what we need)

**Robot API Endpoints**:
```
GET  /server              # List servers
GET  /server/{ip}         # Get server details
POST /server/{ip}         # Update server
POST /boot/{ip}/rescue    # Enable rescue mode
POST /boot/{ip}/linux     # Configure Linux install
DELETE /boot/{ip}/rescue  # Disable rescue mode
POST /reset/{ip}          # Reset server (hardware/software)
POST /wol/{ip}            # Wake on LAN
```

**iPXE Workflow**:
1. Enable rescue mode: `POST /boot/{ip}/rescue`
2. Reboot server: `POST /reset/{ip}`
3. SSH into rescue, run installimage or custom iPXE
4. Reboot to installed OS

**Pros**:
- Exceptional price/performance ratio
- Very reliable hardware
- Strong European presence
- Server auction for even lower prices

**Cons**:
- Robot API is older design (not JSON:API)
- Limited GPU options
- Manual iPXE requires rescue mode workflow
- US presence limited to Ashburn

**Implementation Priority**: HIGH - Best value for non-GPU workloads

---

### 4. Cherry Servers

**API Documentation**: https://api.cherryservers.com/doc/

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐ Good |
| **iPXE Support** | ✅ Native iPXE chain URL support |
| **GPU Availability** | ⚠️ Limited |
| **Pricing** | $ Budget-friendly |
| **Regions** | Europe + US |
| **Documentation** | Good - OpenAPI spec |

**Key API Endpoints**:
```
POST /v1/projects/{id}/servers     # Deploy server
GET  /v1/servers/{id}              # Get server
DELETE /v1/servers/{id}            # Delete server
POST /v1/servers/{id}/actions      # Server actions (reboot, power_on, power_off)
POST /v1/servers/{id}/reinstall    # Reinstall with custom OS
```

**iPXE Support**:
```json
{
  "project_id": 123,
  "plan": "e5_1620v4",
  "hostname": "talos-node-1",
  "region": "EU-Nord-1",
  "os_partition_size": 20,
  "image": "custom_ipxe",
  "ipxe_url": "https://boot.talos.dev/ipxe?..."
}
```

**Pros**:
- Clean, modern API
- Native iPXE URL support (no rescue mode needed!)
- Competitive pricing
- Good European presence

**Cons**:
- Smaller company than alternatives
- Limited GPU offerings
- Fewer regions than major providers

**Implementation Priority**: MEDIUM - Excellent iPXE support

---

### 5. Scaleway (Dedibox)

**API Documentation**: https://www.scaleway.com/en/developers/api/

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐ Good |
| **iPXE Support** | ✅ Via rescue mode |
| **GPU Availability** | ✅ L4, L40S, H100 |
| **Pricing** | $$ Mid-range |
| **Regions** | Europe (Paris, Amsterdam) |
| **Documentation** | Good - REST API |

**Key API Endpoints**:
```
POST /baremetal/v1/zones/{zone}/servers    # Create server
GET  /baremetal/v1/zones/{zone}/servers    # List servers
GET  /baremetal/v1/zones/{zone}/servers/{id}  # Get server
DELETE /baremetal/v1/zones/{zone}/servers/{id}  # Delete
POST /baremetal/v1/zones/{zone}/servers/{id}/install  # Install OS
POST /baremetal/v1/zones/{zone}/servers/{id}/reboot   # Reboot
```

**Pros**:
- Strong European provider (GDPR compliance)
- Good GPU availability
- Modern API design
- Terraform provider available

**Cons**:
- Europe-only presence
- Dedibox API separate from main Scaleway API
- Some features require manual intervention

**Implementation Priority**: MEDIUM - Good for European deployments

---

### Honorable Mentions (Future Consideration)

#### Equinix Metal (formerly Packet)

**Note**: Equinix is transitioning/deprecating some services. Monitor status.

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐⭐ Excellent |
| **iPXE Support** | ✅ Best-in-class |
| **GPU Availability** | ✅ Full range |
| **Pricing** | $$$ Premium |
| **Status** | ⚠️ Service changes pending |

**If stable**: Would be top recommendation for enterprise.

#### PhoenixNAP

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐ Moderate |
| **iPXE Support** | ✅ Supported |
| **GPU Availability** | ✅ Good |
| **Pricing** | $$ Mid-range |

**Developer Portal**: https://developers.phoenixnap.com/

#### OVHcloud

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐ Moderate |
| **iPXE Support** | ✅ Via rescue mode |
| **GPU Availability** | ⚠️ Limited |
| **Pricing** | $ Budget-friendly |

**Note**: Complex API with regional variations

---

## Big 3 Cloud Providers

### AWS EC2 Bare Metal

**Documentation**: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐⭐ Excellent (boto3/SDK) |
| **iPXE Support** | ⚠️ Not directly - use custom AMI |
| **GPU Availability** | ✅ P5, P4d (H100, A100) |
| **Pricing** | $$$ Premium |
| **Regions** | Global |

**Bare Metal Instance Types**:
- `i3.metal` - Storage optimized
- `m5.metal`, `m5d.metal`, `m5n.metal` - General purpose
- `c5.metal`, `c5n.metal` - Compute optimized
- `r5.metal`, `r5d.metal` - Memory optimized
- `p4d.24xlarge` - GPU (A100)
- `p5.48xlarge` - GPU (H100)

**Talos on AWS**:
- Use Talos AMI (pre-built)
- Or custom AMI with Talos image
- No iPXE - direct AMI boot

**Implementation Approach**:
```rust
// Use aws-sdk-ec2 crate
async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
    let result = self.ec2_client
        .run_instances()
        .image_id("ami-talos-xxx")
        .instance_type(InstanceType::M5Metal)
        .min_count(1)
        .max_count(1)
        .send()
        .await?;
    // ...
}
```

**Pros**:
- Most mature cloud platform
- Excellent SDK support
- Global presence
- Strong GPU availability

**Cons**:
- No iPXE - requires pre-built AMI
- Premium pricing
- Complex IAM setup

**Implementation Priority**: HIGH - Market leader

---

### Google Cloud Platform (GCP)

**Documentation**: https://cloud.google.com/bare-metal

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐ Good |
| **iPXE Support** | ⚠️ Not directly - use custom image |
| **GPU Availability** | ✅ A100, H100, TPU |
| **Pricing** | $$$ Premium |
| **Regions** | Global |

**Bare Metal Solution**:
- Enterprise-grade bare metal
- SAP HANA certified
- Dedicated hardware

**Compute Engine Bare Metal**:
- `n2-standard-*` with sole-tenant nodes
- Custom machine types

**Talos on GCP**:
- Use Talos GCP image
- Upload custom image to GCS
- No iPXE - direct image boot

**Implementation Priority**: MEDIUM - Enterprise focus

---

### Microsoft Azure

**Documentation**: https://learn.microsoft.com/en-us/azure/baremetal-infrastructure/

| Attribute | Value |
|-----------|-------|
| **API Quality** | ⭐⭐⭐⭐ Good |
| **iPXE Support** | ⚠️ Not directly |
| **GPU Availability** | ✅ A100, H100 |
| **Pricing** | $$$ Premium |
| **Regions** | Global |

**BareMetal Infrastructure**:
- SAP HANA Large Instances
- Enterprise workloads

**Azure Dedicated Host**:
- Physical server isolation
- Custom VM placement

**Talos on Azure**:
- Use Talos VHD image
- Upload to Azure Blob Storage
- Create managed image

**Implementation Priority**: MEDIUM - Enterprise focus

---

## At-Home Appliance

### Local Deployment Options

#### 1. Bare Metal (Mini PC/Server)

**Target Hardware**:
- Intel NUC / ASUS NUC
- Beelink Mini PCs
- Dell OptiPlex Micro
- HP EliteDesk Mini
- Custom server hardware

**Deployment Method**:
- Bootable USB with Talos installer
- iPXE network boot from local server
- Direct disk image flash

**Provider Implementation**:
```rust
pub struct LocalProvider {
    /// Path to Talos ISO or raw image
    image_path: PathBuf,
    /// SSH access for existing machines
    ssh_config: Option<SshConfig>,
}

impl Provider for LocalProvider {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError> {
        // For local deployment:
        // 1. Generate Talos machine config
        // 2. If SSH available, push config and trigger install
        // 3. If not, generate bootable media instructions
        todo!("Local deployment guidance")
    }
}
```

#### 2. Kind (Development)

**Use Case**: Local development and testing

Already partially implemented in installer.

#### 3. Multipass/Vagrant (VMs)

**Use Case**: Local VM-based clusters

Useful for testing without dedicated hardware.

---

## Provider Comparison Matrix

| Provider | API Quality | iPXE | GPU | Price | Regions | Priority |
|----------|-------------|------|-----|-------|---------|----------|
| **Latitude.sh** | ⭐⭐⭐⭐⭐ | ✅ | ✅ | $$ | 15+ | ✅ Done |
| **Vultr** | ⭐⭐⭐⭐ | ✅ | ✅ | $$ | 25+ | HIGH |
| **Hetzner** | ⭐⭐⭐⭐ | ✅ | ⚠️ | $ | EU+US | HIGH |
| **Cherry Servers** | ⭐⭐⭐⭐ | ✅ | ⚠️ | $ | EU+US | MEDIUM |
| **Scaleway** | ⭐⭐⭐⭐ | ✅ | ✅ | $$ | EU | MEDIUM |
| **AWS** | ⭐⭐⭐⭐⭐ | ⚠️ | ✅ | $$$ | Global | HIGH |
| **GCP** | ⭐⭐⭐⭐ | ⚠️ | ✅ | $$$ | Global | MEDIUM |
| **Azure** | ⭐⭐⭐⭐ | ⚠️ | ✅ | $$$ | Global | MEDIUM |
| **Local** | N/A | ✅ | ⚠️ | $ | Local | HIGH |

---

## Implementation Roadmap

### Phase 1: Core Bare Metal (Q1)
1. ✅ Latitude.sh (done)
2. 🔲 Vultr
3. 🔲 Hetzner

### Phase 2: Cloud Providers (Q2)
4. 🔲 AWS EC2 Bare Metal
5. 🔲 GCP Compute

### Phase 3: Expanded Coverage (Q3)
6. 🔲 Cherry Servers
7. 🔲 Scaleway
8. 🔲 Azure

### Phase 4: At-Home Appliance (Q3-Q4)
9. 🔲 Local bare metal installer
10. 🔲 Appliance image builder

---

## Provider Trait Compatibility

Current `Provider` trait in `crates/metal/src/providers/traits.rs`:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn create_server(&self, req: CreateServerRequest) -> Result<Server, ProviderError>;
    async fn get_server(&self, id: &str) -> Result<Server, ProviderError>;
    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<Server, ProviderError>;
    async fn reinstall_ipxe(&self, id: &str, req: ReinstallIpxeRequest) -> Result<(), ProviderError>;
    async fn delete_server(&self, id: &str) -> Result<(), ProviderError>;
    async fn list_servers(&self) -> Result<Vec<Server>, ProviderError>;
}
```

**Compatibility Notes**:

| Provider | create_server | get_server | wait_ready | reinstall_ipxe | delete_server | list_servers |
|----------|--------------|------------|------------|----------------|---------------|--------------|
| Latitude | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vultr | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Hetzner | ✅ | ✅ | ✅ | ⚠️ Rescue mode | ✅ | ✅ |
| Cherry | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Scaleway | ✅ | ✅ | ✅ | ⚠️ Rescue mode | ✅ | ✅ |
| AWS | ✅ AMI | ✅ | ✅ | ❌ Use AMI | ✅ | ✅ |
| GCP | ✅ Image | ✅ | ✅ | ❌ Use Image | ✅ | ✅ |
| Azure | ✅ VHD | ✅ | ✅ | ❌ Use VHD | ✅ | ✅ |

**Trait Extension Needed**:
For cloud providers, we may need to extend the trait to support image-based deployment:

```rust
#[async_trait]
pub trait CloudProvider: Provider {
    /// Deploy using a pre-built image (AMI, GCP Image, VHD)
    async fn create_from_image(&self, req: CreateFromImageRequest) -> Result<Server, ProviderError>;
    
    /// Upload custom image to cloud storage
    async fn upload_image(&self, path: &Path) -> Result<String, ProviderError>;
}
```

---

## References

- [Latitude Research](./latitude-research.md)
- [Strategy Analysis](./strategy-analysis.md)
- [Talos iPXE POC](./talos-ipxe-poc.md)
- [Vultr API](https://www.vultr.com/api/)
- [Hetzner Robot API](https://robot.hetzner.com/doc/webservice/en.html)
- [Cherry Servers API](https://api.cherryservers.com/doc/)
- [Scaleway API](https://www.scaleway.com/en/developers/api/)
- [AWS EC2 API](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/)






