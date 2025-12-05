# CTO Platform: Licensing & Compliance Analysis

## Purpose

This document analyzes the licensing status of all components in the CTO Platform
infrastructure to ensure compliance when distributing as a commercial product.

**Key Questions:**
1. Can we bundle this component in a commercial product?
2. Are there any copyleft obligations that affect our code?
3. Do we need to purchase commercial licenses?
4. Are there alternatives if licensing is problematic?

---

## License Types Overview

| License | Commercial Use | Redistribution | Copyleft | Notes |
|---------|---------------|----------------|----------|-------|
| **Apache 2.0** | ✅ Yes | ✅ Yes | ❌ No | Best for commercial |
| **MIT** | ✅ Yes | ✅ Yes | ❌ No | Very permissive |
| **BSD** | ✅ Yes | ✅ Yes | ❌ No | Very permissive |
| **MPL 2.0** | ✅ Yes | ✅ Yes | 🟡 File-level | Okay, keep MPL files separate |
| **LGPL** | ✅ Yes | ✅ Yes | 🟡 Library-level | Okay if dynamically linked |
| **GPL-3.0** | ⚠️ Complex | ✅ Yes | 🔴 Strong | Must open source derivative works |
| **AGPL-3.0** | ⚠️ Complex | ✅ Yes | 🔴 Network | GPL + network use triggers |
| **BSL** | ⚠️ Restricted | ⚠️ Limited | ❌ No | Time-delayed open source |
| **Elastic/SSPL** | ❌ No | ❌ No | 🔴 Extreme | Cannot offer as service |

---

## Infrastructure Components Analysis

### ✅ CLEAR - No Licensing Concerns

| Component | License | Status | Notes |
|-----------|---------|--------|-------|
| **Kubernetes** | Apache 2.0 | ✅ Clear | CNCF graduated, fully permissive |
| **Talos Linux** | MPL 2.0 | ✅ Clear | File-level copyleft only, Sidero Labs is commercial-friendly |
| **ArgoCD** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **CNPG** | Apache 2.0 | ✅ Clear | CloudNativePG, EDB-backed |
| **cert-manager** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **Ingress-NGINX** | Apache 2.0 | ✅ Clear | Kubernetes project |
| **Longhorn** | Apache 2.0 | ✅ Clear | CNCF graduated, Rancher/SUSE |
| **Cilium** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **CoreDNS** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **Prometheus** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **Alertmanager** | Apache 2.0 | ✅ Clear | Part of Prometheus project |
| **Helm** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **containerd** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **etcd** | Apache 2.0 | ✅ Clear | CNCF graduated |
| **MetalLB** | Apache 2.0 | ✅ Clear | |
| **External Secrets** | Apache 2.0 | ✅ Clear | |
| **Kustomize** | Apache 2.0 | ✅ Clear | Kubernetes SIG |

### ⚠️ NEEDS ATTENTION - Review Required

#### HashiCorp Vault - BSL (Business Source License)

| Aspect | Status |
|--------|--------|
| License | BSL 1.1 (since Aug 2023) |
| Commercial Use | ⚠️ Restricted |
| Our Use Case | Bundling in commercial appliance |

**The Issue:**
HashiCorp changed Vault from MPL 2.0 to BSL 1.1 in August 2023. The BSL restricts:
- Offering Vault as a managed service
- Competing with HashiCorp's commercial offerings

**Analysis:**
Reading the BSL additional use grant for Vault:
> "You may make production use of the Licensed Work, provided such use does not 
> include offering the Licensed Work to third parties on a hosted or embedded 
> basis which is competitive with HashiCorp's products."

**Our situation:**
- We're not offering Vault-as-a-Service
- We're bundling Vault as part of a larger platform
- Primary product is AI agents, not secrets management
- Vault is an internal component, not the product itself

**Risk Level: MEDIUM**

**Options:**
1. **Continue with Vault** - Likely okay under BSL terms, but legal gray area
2. **Use OpenBao** - Community fork of Vault (MPL 2.0), API-compatible
3. **Commercial license** - Contact HashiCorp for OEM licensing
4. **Alternative** - Use Kubernetes secrets + Sealed Secrets (less feature-rich)

**Recommendation:** Switch to **OpenBao** (Vault fork, MPL 2.0)
- Drop-in replacement, same API
- Actively maintained by Linux Foundation
- No licensing concerns
- https://openbao.org/

---

#### Grafana - AGPL-3.0

| Aspect | Status |
|--------|--------|
| License | AGPL-3.0 |
| Commercial Use | ⚠️ Complex |
| Our Use Case | Bundled dashboards |

**The Issue:**
AGPL requires that if you modify Grafana and provide it over a network, you must 
release your modifications under AGPL.

**Analysis:**
- We're not modifying Grafana source code
- We're bundling unmodified Grafana
- We're providing pre-built dashboards (JSON, not code)
- Users access Grafana in their own cluster

**Risk Level: LOW**

**Grafana Labs' stance:**
Grafana Labs explicitly allows bundling unmodified Grafana:
> "Using unmodified Grafana in your solution is fine."

**Options:**
1. **Continue with Grafana** - Okay if unmodified
2. **Grafana Enterprise** - Commercial license available
3. **Alternative** - Use Perses (CNCF sandbox, Apache 2.0) - less mature

**Recommendation:** Continue with unmodified Grafana, document that we don't modify it.

---

#### Grafana Loki - AGPL-3.0

| Aspect | Status |
|--------|--------|
| License | AGPL-3.0 |
| Commercial Use | ⚠️ Complex |
| Our Use Case | Log aggregation backend |

**Analysis:**
Same as Grafana - bundling unmodified is okay.

**Risk Level: LOW**

**Options:**
1. **Continue with Loki** - Okay if unmodified
2. **Alternative** - OpenSearch (Apache 2.0) - heavier but more permissive
3. **Alternative** - VictoriaLogs (Apache 2.0) - newer, very promising

**Recommendation:** Continue with unmodified Loki, or consider VictoriaLogs for v2.

---

#### MinIO - AGPL-3.0

| Aspect | Status |
|--------|--------|
| License | AGPL-3.0 |
| Commercial Use | ⚠️ Complex |
| Our Use Case | S3-compatible storage |

**The Issue:**
AGPL means modifications must be shared. MinIO also has commercial licensing.

**Analysis:**
- We're bundling unmodified MinIO
- It's a backend service, users don't directly interact with code
- We're not building a storage product

**Risk Level: LOW-MEDIUM**

**MinIO's stance:**
MinIO offers commercial licenses for OEM/embedding use cases.

**Options:**
1. **Continue with MinIO** - Okay if unmodified (verify with MinIO)
2. **Commercial license** - Contact MinIO for OEM pricing
3. **Alternative** - SeaweedFS (Apache 2.0) - S3-compatible
4. **Alternative** - Garage (AGPL, but simpler) - for edge/small deployments

**Recommendation:** Verify with MinIO legal, or switch to **SeaweedFS** (Apache 2.0).

---

### 🔴 MUST REPLACE

#### TaskMaster

| Aspect | Status |
|--------|--------|
| License | Unknown/Proprietary? |
| Commercial Use | ❌ Not suitable |
| Our Use Case | Task management |

**The Issue:**
TaskMaster (assuming @anthropic-ai/claude-code based task management) may not be 
licensed for redistribution in commercial products.

**Recommendation:** Build our own task management solution.

**Replacement scope:**
- Task creation and tracking
- Status management
- Integration with agents
- UI components

**Priority: HIGH** - Must replace before commercial release.

---

### External Services (Not Bundled)

These are services that customers may use but we don't bundle:

| Service | How We Use It | Licensing Impact |
|---------|---------------|------------------|
| **Cloudflare** | Optional tunnel for ingress | ✅ None - customer's account |
| **GitHub** | Repository integration | ✅ None - customer's account |
| **OpenAI/Anthropic** | API calls from agents | ✅ None - customer's API keys |

**Cloudflare Operator:**
- The cloudflare-operator is Apache 2.0 ✅
- We can bundle the operator
- Customers use their own Cloudflare accounts
- No licensing issue

---

## Rust Crates Analysis

Our Rust code uses various crates. Common permissive licenses in Rust ecosystem:

| License | Crates | Status |
|---------|--------|--------|
| MIT | Most crates | ✅ Clear |
| Apache 2.0 | Many crates | ✅ Clear |
| MIT OR Apache-2.0 | Standard dual-license | ✅ Clear |

**Action:** Run `cargo license` to audit all dependencies before release.

```bash
# Install cargo-license
cargo install cargo-license

# Audit all crates
cargo license --all-features --json > license-audit.json

# Check for problematic licenses
cargo license | grep -E "(GPL|AGPL|SSPL|BSL)"
```

---

## Summary: Required Actions

### Before Commercial Release

| Priority | Component | Action | Effort |
|----------|-----------|--------|--------|
| 🔴 HIGH | TaskMaster | Build replacement | 2-4 weeks |
| 🟡 MEDIUM | Vault | Switch to OpenBao | 1 week |
| 🟢 LOW | MinIO | Verify with MinIO legal OR switch to SeaweedFS | 1 day / 1 week |
| 🟢 LOW | Grafana/Loki | Document "unmodified" usage | 1 day |
| 🟢 LOW | Rust crates | Run license audit | 1 day |

### Recommended Replacements

| Current | Replacement | License | Compatibility |
|---------|-------------|---------|---------------|
| HashiCorp Vault | OpenBao | MPL 2.0 | 100% API compatible |
| MinIO (if needed) | SeaweedFS | Apache 2.0 | S3 compatible |
| Grafana (if needed) | Perses | Apache 2.0 | Less mature |
| Loki (if needed) | VictoriaLogs | Apache 2.0 | Different query lang |
| TaskMaster | Custom solution | Ours | Must build |

---

## License Compliance Checklist

Before each release:

- [ ] Run `cargo license` audit on all Rust dependencies
- [ ] Verify no GPL/AGPL code has been modified
- [ ] Document all bundled components and their licenses
- [ ] Include NOTICE file with attribution
- [ ] Include LICENSE files for all bundled components
- [ ] Verify no proprietary code is bundled
- [ ] Legal review of any new dependencies

---

## NOTICE File Template

```
CTO Platform
Copyright 2025 5D Labs

This product includes software developed by:

- The Kubernetes Authors (Apache 2.0)
- Sidero Labs - Talos (MPL 2.0)
- Argo Project - ArgoCD (Apache 2.0)
- CloudNativePG - CNPG (Apache 2.0)
- OpenBao Project - OpenBao (MPL 2.0)
- Grafana Labs - Grafana, Loki (AGPL 3.0, unmodified)
- MinIO Inc - MinIO (AGPL 3.0, unmodified)
- Prometheus Authors - Prometheus (Apache 2.0)
- CNCF - Various projects (Apache 2.0)

Full license texts are available in the /licenses directory.
```

---

## Legal Disclaimer

This document is for internal planning purposes and does not constitute legal advice.
Before commercial release, consult with a lawyer specializing in open source licensing.

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2025-11-27 | 5D Labs | Initial analysis |

