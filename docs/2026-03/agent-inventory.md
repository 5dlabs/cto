# CTO Agent Inventory

This page tracks the current agent roster and where each agent's behavior is defined.

## Source of truth

- Roster and human-facing role names: `AGENTS.md`
- Runtime agent config (CLI/model/tools and partial skills): `cto-config.json`
- Skill bundles by run type (`coder`, `healer`, etc.): `templates/skills/skill-mappings.yaml`

## Current inventory (2026-03-21)

| Agent | In AGENTS.md | In cto-config.json (`agents`) | In skill-mappings.yaml | Notes |
|---|---|---|---|---|
| Morgan | yes | yes (`morgan`) | yes (`morgan`) | Intake/PRD agent with explicit config + mapped skills. |
| Atlas | yes | yes (`atlas`) | yes (`atlas`) | Merge gate agent with explicit config + mapped skills. |
| Stitch | yes | yes (`stitch`) | yes (`stitch`) | Review agent with explicit config + mapped skills. |
| Rex | yes | yes (`rex`) | yes (`rex`) | Rust implementation agent. |
| Blaze | yes | yes (`blaze`) | yes (`blaze`) | Frontend implementation agent. |
| Grizz | yes | yes (`grizz`) | yes (`grizz`) | Go implementation agent. |
| Tess | yes | yes (`tess`) | yes (`tess`) | Testing agent; skills currently sourced from mapping file. |
| Cleo | yes | yes (`cleo`) | yes (`cleo`) | Code quality agent. |
| Cipher | yes | yes (`cipher`) | yes (`cipher`) | Security agent. |
| Healer | yes | no | no | Roster-only umbrella role; no standalone runtime config key. |
| Bolt | yes | yes (`bolt`) | yes (`bolt`) | DevOps/infrastructure agent with expanded tool set. |
| Angie | yes | yes (`angie`) | yes (`angie`) | Agent architecture agent; now fully represented in config/mappings. |
| Keeper | yes | no | no | Roster-only operations role; no standalone runtime config key. |
| Nova | yes | yes (`nova`) | yes (`nova`) | AGENTS role is research/docs; runtime key exists and is active. |
| Spark | yes | yes (`spark`) | yes (`spark`) | Rapid prototyping agent with active runtime key. |
| Tap | yes | yes (`tap`) | yes (`tap`) | AGENTS role is integration/webhooks; runtime key exists and is active. |
| Vex | yes | yes (`vex`) | yes (`vex`) | Debugging agent now present in both config and mappings. |
| Block | yes | yes (`block`) | no | Blockchain agent (Solana, EVM, cross-chain, DeFi). Full runtime config with 6 blockchain skills. |
| Pixel | yes | yes (`pixel`) | no | Desktop app agent with runtime config; no skill-mappings entry yet. |

## Tool families by configured runtime agent

Derived from `cto-config.json` `agents.*.tools.remote` prefixes.

| Agent key | Tool families |
|---|---|
| `morgan` | `context7`, `firecrawl`, `octocode`, `openmemory`, `repomix` |
| `rex` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `cleo` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `tess` | `context7`, `firecrawl`, `github`, `kubernetes`, `octocode`, `openmemory` |
| `blaze` | `ai`, `context7`, `firecrawl`, `github`, `octocode`, `openmemory`, `shadcn` |
| `cipher` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `grizz` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `nova` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `pixel` | `context7`, `firecrawl`, `github`, `linear`, `nano`, `octocode`, `openmemory` |
| `tap` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `spark` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `bolt` | `argo`, `argocd`, `context7`, `firecrawl`, `github`, `grafana`, `kubernetes`, `loki`, `octocode`, `openmemory`, `prometheus`, `terraform` |
| `block` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `atlas` | `context7`, `github`, `octocode`, `openmemory` |
| `stitch` | `context7`, `github`, `octocode`, `openmemory` |
| `vex` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |
| `angie` | `context7`, `firecrawl`, `github`, `octocode`, `openmemory` |

## Drift and gaps to resolve

- `Pixel` exists in `AGENTS.md` and `cto-config.json`, but is missing from `templates/skills/skill-mappings.yaml`.
- `Healer` and `Keeper` are roster entries but do not have standalone runtime keys in `cto-config.json`.
- Some agents in `cto-config.json` have minimal/no `skills` sections and depend on `templates/skills/skill-mappings.yaml` for job-type skill composition.

## Quick verification commands

```bash
# Runtime agent keys
jq -r '.agents | keys[]' cto-config.json

# Skill-mapping agent keys
awk '/^[a-z][a-z0-9-]*:$/ {gsub(":","",$1); print $1}' templates/skills/skill-mappings.yaml

# Roster names
rg '^\| \*\*' AGENTS.md
```
