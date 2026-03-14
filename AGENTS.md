# CTO Platform Agents

This file documents the AI agents deployed as part of the Cognitive Task Orchestrator platform.

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

## Configuration

Agent configurations are defined in `infra/charts/cto/` via Helm values. Each agent runs as an OpenClaw container with its own identity, skills, and tool permissions.

See the [OpenClaw Helm chart](https://github.com/5dlabs/openclaw-helm) for agent deployment configuration.
