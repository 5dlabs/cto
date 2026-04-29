# Intake Tool Discovery — Deprecation Note

> Worktree: `cto-intake-video` (branch `intake-video`). Companion to
> `docs/2026-04/intake-flow.md` — supersedes the `discover-tools` and
> `codebase-analysis` comments at lines ~99 and ~107 of that doc.

This locks in two decisions we kept rediscovering in intake debugging:
**(1) `cto-config.json` is no longer the source of truth for what tools an
agent gets.** **(2) Repomix is out — OctoCode is the only repo-pack tool we'll
keep wiring up.** Brownfield-vs-greenfield detection itself is a research
track, not a near-term ship item.

## Why this changed

`pipeline.lobster.yaml`'s `discover-tools` step (line 964) still infers MCP
server names from `cto-config.json` → `agents[*].tools.remote` (split on `_`).
That was fine when `cto-config.json` was the only place tool wiring lived,
but it's now stale: the **`5dlabs/cto-agents` repo is the authoritative
source** for both MCP tools (`tools-catalog/tools/`) and agent skills
(`rex/_default/**/SKILL.md`), and the **compressed Context7 catalog built
from that repo's expertise** is what intake actually queries during
provisioning. Reading server names back out of `cto-config.json` double-sources
the truth and silently drifts whenever the agents repo changes — which it
does every time we publish a new tool or skill.

## What's authoritative now

The agents-repo + catalog flow is already wired into the heavy intake child
workflow. Concrete surfaces:

| Layer | Path | Role |
|---|---|---|
| Agents repo (truth) | `5dlabs/cto-agents` → `tools-catalog/tools/`, `rex/_default/**/SKILL.md` | Per-tool MCP server defs (name, prefix, transport, tools) and per-skill `SKILL.md` files keyed to triggers |
| Compressed catalog | `5dlabs/cto-agents:catalog.json` + Context7 index `/5dlabs/cto-agents` | Machine-readable lookup; semantic surface for fuzzy capability matches |
| Skill mappings | `5dlabs/cto-agents:skill-mappings.yaml` (fetched in `intake.lobster.yaml` step `commit-agent-package`, ~line 1691) | Project-side skill assignments |
| Capability registry | `intake/config/capability-registry.yaml` | Curated capability → tool/skill resolutions used before falling back to the catalog |
| Gap computer | `intake/scripts/compute-gaps.py` | Diffs **required capabilities** against effective inventory + registry; emits `resolvable` vs `needs_search` |
| Catalog query (LLM) | `intake/workflows/intake.lobster.yaml` step `query-tools-catalog` (~line 1758) | Calls Context7 (`https://context7.com/api/v2/context?libraryId=/5dlabs/cto-agents&...`) for each unresolved capability; system prompt at `intake/prompts/catalog-query-system.md` |
| Deeper catalog dive | `intake.lobster.yaml` ~line 1972 | Same Context7 endpoint with larger token budget for residual gaps |
| Package commit | `intake.lobster.yaml` ~line 2143 (`AGENTS_REPO="5dlabs/cto-agents"`) | Opens the PR back into `cto-agents` — closing the loop on the same source of truth |

So: capability list → `compute-gaps.py` (registry-backed) → `query-tools-catalog`
(Context7 over `cto-agents`) → optional deeper catalog query → PR back to
`cto-agents`. `cto-config.json` is **not** part of this chain and should not
be re-introduced into it.

## Repomix → OctoCode

`codebase-analysis.lobster.yaml` currently prefers **Repomix
(`pack_remote_repository` via OpenClaw)** and only falls back to **OctoCode
(`githubViewRepoStructure` + `githubSearchCode`)** when Repomix is unavailable.
Drop Repomix. Per user feedback it has been the source of repeated
brownfield-detection problems (truncation, `E2BIG`, partial packs that
mislead the LLM about repo maturity — see commits `fe5a6ef6`, `9c6afc1d`,
`d2accfe1`, `83675b16`, `877d8f3e`). **OctoCode is the only repo-pack tool
we'll evaluate going forward.** The deeper question — how intake reliably
distinguishes brownfield from greenfield, and what signals beyond a pack
should drive that — is **deferred to a research track**, not this cut.
Today's practical proxy (`include_codebase` flag off ⇒ treat as greenfield)
stays.

## Action items (TODO — do not execute now)

- [ ] **Retarget `discover-tools` (`pipeline.lobster.yaml` line 964).**
  Stop parsing `cto-config.json` `agents[*].tools.remote`. Read tool
  inventory from the `cto-agents` catalog (`catalog.json` + Context7 index)
  joined with the live `kubectl get svc -n operators -l app.kubernetes.io/component=mcp-server`
  cluster check. The cluster check stays — it answers "is this server
  actually reachable right now"; the catalog answers "what does this server
  expose."
- [ ] **Audit `cto-config.json` consumers** (`grep -rn cto-config.json`) and
  document which fields are still load-bearing vs which are vestigial. The
  `verify-agent-resolution` step (line 812) already calls out that
  `cto-config.json` isn't read there — make sure no other intake step
  silently depends on the `agents[*].tools.remote` shape.
- [ ] **Remove Repomix from `codebase-analysis.lobster.yaml`.** Make
  OctoCode the only path; delete the "prefer Repomix, fall back to OctoCode"
  branch and the truncation/`E2BIG` workarounds that exist solely to babysit
  Repomix output. Keep the disk-backed step-output pattern from
  `83675b16` — that's general-purpose and still useful.
- [ ] **Update `docs/2026-04/intake-flow.md`** comments at lines ~99 and
  ~107 to point here, so the next reader doesn't re-litigate the
  `cto-config.json` / Repomix questions from scratch.
- [ ] **Open a research-track ticket** for brownfield-vs-greenfield
  detection. Out of scope for the next intake ship; keep `include_codebase`
  as the manual switch in the meantime.
- [ ] **Confirm Context7 `libraryId`** (`/5dlabs/cto-agents`) is the
  published, indexed name and that the catalog is rebuilt on every merge to
  `cto-agents:main`. If not, that's the prerequisite for any of this being
  reliable.

---

Scope note: this memo is about **tool/skill discovery and repo packing
only**. Linear plumbing, Discord, design intake, and the deliberation /
video tracks are unaffected.
