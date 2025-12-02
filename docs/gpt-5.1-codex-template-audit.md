# GPT-5.1-Codex Template Refactor Notes

## Snapshot of the Current Template Surface
- `templates/code/*` hosts provider-specific agent prompts, configs, and container scripts for Claude, Codex, Cursor, Factory, Gemini, OpenCode, plus integration hooks; most files embed near-identical playbooks with provider tweaks.
- `templates/agents/*`, `templates/remediate/*`, `templates/review/*`, `templates/heal/*`, and `templates/pm/*` contain additional agent personas that duplicate GitHub workflow guidance with only minor wording changes.
- `templates/shared/*` currently exposes just four shared assets (`design-system.md`, `context7-instructions.md.hbs`, `memory-functions.sh`, `task-setup-functions.sh`), leaving most duplication unmanaged.

## Modularity & Inheritance Opportunities
1. **Unify provider-specific agent prompts.** The Claude and Codex Blaze prompts differ only on a couple of lines (testing stack, state library) yet the full files are copy-pasted and maintained separately. Replacing the provider directories with data-driven overrides (e.g., Handlebars partials fed via JSON) would let us author the core UX playbook once and inject the small differences per model.
```26:35:templates/code/claude/agents-blaze.md.hbs
- **Styling**: Tailwind CSS 4+ ONLY - NO Material-UI, NO CSS-in-JS, NO styled-components
- **State**: XState (Stately) for complex flows / Zustand for simple state
- **Testing**: Jest + React Testing Library + Playwright
```
```26:35:templates/code/codex/agents-blaze.md.hbs
- **Styling**: Tailwind CSS 4+
- **State**: React Context / Zustand (as needed)
- **Testing**: Vitest + React Testing Library
```
2. **Centralize Context7 & design-system instructions.** We already wrote a comprehensive Context7 partial in `templates/shared/context7-instructions.md.hbs`, but none of the agent prompts include it yet; each file rolls its own documentation guidance. Similarly, the `design-system.md` content is referenced by path rather than an include, so updating tone/links requires editing dozens of files. Extracting common sections into shared partials keeps production docs consistent.
```1:44:templates/shared/context7-instructions.md.hbs
## 📚 Context7 Documentation Tools
... (pre-resolved IDs for both Rust and frontend stacks) ...
```
3. **Share container shell logic.** Every provider’s `container-base.sh.hbs` implements the same label creation, diff validation, fallback PR creation, and GitHub Project linking logic. These 200+ line blocks only vary in copy (“Codex agent” vs “Cursor agent”) yet are repeated verbatim, which makes bug fixes risky. We could extract those functions into a sourced script (similar to `shared/task-setup-functions.sh`) and expose provider-specific hooks.
```640:760:templates/code/cursor/container-base.sh.hbs
case "$label" in
  task-*) COLOR="28a745" ...
esac
...
if gh pr create "${LABEL_ARGS[@]}"; then
  echo "✅ Auto-created pull request for $CURRENT_BRANCH"
  ... link PR to project ...
fi
```
```760:840:templates/code/codex/container-base.sh.hbs
case "$label" in
  task-*) COLOR="28a745" ...
esac
...
if gh pr create "${LABEL_ARGS[@]}"; then
  echo "✅ Auto-created pull request for $CURRENT_BRANCH"
  ... link PR to project ...
fi
```
4. **Refine specialized agents (Heal, Remediate, Review) via shared workflows.** The remediation prompts for Claude vs Factory and the Heal instructions for Claude copy entire sections (worktree setup, CI steps, GitHub commands). Moving the common remediation workflow into `templates/agents/system-prompt.md.hbs` or another shared partial would reduce drift and let us inject only the model-specific caveats.
```1:64:templates/remediate/claude/agents.md.hbs
# Claude Project Memory — Rex PR Remediation Agent
... identical “Remediation Workflow” + cargo commands ...
```
```1:84:templates/remediate/factory/agents.md.hbs
# Factory Project Memory — Rex PR Remediation Agent
... same workflow with different introductory sentence ...
```

## Anti-Patterns, Redundancies & Accuracy Gaps
1. **Inconsistent tech stack instructions across providers.** Blaze on Claude mandates Jest + Playwright and forbids CSS-in-JS, while Blaze on Codex lists Vitest and omits the “no Material UI” warning. Unless the runtime truly differs, this inconsistency will confuse open-source contributors who switch models mid-task (see snippets above).
2. **Hard-coded internal paths.** Multiple prompts (e.g., Claude Blaze) direct agents to `/workspace/docs/ui/apps/v4/...` for shadcn components, but that folder only exists in our internal clusters. Open-source users won’t have those assets mounted, so the instructions become broken links.
```43:52:templates/code/claude/agents-blaze.md.hbs
- **shadcn/ui Component Library**: `/workspace/docs/ui/apps/v4/content/docs/components/`
- **Component Examples**: `/workspace/docs/ui/apps/v4/registry/`
```
3. **Heal agents re-clone repositories.** `templates/heal/claude/agents.md.hbs` instructs the remediation agent to run a fresh shallow clone and git worktree setup even though the controller already injects the repo into the workspace PVC. This double-clone is fragile (extra network pulls, duplicate repos under `/workspace`).
```28:54:templates/heal/claude/agents.md.hbs
if [ ! -d "${REPO_PATH}" ]; then
    git clone --depth 1 {{repository_url}}.git "${REPO_PATH}"
fi
...
git worktree add "${WORKTREE_PATH}" origin/main
```
4. **Deprecated intake script still ships.** `templates/intake/intake.sh` is flagged as deprecated but remains executable. Users browsing the repo may run it instead of `unified-intake.sh.hbs`, leading to missing Firecrawl/context enrichment.
```1:10:templates/intake/intake.sh
# DEPRECATED: This script is deprecated in favor of unified-intake.sh.hbs
# This file will be removed in a future release.
```
5. **Context7 partial is unused.** Even though the shared partial exists, every prompt re-describes the same two-step workflow manually. We should either wire up `{{> shared/context7-instructions}}` or remove the partial to avoid dead files.
6. **Provider containers assume GitHub CLI/project access.** The fallback PR logic hard-codes `gh project item-add ... --owner 5dlabs` which won’t succeed for community forks. Making this optional (e.g., detect org/project first) would prevent noisy failures.
```700:755:templates/code/cursor/container-base.sh.hbs
if gh pr create ...
  gh project item-add "$PROJECT_NUMBER" --owner 5dlabs --url "https://github.com/$REPO_SLUG/pull/$PR_NUMBER"
```

## Proposed Refactor Path
1. **Introduce template data & partials.** Define a YAML/JSON manifest per provider (test runner, state library, CLI quirks) and render prompts via a shared Handlebars layout. Start with Blaze, Rex, Tess, Cleo to validate the approach, then roll out to heal/remediate/review agents.
2. **Modularize shell logic.** Extract shared Git/GitHub helpers (labeling, validation, PR fallback, project linking) into `templates/shared/container-functions.sh` and have each `container-base.sh.hbs` source it. Keep provider differences (e.g., CLI names) as env vars or template placeholders.
3. **Replace hard-coded docs paths with configurable mounts.** Reference `design-system.md` and shadcn docs through helper functions that verify availability (the helpers already exist in `shared/task-setup-functions.sh`). Provide a graceful fallback when the internal `/workspace/docs` tree isn’t present.
4. **Retire or rewire deprecated scripts.** Remove `templates/intake/intake.sh` (or make it exec `unified-intake.sh.hbs` immediately) to avoid drift, and document the preferred intake path in README.
5. **Align Context7 usage.** Include the shared partial everywhere and delete bespoke text blocks. This ensures all agents see the same instructions and future updates happen in one place.

With these steps we can dramatically shrink the template surface, cut copy-paste errors, and make the open-source release easier to audit.