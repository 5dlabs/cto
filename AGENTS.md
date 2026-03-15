# CTO Platform Agents

## Agent Roster

| Agent | Role | Specialty |
|-------|------|-----------|
| **Morgan** | Intake & PRD Processing | Task decomposition, agent assignment |
| **Atlas** | Merge Gate | PR merging, branch management |
| **Stitch** | Code Review | Automated PR review, quality checks |
| **Rex** | Rust Implementation | Backend systems, CLI tools |
| **Blaze** | Frontend Implementation | React, TypeScript, UI components |
| **Grizz** | Go Implementation | Go services, infrastructure tooling |
| **Tess** | Testing | Test strategy, coverage analysis |
| **Cleo** | Code Quality | Linting, standards enforcement |
| **Cipher** | Security | Security audits, vulnerability scanning |
| **Healer** | Self-Healing | Failure detection, automated remediation |
| **Bolt** | DevOps | Infrastructure, Helm, Kubernetes |
| **Angie** | Agent Architecture | OpenClaw-first agent systems and orchestration |
| **Keeper** | Operations | Cluster maintenance, monitoring |
| **Nova** | Research | Web research, documentation |
| **Spark** | Rapid Prototyping | Quick iterations, experiments |
| **Tap** | Integration | API integration, webhooks |
| **Vex** | Debugging | Root cause analysis, troubleshooting |
| **Pixel** | Desktop App | CTO Lite Tauri app |

## Configuration

- Agent configs: `cto-config.json` (models, tools, skills per agent)
- Deployment: `infra/charts/cto/` (Helm values)
- Agent expertise docs: `.codex/agents/`
- Skill mappings: `templates/skills/skill-mappings.yaml`

## Tools & Skills

All agents have access to the tools and skills documented in [TOOLS.md](TOOLS.md).

Per-agent tool assignments are defined in `cto-config.json`. See `docs/agent-inventory.md` for the full breakdown of which agent gets which tools.
