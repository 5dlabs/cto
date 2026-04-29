# Intake Lobster Flow — Sequential Walkthrough

> Worktree: `cto-intake-video` (branch `intake-video`).
> Sources: `intake/workflows/pipeline.lobster.yaml` (orchestrator) and
> `intake/workflows/intake.lobster.yaml` (heavy intake child), plus the
> codebase-analysis / design-deliberation / task-refinement child workflows
> and helper scripts under `intake/scripts/`.
>
> Step IDs below are the literal `- id:` values from the YAML so you can
> grep straight to them.

---

## Top-level: `pipeline.lobster.yaml`

The pipeline is the conductor. It bootstraps the run, gathers context, kicks
off codebase + design analysis in parallel branches, then delegates the
real heavy lifting (PRD parse → task refine → skills/tools resolve → agent
package PR) to the `intake` child workflow.

### Phase 0 — Boot & guardrails
1. **`load-config`** (line 67) — read `intake/config/*.yaml` + repo config.

   **Comment:** I’d like to understand what is in these config files and what is in the repo config.

2. **`preflight`** (94) — runs `intake/scripts/pipeline-preflight.sh`:
   `LINEAR_API_KEY`, `DISCORD_BRIDGE_URL/health`, `LINEAR_BRIDGE_URL/health`,
   `kubectl cluster-info`. Skips are documented in
   `docs/2026-03/intake-local-prereqs.md`.

   **Comment:** What exactly does the preflight script do, beyond those env curls and kubectl? (Confirm full behavior against `intake/scripts/pipeline-preflight.sh` — it also validates tokens: GitHub + at least one LLM path, warns on `DISCORD_BRIDGE_TOKEN` / `STITCH_API_KEY`, probes OpenClaw gateway, and honors escape hatches `INTAKE_PREFLIGHT_SKIP`, `*_KUBECTL_SKIP`, `*_BRIDGES_SKIP`, `*_TOKENS_SKIP`.)

3. **`materialize-prd`** (107) — pull the PRD source into the run workspace.

   **Comment:** How do we know what the PRD source is and where we are pulling it from?

   **Behavior (grep `materialize-prd` in `pipeline.lobster.yaml`):** Inputs are Lobster/workflow variables `prd_path` and/or `prd_content`. If `prd_path` is set, the file is copied to `.intake/run-prd.txt` (path may be workspace-relative or absolute; if absent in the workspace, the step tries `target_repo_local_path/$REL`). If `prd_path` is unset, `prd_content` must be provided and is written to the same file. The step echoes the output path on stdout.

4. **`reset-audio-artifacts`** (140) — clear `say11`-style narration outputs
   from prior runs.

### Phase 1 — Design intake (parallel-ish branch)
5. **`materialize-design-inputs`** (158) — calls `intake-agent design_intake`
   (provider = `stitch` / `auto` / `ingest_plus_stitch`); requires
   `INTAKE_AGENT_BIN` and `STITCH_API_KEY`. Falls back to a stub
   `.intake/design/design-context.json` if the provider is disabled.
6. **`save-design-bundle`** (288) — write the bundle into
   `.tasks/design/` (incl. `component-library.json` and the OSS catalog
   selections used downstream by `generate-storybook.sh`).
7. **`persist-design-metadata`** (361), **`generate-design-variants`** (403),
   **`design-review`** (440), **`save-design-selections`** (473) — variant
   generation (e.g. Stitch mockups) and reviewer-driven down-select.

   **Comment (Stitch / design assets):** Phase 1 looks good end-to-end, but the UI components and mockups from Stitch are not landing in repo-viewable artifacts (HTML or equivalent in git). If we can embed or otherwise include those mockups in the repo (or linked artifacts reviewers can open from the tree), that would close the gap.

### Phase 2 — Pipeline plumbing

**Comment (shipping scope / E2E testing):** We are **not** shipping with Linear on the first version we release; Discord is what we care most about for that cut. Linear-related steps remain in the stack for later — **do not spend time ripping out Linear plumbing while re-running intake E2E.** Treat Linear failures or noise as acceptable for now unless they block Discord.

8. **`notify-pipeline-start`** (712) — Discord/Linear announce.
9. **`register-run`** (742) — record run id in the bridge state store.
10. **`setup-repo`** (762) — clone/checkout the target repo into the
    workspace; also resolves `OWNER/REPO`.
11. **`create-linear-project`** (794) — Linear project scaffolding.
12. **`verify-agent-resolution`** (812) — ensure the **Linear `agentMap`**
    from `create-linear-project` (output of `intake-util sync-linear init` or
    stub) maps every name in the **hardcoded** expected roster
    (`morgan`, `bolt`, `rex`, … — 14 agents). **Does not read `cto-config.json`
    or the PRD** in the current script; if product language still says “PRD vs
    cto-config,” that is out of date until we wire those sources.

    **Comment (cto-config revamp):** Need a solid mental model of which fields
    this step will honor after we revamp `cto-config.json` — today the only
    structured input is `CTO_CREATE_LINEAR_JSON.agentMap`, not repo agent
    definitions.
13. **`ensure-linear-session`** (863), **`session-url-repo`** (929) — Linear
    session bookkeeping.

### Phase 3 — Context build (the “read the world” phase)
14. **`build-infra-context`** (944) — gather cluster/infra facts (k8s
    objects, namespaces, ingress, etc.) into the run’s context bundle.
15. **`discover-tools`** (964) — **the tool registry**:
    - reads `cto-config.json` `agents[*].tools.remote` (split on `_` to
      recover server names),
    - lists in-cluster MCP servers via
      `kubectl get svc -n operators -l app.kubernetes.io/component=mcp-server`,
    - enumerates cluster databases.
    Output is a normalized inventory of *currently available* tools.

    **Comment (review / remediate):** This path still pulls MCP server *names*
    from `cto-config.json` → `agents[*].tools.remote` (split on `_`). Newer work
    centralizes tool/skill truth in the **agents repo** / published packages;
    this step may **conflict** with that — needs review and likely retargeting
    so we are not double-sourcing.
16. **`codebase-analysis`** (992) — delegates to
    `intake/workflows/codebase-analysis.lobster.yaml` to summarize the
    target repo (modules, languages, key files). Feeds capability sizing.

    **Comment:** How do we tell **brownfield vs greenfield**, and are we happy
    with the **pack/index** path (Repomix vs OctoCode fallback)?

    **Behavior (`pipeline` args + child workflow):** Step body runs only when
    `include_codebase` is `true` (pipeline default is `false` — so many runs
    skip this entirely). URL order: `codebase_repository_url` if set, else
    `setup-repo`’s `repository_url`. Child `codebase-analysis.lobster.yaml`
    prefers **Repomix** `pack_remote_repository` via OpenClaw; if Repomix is
    unavailable, falls back to **OctoCode** `githubViewRepoStructure` and
    `githubSearchCode`. There is **no explicit greenfield flag** — practical
    “greenfield” is leaving `include_codebase` off or analyzing an empty/minimal
    tree; the LLM infers maturity from packed content and PRD.
17. **`initial-parse-prd`** (1017) — first LLM pass over the PRD using the
    context bundle from steps 14-16. Produces a draft task tree.
18. **`validate-tasks`** (1106) — schema + sanity gate over the draft.
19. **`extract-decision-points`** (1127) → **`split-decision-points`**
    (1200) — pull out architecture/UX decisions that need deliberation.

### Phase 4 — Deliberation

**Comment (deliberation + video — active work):** Plan to bring an **MCP server
into intake** and experiment (product side: **Scenario** — confirm canonical URL).
We are **not** dropping **MP3 generation** — that stays the intake audio
deliverable. The new track is to **use that MP3 as source** to produce **MP4**
(Morgan as host, **Optimus & Pessimus** and committee as separate characters,
lip-sync / multi-character experiments) — **video from audio is still being
tested**. Ideally one **multi-character scene** in a **single MP3** with
turn-taking through the deliberation so we **do not** manually coordinate
tracks; otherwise separate recordings + **lip sync** (delegate research). For
publishing, aim to ship **video (MP4)** built from the pipeline **MP3**, not
instead of generating the MP3.

20. **`design-deliberation`** (1227) — invokes
    `design-deliberation.lobster.yaml`. Multi-agent debate over the design
    decisions surfaced above.
21. **`deliberation`** (1351) — generic deliberation child for non-design
    decisions.
22. **`notify-task-gen`** (1443).
23. **`quality-gate-deliberation`** (1468) — gate before we burn cycles
    generating per-task scaffolds.

### Phase 5 — Heavy intake delegation
24. **`gateway-health-check`** (1522) — make sure OpenClaw/MCP gateway is up.
25. **`intake`** (1549) — **calls `intake.lobster.yaml`**. This is where
    most of the work happens (see next section).
26. **`verify-intake-output`** (1648).
27. **`quality-gate-intake-output`** (1675).

### Phase 6 — Wire results into the target repo
28. **`sync-to-target-repo`** (1757) — push generated artifacts into the
    target repo as a PR.
29. **`upload-deliberation-release`** (1886) → **`update-pr-with-release`**
    (1905) — attach the deliberation transcript as a release artifact and
    link it from the PR.
30. **`fix-linear-urls-for-target`** (1943), **`sync-github-issues`** (2010),
    **`notify-pipeline-complete`** (2059), **`deregister-run`** (2144).

---

## `intake.lobster.yaml` — the heavy child

Called from pipeline step `intake` (1549). Step ids in this section are
from `intake.lobster.yaml`.

### Phase A — Refine PRD into tasks
1. **`register-run`** (78), **`linear-plan-parsing`** (88).
2. **`parse-prd`** (102) — second pass, deeper. Produces structured task
   spec.
3. **`verify-parse-prd`** (168), **`breakpoint-parse-prd`** (188),
   **`linear-activity-parse-prd`** (197).
4. **`analyze-complexity`** (213) — score each task for effort /
   capabilities required.
5. **`verify-analyze-complexity`** (260), **`breakpoint-analyze-complexity`**
   (268), **`linear-activity-analyze-complexity`** (274).
6. **`review-tasks`** (300) → **`linear-action-refine-tasks`** (306) →
   **`refine-tasks`** (319) — human-in-the-loop / LLM refinement.
7. **`verify-refine-tasks`** (378), **`breakpoint-refine-tasks`** (417),
   **`linear-activity-refine-tasks`** (423), **`linear-plan-generation`**
   (449).

### Phase B — Per-agent scaffold generation
8. **`reset-task-artifacts`** (463).
9. **`generate-scaffolds`** (475) — emit per-task / per-agent skeletons
   under `.tasks/`.
10. **`verify-generate-scaffolds`** (524), **`breakpoint-generate-scaffolds`**
    (537), **`linear-activity-generate-scaffolds`** (543),
    **`linear-action-fan-out-docs`** (553).
11. **`fan-out-docs`** (563) → **`validate-docs`** (657) →
    **`write-docs`** (820) → **`linear-activity-write-docs`** (829).

### Phase C — Skill/Tool discovery (per-agent, *the* feature you flagged)
12. **`search-skills`** (856), **`discover-skills`** (868) — first pass:
    look in the local repo for skills that already match the per-task
    capability descriptions.
13. **`verify-discover-skills`** (898), **`linear-activity-discover-skills`**
    (906).
14. **`generate-tool-manifest`** (916) — build a per-agent draft tool
    manifest based on the discovered skills + the pipeline-level
    `discover-tools` inventory.

### Phase D — Prompts / workflows / scale tasks
15. **`fan-out-prompts`** (942) → **`validate-prompts`** (1071) →
    **`write-prompts`** (1220) → **`linear-activity-write-prompts`** (1229).
16. **`generate-workflows`** (1239) → **`validate-workflows`** (1262) →
    **`write-workflows`** (1296).
17. **`generate-scale-tasks`** (1304).
18. **`generate-security-report`** (1332) →
    **`linear-activity-security-report`** (1366) →
    **`generate-remediation-tasks`** (1377).
19. **`verify-artifact-gates`** (1411), **`verify-folder-structure`** (1517),
    **`update-plan-quality-gate`** (1593),
    **`llm-quality-gate-artifacts`** (1599).

### Phase E — Capability gap math + skill resolution (the part most in flux)
This is where the “search for skills/tools, generate via Context7, fall
back to OS library, compare against the cto-agents repo” story lives.

20. **`inventory-effective-tools`** (1657) — what tools the agent set
    *currently* has access to (cto-config.json + cluster discovery).
21. **`inventory-effective-skills`** (1677) — what skills the agent set
    *currently* has. Pulls `skill-mappings.yaml` from `5dlabs/cto-agents`
    via `gh api repos/5dlabs/cto-agents/contents/skill-mappings.yaml`
    (line 1694) so the comparison is against the **published agent
    package**, not just local files.
22. **`analyze-required-capabilities`** (1704) — LLM analysis of the
    refined task tree → list of capabilities each agent will need.
23. **`compute-capability-gaps`** (1734) — calls
    `intake/scripts/compute-gaps.py`. Diff = `required - effective`.
24. **`query-tools-catalog`** (1756) — **first resolution attempt**:
    - `curl https://context7.com/api/v2/context?libraryId=/5dlabs/cto-agents&query=<cap>&tokens=2000`
      for each unresolved capability (lines 1786-1793),
    - feeds the responses + the unresolved gap list + current inventory
      into an LLM call gated by `intake/schemas/catalog-query-result.schema.json`
      (line 1806) using `intake/prompts/catalog-query-system.md`.
    - This is the **“OS library” lookup**: Context7 is being used as a
      hosted index over the cto-agents repo so we can ask “does an
      existing OS skill/tool already cover capability X?” without
      crawling the repo every run.
25. **`search-and-resolve-gaps`** (1814) — folds Context7 catalog hits
    into the resolved set; only capabilities the catalog could **not**
    answer fall through to web search (lines 1826-1832). Output is a
    merged `{tools[], skills[]}` resolved bundle (1897-1901).
26. **`install-resolved-skills`** (1908) — pull the matched OSS skill
    contents into the per-agent staging dir.
27. **`generate-custom-skills`** (1942) — **Context7-backed custom skill
    generation** for whatever is *still* unresolved:
    - For each remaining capability, fetch up to 4000 tokens of
      Context7 docs (line 1972),
    - LLM-generate a SKILL.md (system + JSON schema-governed output),
    - Write each generated skill to the staging dir for inclusion in the
      agent package (1999).

### Phase F — Package & PR to `5dlabs/cto-agents`
28. **`generate-agent-package`** (2012) — assembles the per-agent package:
    - `<AGENT>/_package/manifest.json` (incl. catalog hash, currently
      stubbed `"pending-generation"` at line 2070),
    - `<AGENT>/_default/<skill>/SKILL.md` for every resolved + generated
      skill,
    - `<AGENT>/_config/` overrides.
29. **`validate-agent-package`** (2088) — schema + lint over the staged
    package.
30. **`commit-agent-package`** (2127) — clones `5dlabs/cto-agents`,
    creates a branch, commits the staged tree, opens a PR via `gh`.
    Inputs include `$generate-custom-skills.stdout` (line 2131) so the
    PR carries the freshly minted skills.
31. **`sync-linear-issues`** (2222), **`linear-plan-commit`** (2231).

### Phase G — Commit pipeline outputs to the *target* repo
32. **`commit-outputs`** (2245) — write `.tasks/` + agent docs into the
    target repo working tree.
33. **`session-url-branch`** (2295), **`create-pr`** (2306) — open the
    target repo PR.
34. **`sync-linear-issues-post-push`** (2384),
    **`verify-delivery-gates`** (2468),
    **`linear-activity-pr-created`** (2511),
    **`session-url-pr`** (2527),
    **`write-handoff-summary`** (2539),
    **`deregister-run`** (2588).

---

## Where “search / Context7 / OS library / cto-agents compare” lives

A single mental model that ties the moving pieces together:

| Concern | Step(s) | Source of truth |
|---|---|---|
| **Currently available tools** | `pipeline.discover-tools` (964) + `intake.inventory-effective-tools` (1657) | `cto-config.json`, in-cluster MCP services, cluster DBs |
| **Currently available skills** | `intake.inventory-effective-skills` (1677) | `5dlabs/cto-agents/skill-mappings.yaml` via `gh api` |
| **What we will need** | `intake.analyze-required-capabilities` (1704) | LLM over refined tasks |
| **Gap math** | `intake.compute-capability-gaps` (1734) | `compute-gaps.py` |
| **OS / catalog lookup** | `intake.query-tools-catalog` (1756) → `search-and-resolve-gaps` (1814) | Context7 index over `/5dlabs/cto-agents` + `intake/prompts/catalog-query-system.md` + `intake/schemas/catalog-query-result.schema.json` |
| **Install matched OSS skills** | `intake.install-resolved-skills` (1908) | staged into per-agent dir |
| **Generate custom skill (last resort)** | `intake.generate-custom-skills` (1942) | Context7 docs + LLM, schema-governed |
| **Compare vs cto-agents repo** | implicit — both `inventory-effective-skills` (read) and `commit-agent-package` (write) target `5dlabs/cto-agents`; the catalog query is *also* scoped to `libraryId=/5dlabs/cto-agents` | — |
| **Publish back** | `intake.commit-agent-package` (2127) | PR to `5dlabs/cto-agents` |

---

## Open / fragile spots worth discussing before adding video features

1. **Catalog hash is stubbed** — `manifest.json` carries
   `"catalog_hash": "pending-generation"` (intake.lobster.yaml:2070). We
   currently can’t prove that two runs resolved the same OSS catalog
   version. If video agents pull large/unstable model SDK skills, this
   matters.
2. **Context7 library scope is hard-coded** to `/5dlabs/cto-agents` for
   both the catalog query (1793) and the custom-skill doc fetch (1972).
   Any new OSS sources for video (e.g. ComfyUI nodes, FFmpeg recipes,
   model provider SDKs) need either a second `libraryId` or a multiplexer.
3. **`skill-mappings.yaml` is fetched from `main` of cto-agents** with no
   pinning (1694). If we open a PR to cto-agents in this same run, the
   mappings used by the next stage may or may not include it depending
   on merge timing.
4. **`generate-tool-manifest` (916) runs *before* the gap math** in
   Phase E — it bases the per-agent draft manifest on whatever
   `discover-tools` returned, not on resolved gaps. Today the resolved
   tools/skills only land via `commit-agent-package`. For the video
   feature we likely want a feedback edge from Phase E back into the
   per-agent prompts.
5. **`.plan/` vs `.prd/` marker reconciliation** is still in flux
   (PR #4822 vs #4836). The doc above uses neutral “PRD” language;
   final video-feature wording should track whichever marker wins.
6. **Co-change reminder** — any change that touches the agent package
   shape (e.g. new top-level video skill category) needs to be reflected
   both in the intake pipeline templates here *and* in the cto-agents
   repo layout/validator. See top-level `AGENTS.md` § *Co-change
   requirements*.

---

*Generated from a read of the YAMLs at this worktree’s HEAD; if you spot
a mismatch with what you remember from a recent merge, that probably
means the spec under discussion is in flight.*
