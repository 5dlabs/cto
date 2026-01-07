# Implementation Roadmap: Freemium SaaS Model

This document outlines the changes required to implement the freemium SaaS model with fully managed bare metal infrastructure.

## Summary of Strategic Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Business Model** | Freemium (proprietary) | No contributor overhead, faster iteration, "staying current" as moat |
| **Open Source** | No | Full control, no forks, no community management |
| **Infrastructure (5D Labs)** | Bare metal only (Latitude.sh) | Lower cost, better performance, full control |
| **Infrastructure (Enterprise)** | Customer-managed (their cloud/on-prem) | We provide software, they run it |
| **Cloud Provisioning** | Remove | We don't provision cloud resources; enterprise customers do their own |

---

## Code Changes Required

### 1. Remove `crates/cloud`

**Status:** Not yet started
**Priority:** Medium
**Effort:** Low (not yet integrated)

The `crates/cloud` crate provides AWS, GCP, and Azure provisioning capabilities. This is no longer needed:

- **5D Labs managed:** Uses bare metal only (Latitude.sh via `crates/metal`)
- **Enterprise customer-managed:** Customer provisions their own cloud infrastructure

**Actions:**
- [ ] Remove `crates/cloud` from workspace `Cargo.toml`
- [ ] Delete `crates/cloud/` directory
- [ ] Remove any references in documentation
- [ ] Update architecture diagrams

**Files to modify:**
```
Cargo.toml                    # Remove "crates/cloud" from members
crates/cloud/                 # Delete entire directory
docs/business/saas-architecture.md  # Remove cloud provisioning sections
```

---

### 2. Update Architecture Documentation

**Status:** Not yet started
**Priority:** High
**Effort:** Medium

The architecture docs still reference cloud provisioning by 5D Labs. Update to reflect:

- 5D Labs infrastructure = Bare metal only (Latitude.sh)
- Enterprise customer infrastructure = Customer's responsibility (they can use any cloud)
- We don't provision cloud resources FOR customers

**Sections to update in `saas-architecture.md`:**
- [ ] "Infrastructure Provisioning" section - remove "Cloud Providers" column
- [ ] "Infrastructure Provider Integration" section - simplify to bare metal only
- [ ] "Infrastructure MCP Tools Available" table - remove future AWS/GCP tools
- [ ] "Data Flow" section - clarify enterprise options

---

### 3. Feature Gating by Tier

**Status:** Not implemented
**Priority:** High
**Effort:** High

Implement tier-based feature access in the platform:

| Feature | Free | Team | Growth | Enterprise |
|---------|------|------|--------|------------|
| CodeRuns/month | 50 | 200 | 1000 | Custom |
| Users | 1 | 10 | Unlimited | Unlimited |
| Repositories | 5 | Unlimited | Unlimited | Unlimited |
| AI Keys | BYOK only | BYOK or managed | BYOK or managed | Flexible |
| SSO | ✗ | Basic (Google/MS) | Full SAML/SCIM | Full + custom |
| Healer | ✗ | ✗ | ✓ | ✓ |
| Audit logs | ✗ | ✗ | ✓ | ✓ |
| Source code access | ✗ | ✗ | ✗ | ✓ (under NDA) |
| Customer infra | ✗ | ✗ | ✗ | ✓ (optional) |

**Implementation needs:**
- [ ] Tenant configuration schema with tier field
- [ ] Middleware for feature gating
- [ ] Usage tracking (CodeRuns, users, repos)
- [ ] Overage billing integration
- [ ] Upgrade prompts in UI/CLI

---

### 4. Multi-Tenancy

**Status:** Partially implemented
**Priority:** High
**Effort:** High

Enable isolated tenant environments on shared infrastructure:

**Requirements:**
- [ ] Per-tenant namespaces in Kubernetes
- [ ] Per-tenant secrets vault (OpenBao paths)
- [ ] Per-tenant API keys / installation tokens
- [ ] Tenant isolation in agent execution
- [ ] Usage metering per tenant

---

### 5. Web Portal / Dashboard

**Status:** Not started
**Priority:** High
**Effort:** Very High

Customer-facing web application for:

- [ ] Sign up / login (GitHub OAuth)
- [ ] Connect integrations (GitHub App, Linear)
- [ ] Create and monitor tasks
- [ ] View CodeRun history and logs
- [ ] Team management
- [ ] Billing and usage

**Stack considerations:**
- Next.js 15 + shadcn/ui (Blaze agent stack)
- Better Auth for authentication
- Drizzle for database

---

### 6. Public GitHub App

**Status:** Not started
**Priority:** High
**Effort:** Medium

Create a public GitHub App for customer OAuth:

- [ ] Register 5D Labs GitHub App
- [ ] Implement OAuth callback flow
- [ ] Store installation tokens per tenant
- [ ] Token refresh mechanism

---

### 7. Billing Integration

**Status:** Not started
**Priority:** Medium
**Effort:** Medium

Integrate with Stripe for:

- [ ] Subscription management (tiers)
- [ ] Usage-based billing (CodeRun overage)
- [ ] AI token pass-through billing
- [ ] Invoice generation

---

## Documentation Updates

### Files to Create
- [x] `docs/business/` folder
- [x] `docs/business/implementation-roadmap.md` (this file)
- [ ] `docs/business/security-compliance.md` - Enterprise security approach

### Files to Update
- [ ] `docs/business/saas-architecture.md` - Remove cloud provisioning
- [ ] `AGENTS.md` - Update with SaaS context
- [ ] `docs/development-guide.md` - Add multi-tenant dev setup

### Files to Remove
- [x] `docs/open-source-strategy.md` - No longer applicable

---

## Infrastructure Changes

### Bare Metal Only (5D Labs Managed)

```
Current: crates/metal (Latitude.sh) + crates/cloud (AWS/GCP/Azure)
Target:  crates/metal (Latitude.sh) only

5D Labs provisions:
├── Control plane nodes
├── Agent execution pool
├── GPU nodes (for ML workloads)
└── All managed via Latitude.sh API
```

### Enterprise Customer-Managed

```
Enterprise customers who need their own infrastructure:

Option A: We provide Helm chart, they deploy to their K8s
Option B: We provide container images, they run how they want
Option C: Air-gapped package for fully isolated environments

We provide: Software, updates, support
They provide: Infrastructure, networking, security controls
```

---

## Migration Path

### Phase 1: Foundation (Current Focus)
- [ ] Remove cloud crate
- [ ] Update documentation
- [ ] Define tenant schema
- [ ] Implement basic feature gating

### Phase 2: Multi-Tenancy
- [ ] Per-tenant isolation
- [ ] Usage tracking
- [ ] GitHub App OAuth flow

### Phase 3: Web Portal
- [ ] Customer dashboard
- [ ] Billing integration
- [ ] Self-serve signup

### Phase 4: Enterprise Features
- [ ] Helm chart for customer deployment
- [ ] Source code access portal
- [ ] SSO/SAML integration
- [ ] Audit logging

---

## Open Questions

1. **Free tier limits:** Is 50 CodeRuns/month the right number? Should we track by minutes instead?
2. **Healer access:** Should Healer be Growth+ only, or available to all tiers with limits?
3. **AI key management:** Should we offer managed keys on Free tier (with markup)?
4. **Enterprise pricing:** Flat fee vs. usage-based vs. hybrid for enterprise customers?
