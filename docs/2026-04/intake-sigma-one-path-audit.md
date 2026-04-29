# Sigma-1 path-handling audit vs. CTO intake pipeline

**Audited repo:** `5dlabs/sigma-1` (cloned to `/Users/edge_kase/5dlabs/_audit-tmp/sigma-audit`, deleted after audit).
**Audited against:** `cto-intake-video` worktree on branch `intake-video`, primarily:
- `intake/workflows/pipeline.lobster.yaml`
- `intake/workflows/intake.lobster.yaml`
- `intake/scripts/{generate-design-md.sh,generate-pr-body.sh,run-quick-intake.sh,design-intake-optimizer-poc.sh,upload-deliberation-release.sh}`
- `docs/2026-04/intake-flow.md` (annotated walkthrough; cross-referenced for §4)

Sigma-1 is purely a *target / input* repo — there is no `intake/` directory, no `cto-config.json`, no `intake-config.*`, and the README does not mention intake or PRD location.

---

## 1. Sigma-1 layout

Top-level (depth 3, abridged):

```
sigma-1/
├── .plan/
│   ├── status.txt                    # 3 bytes — content: "new"
│   ├── prd/
│   │   ├── prd.md                    # 47 KB — primary PRD (Markdown)
│   │   └── prd.txt                   # 53 KB — text variant
│   └── spec/
│       ├── architecture.md           # 63 KB
│       ├── av-product-browser.html   # 1.1 MB — design mockup
│       ├── sigma-1_logo.png
│       ├── assets/
│       │   └── images/               # hundreds of equipment photos
│       └── docs/
│           └── rentalsystem-spec.md
├── .archive/
│   └── .tasks/                       # PRIOR intake outputs, archived (not deleted)
│       ├── audio/
│       ├── design/
│       ├── docs/
│       │   ├── design-brief.md
│       │   ├── security-report.json
│       │   └── task-1/ … task-10/
│       └── tasks/
│           └── tasks.json
├── .github/
├── .gitignore
├── .gitlab-ci.yml
└── README.md
```

Notable absences:
- No `intake/`, no `cto-config.json`, no `intake-config.{yaml,json}`.
- No top-level `tasks/` or `.prd/` — sigma-1 uses `.plan/` exclusively.
- `README.md` does not document the `.plan/` convention or how to invoke intake against this repo.

Notable presences:
- **`.plan/status.txt`** with the literal value `new` — a state marker the repo evidently uses, but the pipeline never reads or writes it (see §4).
- **`.archive/.tasks/`** — a prior intake run was archived (moved aside) rather than deleted. Its shape exactly matches what the current pipeline emits, which is a strong positive signal that *output* conventions are stable.

---

## 2. Expected vs. actual paths

### Inputs

| Concern | Pipeline expects | Sigma-1 actually has | File:line |
|---|---|---|---|
| PRD source | Caller passes `prd_path` (workspace-relative or absolute) **or** `prd_content` (raw string). No autodetect. | `.plan/prd/prd.md` (and a `.txt` variant). Caller must know to pass `prd_path: .plan/prd/prd.md`. | `pipeline.lobster.yaml:9-11`, `:107-141` |
| PRD copy destination | `$WORKSPACE/.intake/run-prd.txt` (then `.intake/run-prd-for-intake.txt` between pipeline and intake child). | n/a — written into CTO workspace, not target repo. | `pipeline.lobster.yaml:113`, `intake.lobster.yaml:112` |
| Multi-repo fallback | If `prd_path` not found in `$WORKSPACE`, retry under `${target_repo_local_path}/$REL`. | This is exactly the path sigma-1 needs. Works, but is undocumented and is itself a regression-risk surface (§4). | `pipeline.lobster.yaml:122-125` |
| Architecture / spec docs | **No slot.** Pipeline ingests only the PRD text. | `.plan/spec/architecture.md` (63 KB) and `.plan/spec/docs/rentalsystem-spec.md` exist and are clearly load-bearing context. Currently invisible to `parse-prd` / `analyze-complexity`. | (no path — gap) |
| Local design assets | Design discovery only scrapes URLs from PRD text. | `.plan/spec/av-product-browser.html` (1.1 MB mockup) and `.plan/spec/assets/images/` are the primary design references. None are URLs. → design-intake will return effectively empty. | `pipeline.lobster.yaml:~237` (materialize-design-inputs) |
| Project-side config | `load-config` reads `cto-config.json` from the **CTO** workspace, not from the target. | Sigma-1 having no `cto-config.json` is fine; not a gap. | `pipeline.lobster.yaml:67-92` |
| State/marker file | None. | `.plan/status.txt` = `new`. Sigma-1 evidently expects something to read/transition this. | (no path — gap) |

### Outputs

The pipeline writes its results into the **target repo's working tree** under `.tasks/`. The schema is hard-coded by `verify-folder-structure` and the file-list verification in `intake.lobster.yaml`.

| Output area | Pipeline writes | Sigma-1 archive at `.archive/.tasks/` | File:line |
|---|---|---|---|
| Tasks JSON | `.tasks/tasks/tasks.json` (mandatory) | `.archive/.tasks/tasks/tasks.json` ✅ | `intake.lobster.yaml:1411-1581` (verify), `:1530` (folder structure) |
| Per-task docs | `.tasks/docs/task-N/…` | `.archive/.tasks/docs/task-1..task-10/` ✅ | `intake.lobster.yaml:1494-1551` |
| Cross-cutting docs | `.tasks/docs/{design-brief.md, security-report.json, remediation-tasks.json, scale-tasks.json}` plus prompts/workflows | All present in archive ✅ | `intake.lobster.yaml:467, 1494-1551` |
| Design artifacts | `.tasks/design/{component-library.json, design-context.json, manifest.json, selections.json, snapshot-links.{json,md}, source-screenshots.json, stitch/, DESIGN.md}` | All present ✅ | `pipeline.lobster.yaml:295, 480-704`; `intake/scripts/generate-design-md.sh:9-15` |
| Audio | `.tasks/audio/{architecture-deliberation,design-deliberation}.mp3` plus `*.transcript.json` | Present ✅ | `pipeline.lobster.yaml:146-156` |
| Workflows | `.tasks/workflows/` | Present ✅ | `intake.lobster.yaml:1302` |

**Net:** output schema lines up cleanly with what sigma-1 archived. Input schema is where the gaps are.

---

## 3. Required changes

### 3a. Caller-side (no repo changes needed; required for any sigma-1 run today)

The pipeline can be run against sigma-1 as-is provided the caller passes:

```yaml
prd_path: .plan/prd/prd.md
target_repo_local_path: <abs path to sigma-1 checkout>
```

Without `target_repo_local_path`, the workspace-relative lookup at `pipeline.lobster.yaml:118` fails and the fallback at `:122-125` is the only thing that resolves the file.

### 3b. Pipeline-side gaps (recommended)

1. **Document the `.plan/` convention as the supported layout for target repos.** Add a comment block to the `prd_path` parameter docstring at `pipeline.lobster.yaml:9-11` showing `.plan/prd/prd.md` and (optionally) `.plan/prd/prd.txt` as the canonical sigma-style locations. This is purely surgical, low-risk, and unblocks future callers without requiring autodetect logic.

2. **Ingest `.plan/spec/` as supplemental context.** `parse-prd` and `analyze-complexity` (driven from `intake.lobster.yaml:467+`) currently see only the materialized `run-prd-for-intake.txt`. For a repo like sigma-1, `.plan/spec/architecture.md` and `.plan/spec/docs/*.md` are required context. Suggested approach: extend `materialize-prd` (`pipeline.lobster.yaml:107-141`) so that when `prd_path` resolves under `.plan/prd/`, sibling files under `.plan/spec/*.md` are concatenated (with a `\n\n## Supplemental: <relpath>\n\n` header) into a second artifact, e.g. `.intake/run-spec-context.txt`, and surfaced to downstream stages.

3. **Surface local design references to design-intake.** `materialize-design-inputs` (`pipeline.lobster.yaml:~237`) only scrapes URLs from the PRD. Add a step that walks `.plan/spec/` (or, more generally, a list configurable via `design_local_globs`) and writes `{path, mime, sha256}` entries to a manifest the design step can read alongside `source-screenshots.json`. Without this, sigma-1's primary mockup (`.plan/spec/av-product-browser.html`) and its asset library are entirely invisible to design-intake.

4. **Read/write `.plan/status.txt`.** Sigma-1 keeps a literal `new` token here. Pipeline should at minimum read it (so a `done` value can short-circuit a re-run), and should ideally update it on successful completion (`new` → `intaking` → `intaken`). This also gives us a sane place to land the `.plan/` vs `.prd/` marker reconciliation called out in `docs/2026-04/intake-flow.md` §"Open / fragile spots" item 5.

5. **(Optional, lower priority) Archive rather than delete on re-run.** `reset-task-artifacts` (`intake.lobster.yaml:467-473`) wipes top-level `.tasks/{docs,tasks}` subdirs. Sigma-1 already demonstrates the desired behavior: prior outputs were *moved* to `.archive/.tasks/`, not deleted. Consider promoting that pattern (timestamped archive directory) into the pipeline so re-runs are non-destructive by default.

---

## 4. Regression / risk notes

1. **`prd_path` workspace-vs-target-repo fallback (`pipeline.lobster.yaml:122-125`)** is a previously-bitten regression surface — the inline comment ("e.g. prd.md lives in cto-pay, not cto") says as much. Today it is the *only* thing that makes sigma-1 work. Two specific risks:
   - If `target_repo_local_path` is unset, the fallback is silently skipped and the only error is a generic "missing PRD file" at `:127`.
   - The fallback only handles the PRD itself. Any future supplemental-input feature (§3b items 2 and 3) needs to consciously decide whether it resolves under `$WORKSPACE` or `${target_repo_local_path}` and apply the same fallback consistently. Easy to forget.

2. **`.plan/` vs. `.prd/` marker still in flux** per `docs/2026-04/intake-flow.md` §"Open / fragile spots" item 5 (refs PRs `#4822` / `#4836`). Sigma-1 commits to `.plan/`. Any pipeline change that hard-codes one or the other will break the other. §3b item 4 (read/write `status.txt`) should be implemented behind a small resolver that accepts both `.plan/` and `.prd/`.

3. **`reset-task-artifacts` is destructive.** If a future caller pre-stages files into `.tasks/` thinking they will be merged, intake will wipe `.tasks/{docs,tasks}` (`intake.lobster.yaml:467-473`). Sigma-1 sidesteps this by archiving to `.archive/.tasks/`; the pipeline does not. Worth either documenting loudly or fixing per §3b item 5.

4. **Design URL-only scrape misses local files.** `materialize-design-inputs` scraping URLs from PRD text (`pipeline.lobster.yaml:~237`) silently degrades when a target repo's design reference set is local files. There is no warning / log line that says "found 0 design URLs in PRD"; the design pipeline just produces empty manifests. For sigma-1 this is the most visible behavior gap.

5. **Output schema is hard-coded in two places.** `intake.lobster.yaml` `verify-folder-structure` and `verify-artifact-gates` (`:1411-1581`) plus `intake/scripts/generate-design-md.sh:9-15` and `generate-pr-body.sh` all independently encode `.tasks/{tasks,docs,design,audio,workflows}` paths. If we ever rename the top-level output directory (e.g. `.tasks/` → `.intake-out/`), at least three call sites must move together. Not a sigma-1 issue today, but a latent regression surface visible from this audit.

---

## Appendix: how this audit was performed

- Cloned `5dlabs/sigma-1` to `/Users/edge_kase/5dlabs/_audit-tmp/sigma-audit` (sibling tmp dir; not `/tmp`, per system constraint). Deleted after the audit.
- Walked layout to depth 3, sized files, checked for `intake/` / `cto-config.json` / README intake docs.
- In `cto-intake-video`, grepped `intake/workflows/{pipeline,intake}.lobster.yaml` and `intake/scripts/*.sh` for every literal that contains `prd`, `.intake`, `.tasks`, `.plan`, `.prd`, `target_repo`, or `WORKSPACE`. Cross-referenced the user-annotated `docs/2026-04/intake-flow.md`.
- Doc-only audit: no Rust, no `cargo` runs, no pushes. Optional fix commit (if any) is purely documentation against `pipeline.lobster.yaml`.
