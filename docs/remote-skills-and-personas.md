# Remote Skills & Agent Persona Distribution

## Overview

Skills and agent persona files are managed in a separate repository
(**[5dlabs/cto-agent-personas](https://github.com/5dlabs/cto-agent-personas)**)
and distributed to agents via per-agent tarballs published as GitHub Release
assets. The controller downloads, verifies, caches, and injects this content
into every CodeRun Job — replacing the previous approach of baking skill files
into the controller Docker image.

## Architecture

```
┌─────────────────────────┐
│  cto-agent-personas     │  ← GitHub repo (skills + persona source of truth)
│                         │
│  rex/_default/          │     Per-agent skills & persona under _default/
│    tool-design/SKILL.md │     (+ optional _projects/{project}/ overrides)
│    _persona/AGENTS.md   │
│  _shared/_default/      │     Shared persona files (SOUL.md, USER.md, etc.)
│    _persona/SOUL.md     │
└──────────┬──────────────┘
           │  CI: merge _shared + agent + project → tar.gz
           ▼
┌─────────────────────────┐
│  GitHub Release (latest) │
│                         │
│  hashes.txt             │  ← SHA256 manifest (one line per tarball)
│  rex-default.tar.gz     │  ← _default skills + persona for Rex
│  rex-test-sandbox.tar.gz│  ← _default merged with test-sandbox overrides
│  blaze-default.tar.gz   │
│  _shared.tar.gz         │  ← shared persona files only
│  ...                    │
└──────────┬──────────────┘
           │  Controller downloads on CodeRun reconciliation
           ▼
┌─────────────────────────┐
│  Controller (PVC cache)  │
│                         │
│  /skills-cache/          │
│    hashes.txt            │  ← cached manifest
│    rex-default.hash      │  ← last-known hash (skip re-download if match)
│    rex/                  │  ← extracted tarball contents
│      tool-design/SKILL.md│
│      _persona/AGENTS.md  │
│      _persona/IDENTITY.md│
└──────────┬──────────────┘
           │  skills_cache::ensure_skills() + get_persona_files()
           ▼
┌─────────────────────────┐
│  Template rendering      │
│  (templates.rs)          │
│                         │
│  Skills: read SKILL.md  │  → embedded inline in rendered bash script
│          content from    │    via skills-setup.sh.hbs heredocs
│          cache           │
│                         │
│  Persona: AGENTS.md     │  → prepended to generated AGENTS.md
│           SOUL.md, etc. │  → added to ConfigMap as separate files
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│  CodeRun Job Pod         │
│                         │
│  /task-files/ (ConfigMap)│  ← AGENTS.md, SOUL.md, USER.md, etc.
│                         │
│  container.sh runs:      │
│    skills-setup.sh.hbs   │  → writes SKILL.md files to CLI native dir
│    task-files.sh.hbs     │  → copies persona files to $CLI_WORK_DIR/
│                         │
│  CLI (Claude/Codex/etc.) │  ← reads AGENTS.md, SOUL.md from work dir
│                         │    + skills from .claude/skills/, .codex/skills/
└─────────────────────────┘
```

## Key Behaviors

### Skills delivery

1. The controller calls `ensure_skills(skills_url, agent_name, project, skill_names)`.
2. It fetches `hashes.txt` from the latest GitHub Release of the skills repo.
3. It compares the remote SHA256 hash for `{agent}-{project}.tar.gz` against
   the cached hash on the PVC. If they match, no download occurs.
4. On a cache miss, it downloads and extracts the tarball to the PVC.
5. It reads `{cache}/{agent}/{skill_name}/SKILL.md` for each requested skill.
6. The skill content is returned as strings and embedded inline into the
   rendered bash script via the `skills-setup.sh.hbs` Handlebars partial.
7. At pod runtime, the bash script writes each skill to the CLI's native
   directory (e.g., `.claude/skills/{name}/SKILL.md`).

**The pod/ConfigMap/container-script mechanism is unchanged.** Only the source
of skill content changed — from baked-in `/app/templates/skills/` to the PVC
cache populated from the remote tarball.

### Persona delivery

1. After skills, the controller calls `get_persona_files(agent_name)`.
2. It reads files from `{cache}/{agent}/_persona/` (AGENTS.md, IDENTITY.md).
3. It also reads shared persona files from `{cache}/_shared/_persona/`
   (SOUL.md, USER.md, TOOLS.md, HEARTBEAT.md, BOOT.md).
4. In `generate_all_templates()`, persona files are injected:
   - **AGENTS.md** is **prepended** to the template-generated AGENTS.md
     (separated by `---`), so persona instructions appear before the
     Handlebars-rendered operating instructions.
   - All other persona files are added directly to the template map.
5. These files become entries in the ConfigMap mounted at `/task-files/`.
6. `task-files.sh.hbs` copies them to `$CLI_WORK_DIR/` (only if not already
   present in the repo, to avoid git conflicts).

### Failure behavior

- **When `skills_url` is set:** remote fetch failures return empty skills
  and log a warning. There is **no silent fallback** to baked-in templates
  (those have been archived). This makes failures visible during testing.
- **When `skills_url` is not set:** the controller attempts to read from the
  baked-in templates directory. Since skill files have been archived, this
  returns empty content — agents run without skills.
- **Hash/extract failures** are propagated as errors.

### Tarball naming convention

| Tarball | Contents |
|---------|----------|
| `{agent}-default.tar.gz` | `_default/` skills + persona for the agent |
| `{agent}-{project}.tar.gz` | `_default/` merged with `_projects/{project}/` overrides |
| `_shared.tar.gz` | Shared persona files only |

The `project` value comes from `spec.skillsProject` on the CodeRun CRD.
When not set, it defaults to `"default"`.

## CRD Fields

```yaml
apiVersion: cto.5dlabs.ai/v1
kind: CodeRun
spec:
  skillsUrl: "https://github.com/5dlabs/cto-agent-personas"  # optional
  skillsProject: "test-sandbox"                                # optional
```

- **`skillsUrl`** — URL of the GitHub repo hosting skills releases. When set,
  the controller downloads tarballs from its latest release. When absent,
  no remote skills are fetched.
- **`skillsProject`** — Project overlay name. Maps to `{agent}-{project}.tar.gz`.
  When absent, defaults to `"default"` → `{agent}-default.tar.gz`.

Both fields are stamped by all PM producers from the intake config
(`SKILLS_REPO` and `SKILLS_PROJECT` env vars).

## Helm Configuration

```yaml
controller:
  skillsCache:
    enabled: false        # opt-in; set true to activate PVC + remote fetch
    storageSize: "1Gi"    # PVC size for the skills cache
```

When `enabled: false` (default), no PVC is created and the controller does not
attempt remote skill fetches — agents run with whatever skills are available
in the baked-in templates directory (currently archived/empty).

## Persona Files Reference

| File | Purpose | Loaded |
|------|---------|--------|
| `AGENTS.md` | Operating instructions, rules, priorities | Every session (prepended to generated) |
| `SOUL.md` | Persona, tone, boundaries | Every session |
| `USER.md` | Who the user is, how to address them | Every session |
| `IDENTITY.md` | Agent name, vibe, emoji | Every session |
| `TOOLS.md` | Local tool notes and conventions | Every session |
| `HEARTBEAT.md` | Tiny checklist for heartbeat runs | Optional |
| `BOOT.md` | Startup checklist on gateway restart | Optional |

## OpenClaw / ACP Agents

OpenClaw dispatches to real CLIs (Claude Code, Codex, etc.) via `acpx --cwd`.
The underlying CLI reads personality files from its working directory.
`openclaw.sh.hbs` copies persona files from `/task-files/` to `$WORK_DIR/`
so they are visible to the delegated CLI.

The ACP protocol itself has no system-prompt field. If persona injection is
needed for the programmatic `run_oneshot_prompt()` path, persona content
should be prepended to the prompt text.

## Testing

To validate the pipeline end-to-end:

1. Ensure `cto-agent-personas` CI has run and produced a release with assets.
2. Set `skillsCache.enabled: true` in Helm values.
3. Set `SKILLS_REPO=https://github.com/5dlabs/cto-agent-personas` in PM config.
4. Create a CodeRun — the controller should download the tarball, cache it,
   and embed skill content in the rendered script.
5. Verify the pod's startup logs show skills loaded from cache (not templates).
6. If the remote fetch fails, the pod should show empty skills (no silent fallback).
