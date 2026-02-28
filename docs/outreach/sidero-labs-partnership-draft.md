# Partnership Email Draft: Sidero Labs x 5D Labs (CTO Platform)

## Email Subject
Partnership Opportunity: CTO Platform Integration with Sidero Metal & Omni

## Email Body

---

**To:** sales@siderolabs.com
**Subject:** Partnership Opportunity: CTO Platform Integration with Sidero Metal & Omni

Hi Sidero Labs Team,

My name is Jonathon Fritz, and I'm the founder of **5D Labs**, where we've built **CTO** (Cognitive Task Orchestrator) — an open-source AI engineering platform that deploys autonomous development teams on bare-metal Kubernetes infrastructure.

## Why We're Reaching Out

We recently realized we made a **significant oversight** by not including Sidero Metal and Omni in our infrastructure provider lineup, despite our platform being **built entirely on Talos Linux**. Given our shared technology foundation, this feels like a natural partnership opportunity.

## About CTO Platform

**CTO** is an AGPL-3.0 licensed platform that provides:
- **13 Specialized AI Agents** (Morgan, Rex, Grizz, Nova, Blaze, Tap, Spark, Cleo, Cipher, Tess, Stitch, Atlas, Bolt) handling everything from PRD intake to production deployment
- **Bare-Metal First Architecture** — running on Talos Linux with 60-80% cost savings vs cloud
- **Multi-Provider Support** — currently supporting Latitude, Hetzner, OVH, Vultr, Scaleway, Cherry Servers, DigitalOcean, Servers.com, PhoenixNAP, i3D.net (FlexMetal), and Hivelocity
- **Self-Healing Infrastructure** — automatic remediation of workflow failures, pod issues, and CI problems
- **GitOps-Driven** — ArgoCD-based deployments with GitHub Apps integration

**Tech Stack:**
- Kubernetes (Talos Linux)
- Rust (Tokio, Axum) + Go + Node.js
- Argo Workflows + ArgoCD
- MCP (Model Context Protocol) for AI agent tools
- CloudNative-PG, Redis Operator, Kafka (Strimzi), SeaweedFS, OpenBao

🔗 **GitHub:** https://github.com/5dlabs/cto
🌐 **Website:** https://5dlabs.ai
💬 **Discord:** https://discord.gg/A6yydvjZKY

## The Integration Plan

We're planning to integrate **Sidero Metal** and **Omni** as first-class infrastructure providers:

### Technical Integration
1. **Sidero Metal Provider** — Add native support in our `cto-metal` CLI for provisioning bare-metal nodes via Sidero Metal's Cluster API integration
2. **Omni SaaS Integration** — Integrate Omni's API for cluster lifecycle management, making it available as a managed option alongside our self-hosted deployments
3. **Agent Skills** — Create OpenClaw agent skills documenting Sidero/Omni provisioning workflows, best practices, and troubleshooting

### Marketing Integration
1. **Homepage Feature** — Prominently display Sidero Metal and Omni on our infrastructure providers page at https://5dlabs.ai
2. **Documentation** — Comprehensive guides for deploying CTO on Sidero Metal and Omni
3. **Cross-Promotion** — Joint blog posts, case studies, and community engagement

### Business Case
This partnership benefits both communities:
- **For Sidero Labs:** Exposure to the AI development automation space and the growing CTO user base
- **For 5D Labs:** Native integration with the best bare-metal provisioning tooling for Talos Linux
- **For Users:** Seamless end-to-end experience from bare-metal provisioning to autonomous AI development

## Partnership Request

We'd love to explore a partnership that includes:

1. **Bare-Metal Credits/Trial Access** — Credits on the Omni platform for cross-pollination, testing, and showcasing the integration in live demos
2. **Technical Collaboration** — Direct communication channels with your engineering team for API guidance and integration best practices
3. **Co-Marketing Opportunities** — Joint announcements, blog posts, and conference presentations highlighting the CTO + Sidero/Omni stack

## Why This Makes Sense

- **Shared Foundation:** CTO is built on Talos Linux — we're already deeply invested in your ecosystem
- **Complementary Solutions:** Sidero Metal handles provisioning, CTO handles autonomous development — together we offer a complete bare-metal AI development platform
- **Open Source Alignment:** Both projects embrace open-source values (CTO is AGPL-3.0, Sidero Metal is open-source, Omni offers on-prem deployments)
- **Cost-Conscious Users:** Our shared audience prioritizes bare-metal deployments for cost savings and data sovereignty

## Next Steps

I'd love to schedule a call to discuss:
1. Technical requirements for deep Sidero Metal / Omni integration
2. Credits or trial access for development and testing
3. Co-marketing opportunities and mutual promotion
4. Potential long-term partnership structure

**Availability:** I'm available for a call anytime this week or next. Feel free to pick a time that works for you: [Calendly link or suggest times]

Looking forward to building something great together!

Best regards,

**Jonathon Fritz**
Founder, 5D Labs
Email: jonathon@5dlabs.ai
GitHub: https://github.com/5dlabs/cto
Website: https://5dlabs.ai
Discord: https://discord.gg/A6yydvjZKY

---

## Follow-Up Strategy

**If no response within 1 week:**
- Reach out via Slack (#omni channel in Talos Community)
- Tag @siderolabs on X/Twitter with a summary of the integration plan
- Open a GitHub Discussion in siderolabs/omni repository

**If positive response:**
- Schedule technical kickoff call
- Create shared Slack channel or Discord bridge
- Begin integration work immediately
- Plan joint announcement timing

## Additional Contact Channels

- **Email:** sales@siderolabs.com, security@siderolabs.com (if no sales response)
- **Slack:** Talos Community Slack (#omni channel)
- **Twitter/X:** @siderolabs
- **GitHub:** https://github.com/siderolabs (open a Discussion)
