# CTO Platform - Product License Agreement

**Version 1.0 - Draft**
**Effective Date: [TBD]**

---

## Overview

This document outlines the licensing terms for the CTO Platform-in-a-Box product
("the Product") offered by 5D Labs ("the Company", "we", "us").

The CTO Platform consists of two distinct components with different licensing:

1. **CTO Platform Core** - Open Source (Apache 2.0)
2. **CTO Platform-in-a-Box** - Commercial License (this document)

---

## Part 1: Open Source Components (Apache 2.0)

### What's Included

The following components are freely available under the Apache 2.0 license:

- CTO Agent Controller
- CTO MCP Server  
- CTO Workflow Engine
- Pre-built Agent Templates (Rex, Blaze, Cypher, etc.)
- CTO CLI Tools
- Helm Charts (manual installation)
- Documentation
- Example Configurations

### Apache 2.0 License Summary

You are free to:
- ✅ Use for any purpose (personal, commercial, etc.)
- ✅ Modify and create derivative works
- ✅ Distribute copies
- ✅ Distribute modified versions
- ✅ Use patents granted by the license

Requirements:
- Include the original copyright notice
- Include the license text
- State significant changes made
- Include NOTICE file if present

The full Apache 2.0 license text is available at:
https://www.apache.org/licenses/LICENSE-2.0

### Source Code

Open source components are available at:
https://github.com/5dlabs/cto

---

## Part 2: Commercial License (Platform-in-a-Box)

### What's Included

The Commercial License covers:

- Bootable ISO / Installation Media
- Setup Wizard and Installer
- Platform Guardian Agent (self-healing system)
- Over-the-Air (OTA) Update System
- Service Marketplace
- Pre-configured Platform Stack
- License Validation System
- Priority Bug Fixes
- Access to Commercial Support (varies by tier)

### License Grant

Subject to the terms of this Agreement and payment of applicable fees, 5D Labs 
grants you a limited, non-exclusive, non-transferable license to:

1. **Install** the Product on hardware you own or control
2. **Use** the Product for your internal business purposes
3. **Allow** your employees and contractors to access the Product
4. **Deploy** applications and workloads on the Product

### License Restrictions

You may NOT:

1. **Redistribute** the Product or any portion thereof
2. **Sublicense** the Product to third parties
3. **Resell** the Product or access to the Product
4. **Reverse engineer** the license validation system
5. **Remove** or modify license validation mechanisms
6. **Exceed** the node limits specified in your license
7. **Share** license keys with other organizations
8. **Use** the Product to provide managed services to third parties
   (unless authorized under an MSP/Reseller agreement)

### License Tiers

| Tier | Nodes | Agents | Features |
|------|-------|--------|----------|
| **Starter** | 1 | 5 | Core features |
| **Team** | 3 | 15 | Core + multi-node ready |
| **Business** | 10 | Unlimited | Full features |
| **Enterprise** | Unlimited | Unlimited | Full + SSO + air-gap |

---

## Part 3: License Terms

### License Duration

- **Subscription licenses** are valid for the purchased term (monthly or annual)
- **Perpetual licenses** (if offered) are valid indefinitely for the purchased version

### License Renewal

- Licenses automatically renew unless cancelled
- Renewal pricing may change with 30 days notice
- Non-renewal results in license expiration (see below)

### License Expiration

When a license expires:

| Timeline | Effect |
|----------|--------|
| Expiration | New feature installation disabled |
| +7 days | OTA updates disabled |
| +14 days | Guardian agent enters monitor-only mode |
| +30 days | New agent deployments blocked |
| +90 days | Platform enters read-only mode |

**Important:** Your existing workloads and data are NEVER deleted or disabled 
due to license expiration. You retain full access to your data at all times.

### License Validation

The Product includes license validation mechanisms that:

- Verify license authenticity using cryptographic signatures
- Check license expiration status
- Enforce node and feature limits
- Optionally communicate with 5D Labs servers for validation

Tampering with, circumventing, or disabling license validation mechanisms 
constitutes a material breach of this Agreement.

---

## Part 4: Support Terms

### Support Tiers

| Tier | Response Time | Hours | Channels |
|------|---------------|-------|----------|
| **Community** | Best effort | - | GitHub, Discord |
| **Standard** | 48 hours | Business hours | Email, Tickets |
| **Priority** | 8 hours | Extended hours | Email, Tickets, Chat |
| **Enterprise** | 4 hours | 24/7 | Phone, Dedicated Slack |

### Support Scope

Support includes:
- Installation assistance
- Configuration guidance
- Troubleshooting platform issues
- Bug fixes and patches
- Upgrade assistance

Support does NOT include:
- Custom development
- Application debugging (your code)
- Hardware support
- Third-party software support
- Training (available separately)

### Support Exclusions

Support may be limited or unavailable for:
- Unsupported hardware configurations
- Modified or tampered installations
- Expired licenses
- Violations of the license agreement

---

## Part 5: Third-Party Components

### Bundled Open Source Software

The Product includes the following open source components:

| Component | License | Copyright |
|-----------|---------|-----------|
| Kubernetes | Apache 2.0 | The Kubernetes Authors |
| Talos Linux | MPL 2.0 | Sidero Labs |
| ArgoCD | Apache 2.0 | Argo Project |
| OpenBao | MPL 2.0 | OpenBao Project |
| CloudNativePG | Apache 2.0 | CloudNativePG Contributors |
| Prometheus | Apache 2.0 | Prometheus Authors |
| Grafana | AGPL 3.0 | Grafana Labs (unmodified) |
| Loki | AGPL 3.0 | Grafana Labs (unmodified) |
| MinIO | AGPL 3.0 | MinIO Inc (unmodified) |
| Longhorn | Apache 2.0 | Longhorn Authors |
| Cilium | Apache 2.0 | Cilium Authors |
| cert-manager | Apache 2.0 | cert-manager Authors |

Full license texts are included in the `/licenses` directory of the installation.

### Your Obligations

You must:
- Retain all copyright notices in bundled software
- Comply with the terms of bundled open source licenses
- Not remove or modify attribution notices

### AGPL Components Notice

Grafana, Loki, and MinIO are licensed under AGPL 3.0. These components are 
included **unmodified**. If you modify these components, you must comply with 
AGPL 3.0 requirements, including making your modifications available under AGPL.

---

## Part 6: Data & Privacy

### Your Data

- **Ownership:** You retain full ownership of all data stored on the platform
- **Access:** We do not access your data without explicit permission
- **Portability:** You can export your data at any time
- **Deletion:** Your data is never deleted due to license expiration

### Telemetry (Opt-In)

The Product may collect anonymized telemetry data if you opt in:

**What we collect:**
- Platform health metrics
- Component version information
- Error rates and types (no stack traces with customer data)
- Feature usage statistics

**What we NEVER collect:**
- Source code or application data
- API keys, passwords, or secrets
- Personal information
- Business data or intellectual property

You can disable telemetry at any time in the platform settings.

### Phone-Home

For license validation, the Product may periodically contact 5D Labs servers.
This communication includes only:
- License key identifier
- Current node count
- Product version

Air-gap installations operate fully offline with no phone-home requirement.

---

## Part 7: Warranties & Liability

### Limited Warranty

5D Labs warrants that:
- The Product will perform substantially as documented
- Updates will not intentionally reduce functionality
- We have the right to license the Product

### Disclaimer

THE PRODUCT IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND, EXPRESS OR 
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, 
FITNESS FOR A PARTICULAR PURPOSE, AND NONINFRINGEMENT.

### Limitation of Liability

IN NO EVENT SHALL 5D LABS BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, 
CONSEQUENTIAL, OR PUNITIVE DAMAGES, INCLUDING BUT NOT LIMITED TO LOSS OF 
PROFITS, DATA, OR USE, ARISING OUT OF OR IN CONNECTION WITH THIS AGREEMENT.

OUR TOTAL LIABILITY SHALL NOT EXCEED THE AMOUNTS PAID BY YOU IN THE TWELVE 
(12) MONTHS PRECEDING THE CLAIM.

---

## Part 8: General Terms

### Termination

This Agreement may be terminated:
- By you at any time by ceasing use and destroying all copies
- By us for material breach with 30 days notice to cure
- Immediately by us for license tampering or circumvention

Upon termination:
- Your license rights cease
- You must destroy all copies of the Product
- Your data remains yours (export before termination)

### Governing Law

This Agreement shall be governed by the laws of [Jurisdiction], without 
regard to conflict of law principles.

### Entire Agreement

This Agreement constitutes the entire agreement between the parties and 
supersedes all prior agreements and understandings.

### Modifications

We may modify this Agreement with 30 days notice. Continued use after 
modifications constitutes acceptance.

### Assignment

You may not assign this Agreement without our prior written consent. 
We may assign this Agreement in connection with a merger or acquisition.

### Severability

If any provision of this Agreement is found unenforceable, the remaining 
provisions shall continue in effect.

---

## Part 9: Contact Information

**5D Labs**

- Website: https://5dlabs.io
- Support: support@5dlabs.io
- Sales: sales@5dlabs.io
- Legal: legal@5dlabs.io

---

## Acceptance

By installing, copying, or otherwise using the Product, you acknowledge that 
you have read this Agreement, understand it, and agree to be bound by its 
terms and conditions.

If you do not agree to these terms, do not install or use the Product.

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0-draft | 2025-11-27 | Initial draft |

---

*This is a draft document for planning purposes. Final license agreement 
should be reviewed by legal counsel before publication.*

