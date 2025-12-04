# Bare Metal Provider Research Prompt

> Give this prompt to a research agent with web search capabilities

---

## Research Assignment: Bare Metal Cloud Providers for Kubernetes Infrastructure

### Context

We are building a platform that deploys Kubernetes clusters on bare metal infrastructure using **Talos Linux**. Talos is an immutable, API-driven OS designed specifically for Kubernetes. Our provisioning system needs to programmatically deploy servers and install Talos via **iPXE network boot**.

We currently have one provider implemented (Latitude.sh) and need to identify **5+ additional bare metal providers** that meet our technical requirements.

### Critical Technical Requirements

#### 1. iPXE / Custom OS Boot Support (MANDATORY)

This is our **most critical requirement**. The provider MUST support one of these methods:

| Method | Priority | Description |
|--------|----------|-------------|
| **Native iPXE Chain URL** | ⭐⭐⭐ Best | API accepts an `ipxe_url` parameter during server creation or reinstall. Server boots directly from our iPXE script. |
| **Custom iPXE Script** | ⭐⭐⭐ Best | API accepts raw iPXE script content that the provider serves to the server. |
| **Rescue Mode + SSH** | ⭐⭐ Good | Provider offers Linux rescue mode. We SSH in, download Talos, and `dd` to disk. |
| **IPMI/KVM Access** | ⭐ Acceptable | Out-of-band management lets us mount ISO and install manually. |
| **Custom Image Upload** | ⭐ Acceptable | We upload a Talos disk image, provider boots from it. |

**Search queries to find this information:**
- `"[provider name]" iPXE custom boot`
- `"[provider name]" API custom operating system`
- `"[provider name]" bare metal PXE boot`
- `"[provider name]" rescue mode SSH`
- `"[provider name]" IPMI remote console`

#### 2. REST API for Automation (MANDATORY)

The provider MUST have a REST API that supports:

| Operation | Required | Notes |
|-----------|----------|-------|
| Create/deploy server | ✅ Yes | Programmatic provisioning |
| Get server status | ✅ Yes | Poll for ready state |
| Delete/terminate server | ✅ Yes | Cleanup |
| List servers | ✅ Yes | Inventory |
| Reinstall OS | ✅ Yes | For iPXE reinstall workflow |
| Power actions (on/off/reboot) | ✅ Yes | Server management |
| List available plans/regions | Nice to have | Discovery |

**Search queries:**
- `"[provider name]" API documentation`
- `"[provider name]" REST API reference`
- `"[provider name]" API server create`
- `"[provider name]" developer documentation`

#### 3. Server Specifications

**Minimum viable specs:**
- 4+ CPU cores
- 16+ GB RAM  
- 100+ GB SSD/NVMe storage
- 1+ Gbps network

**Preferred specs for production:**
- 8+ CPU cores
- 32+ GB RAM
- 500+ GB NVMe storage
- 10+ Gbps network
- GPU options (NVIDIA H100, A100, L40S) for AI workloads

### Research Tasks

For EACH provider you identify, gather the following information:

#### A. Provider Overview
- [ ] Company name, website, headquarters location
- [ ] Year founded, funding status, notable customers
- [ ] Data center locations (list all regions)
- [ ] Overall reputation and reliability

#### B. API Capabilities
- [ ] API documentation URL
- [ ] Authentication method (API key, OAuth, etc.)
- [ ] API style (REST, GraphQL, etc.)
- [ ] SDK availability (Go, Python, Rust, Terraform)
- [ ] Rate limits
- [ ] Webhook support for status updates

#### C. iPXE / Custom Boot Support (CRITICAL)
- [ ] Does the API support custom iPXE URL during provisioning?
- [ ] Does the API support custom iPXE script content?
- [ ] Is there a rescue/recovery mode with SSH access?
- [ ] Is IPMI/KVM/remote console available?
- [ ] Can custom disk images be uploaded and used?
- [ ] Document the exact API endpoint and parameters for custom boot

#### D. Server Plans and Pricing
- [ ] Entry-level bare metal plan (specs + price)
- [ ] Mid-range bare metal plan (specs + price)
- [ ] High-end bare metal plan (specs + price)
- [ ] GPU server options and pricing
- [ ] Billing model (hourly, monthly, yearly)
- [ ] Any setup fees or minimums

#### E. Networking Features
- [ ] Private networking / VLANs between servers
- [ ] Additional IP addresses
- [ ] Load balancer options
- [ ] DDoS protection
- [ ] Bandwidth pricing model

#### F. Talos Linux Compatibility
- [ ] Any documented Talos deployments on this provider
- [ ] Community guides or tutorials
- [ ] Known issues or limitations

### Providers to Research

#### Tier 1: Major Bare Metal Specialists (Research Thoroughly)
1. **Vultr** - https://www.vultr.com/products/bare-metal/
2. **Hetzner** - https://www.hetzner.com/dedicated-rootserver
3. **OVHcloud** - https://www.ovhcloud.com/en/bare-metal/
4. **Equinix Metal** - https://deploy.equinix.com/metal/
5. **Cherry Servers** - https://www.cherryservers.com/
6. **Scaleway Dedibox** - https://www.scaleway.com/en/dedibox/

#### Tier 2: Cloud Providers with Bare Metal (Research for Comparison)
7. **AWS EC2 Bare Metal** - `*.metal` instance types
8. **Google Cloud Bare Metal Solution**
9. **Azure BareMetal Infrastructure**
10. **Oracle Cloud Bare Metal**
11. **IBM Cloud Bare Metal**

#### Tier 3: Emerging/Niche Providers (Quick Assessment)
12. **PhoenixNAP** - https://phoenixnap.com/bare-metal-cloud
13. **Packet** (now Equinix) - historical reference
14. **Leaseweb** - https://www.leaseweb.com/dedicated-servers
15. **Servers.com** - https://www.servers.com/
16. **Limestone Networks**
17. **ServerMania**
18. **Liquid Web**
19. **Fasthosts**
20. **Contabo** - https://contabo.com/

#### Tier 4: Regional/Specialty Providers (If Time Permits)
21. **Hivelocity** - US-based
22. **DataPacket** - European
23. **Zenlayer** - Asia-Pacific focus
24. **Webdock** - European budget
25. **Time4VPS** - Lithuanian budget

### Specific Questions to Answer

1. **Which providers have native iPXE URL support in their API?**
   - This is the gold standard - we can pass `https://boot.talos.dev/...` directly

2. **Which providers support Talos Linux officially or have community guides?**
   - Search: `"Talos Linux" "[provider name]"`
   - Check Talos documentation: https://www.talos.dev/docs/

3. **Which providers have the best price/performance for a 3-node Kubernetes cluster?**
   - Calculate: 3x (8 core, 32GB RAM, 500GB NVMe) monthly cost

4. **Which providers have GPU servers (H100, A100, L40S) available?**
   - Important for AI/ML workloads

5. **Which providers have the most regions/locations?**
   - Important for geographic distribution

6. **Are there any newer bare metal providers (2022-2024) worth considering?**
   - The market is evolving rapidly

7. **What's the current status of Equinix Metal?**
   - Reports of service changes/deprecation - verify current state

### Output Format

Please provide your findings in this structure:

```markdown
## [Provider Name]

### Overview
- Website: 
- Headquarters:
- Founded:
- Regions: [list]

### API Assessment
- Documentation: [URL]
- Quality: ⭐⭐⭐⭐⭐ (1-5 stars)
- SDK Available: [Yes/No - list languages]

### iPXE/Custom Boot Support
- Native iPXE URL: [Yes/No]
- Custom iPXE Script: [Yes/No]
- Rescue Mode: [Yes/No]
- IPMI Access: [Yes/No]
- Custom Image: [Yes/No]
- **Verdict**: [Excellent/Good/Limited/None]

### Pricing (3-node cluster estimate)
- Entry config: $X/mo
- Production config: $X/mo
- GPU available: [Yes/No - models]

### Talos Compatibility
- Official support: [Yes/No]
- Community guides: [links if any]
- Known issues: [any]

### Recommendation
- Priority: [HIGH/MEDIUM/LOW/SKIP]
- Reasoning: [1-2 sentences]
```

### Additional Research Sources

- **Talos Linux Documentation**: https://www.talos.dev/
- **Talos GitHub Discussions**: Search for provider-specific threads
- **Reddit r/homelab**: Community experiences with providers
- **Reddit r/selfhosted**: Self-hosting community insights  
- **Hacker News**: Search for provider reviews/discussions
- **ServerHunter**: https://www.serverhunter.com/ - Comparison tool
- **BareMetalCloud.com**: Industry news and comparisons

### What We're NOT Looking For

- **VPS/Virtual Servers** - We need actual bare metal, not virtualized
- **Managed Kubernetes** - We're deploying our own with Talos
- **Colocation** - We need on-demand API-provisioned servers
- **Resellers** - We want direct relationships with providers

### Deliverables

1. **Ranked list** of top 10 providers with detailed assessments
2. **Comparison matrix** showing key features side-by-side
3. **Recommendation** for which 5 providers we should implement first
4. **Any surprises** - providers that are better or worse than expected
5. **Market insights** - trends, new entrants, providers to watch

---

*This research will inform our multi-provider bare metal provisioning system for the CTO Platform.*






