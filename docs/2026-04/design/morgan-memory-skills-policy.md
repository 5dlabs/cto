# Morgan Memory and Skills Lifecycle Policy

## Purpose

This document defines how Morgan should use workspace memory, long-term OpenMemory, and remote skills/persona bundles when running through Hermes. It is a design unblocker for the Wave 2B Morgan sidecar/MCP work; it intentionally avoids code changes.

Companion design: `docs/2026-04/design/morgan-hermes-agent.md`.

## Current ground truth

Relevant existing behavior in CTO:

- Hermes writes runtime identity files into `/workspace`: `AGENTS.md`, `SOUL.md`, `IDENTITY.md`, `USER.md`, `TOOLS.md`, and `HEARTBEAT.md`.
- The harness tells agents to use `memory/YYYY-MM-DD.md` for session continuity.
- `CodeRun.spec.projectId` exists and is intended for memory isolation.
- `CodeRun.spec.continueSession` and `CodeRun.spec.overwriteMemory` already exist.
- Remote skills/personas are represented by `CodeRun.spec.skillsUrl` and `CodeRun.spec.skillsProject` in the current tree.
- `crates/controller/src/tasks/code/skills_cache.rs` fetches GitHub Release tarballs, verifies `hashes.txt` hashes, extracts to a controller-side cache, and returns skill/persona/config content for template rendering.
- `templates/_shared/partials/skills-setup.sh.hbs` writes inline skill content into CLI-native skill directories at container startup.
- OpenMemory tools exist through the platform tools surface (`openmemory_openmemory_query`, `openmemory_openmemory_store`, `openmemory_openmemory_list`, `openmemory_openmemory_reinforce` naming varies by tool wrapper). They should be treated as long-term semantic memory, not as a raw event log.

Important observed inconsistency:

- Some docs say remote skill fetch failures should produce empty skills with no baked-in fallback.
- `templates.rs` has both behaviors: filtered `get_agent_skills_enriched()` returns empty skills on remote failure, while `fetch_all_skills_for_coderun()` logs a warning and falls back to mapped skills.
- Wave 2B implementation should choose one policy explicitly before adding Morgan-specific behavior. This document recommends **fail visible, degrade safe**: do not silently substitute a different skill bundle, but allow the CodeRun to proceed with a clear warning and an empty/limited skill set unless the run marks skills as required.

## Memory layers

Morgan should use three distinct memory layers. Do not merge them into one path.

| Layer | Storage | Lifetime | Purpose | Writer |
|---|---|---|---|---|
| Runtime streams | `/workspace/runs/<id>/*.jsonl`, `morgan-status.json` | One CodeRun/session | Coordination, replay, debugging, MCP file fallback. | Morgan sidecar, ACPX/Lobster. |
| Workspace session memory | `/workspace/memory/YYYY-MM-DD.md`, task notes | Repo/PVC lifetime | Human-readable decisions, session notes, handoff details. | ACPX/agent. |
| Long-term semantic memory | OpenMemory via MCP/tools | Cross-session/project | Searchable durable facts, preferences, stable decisions. | ACPX/agent, optionally Morgan sidecar through MCP if configured. |

### Runtime streams are not long-term memory

The Morgan sidecar should write complete event streams for observability, but those streams must not be automatically stored in OpenMemory. Runtime streams may contain meeting URLs, transcripts, participant names, provider errors, and other sensitive details.

Only curated summaries should enter long-term memory.

### Workspace session memory

The harness already documents:

```text
memory/YYYY-MM-DD.md
```

Policy:

- Use this for human-readable continuity within the workspace.
- Include task decisions, implementation findings, validation results, and open questions.
- Do not paste raw transcripts, secrets, signed URLs, OAuth tokens, or provider payloads.
- Link to stream files by relative path when useful instead of copying all events.
- If `overwriteMemory=true`, generated prompts may instruct the agent to start fresh, but the controller should not delete memory files unless a separate destructive-cleanup feature is explicitly approved.

Recommended daily memory entry shape:

```md
# 2026-05-03

## Morgan/Hermes session summary
- CodeRun: <id>
- Project: <projectId>
- Morgan session: <session_id>

## Decisions
- ...

## Validated behavior
- ...

## Follow-ups
- ...
```

### Long-term OpenMemory

Use OpenMemory only for curated, reusable knowledge.

Allowed memory content:

- Stable user/team preferences.
- Architecture decisions that should affect future tasks.
- Validated provider behavior and latency findings, summarized without sensitive payloads.
- Reusable failure patterns and fixes.
- Morgan persona/product constraints that are not already in source-controlled persona files.

Disallowed memory content:

- Raw meeting transcripts unless explicit consent and retention scope are provided.
- Secrets, tokens, signed URLs, private keys, cookies, OAuth refresh tokens.
- Unredacted personal data from meeting participants.
- Full provider request/response payloads with account identifiers.
- Temporary run state that can be read from workspace JSONL files.

Recommended OpenMemory namespace keys:

| Dimension | Source |
|---|---|
| `project_id` | `CodeRun.spec.projectId` or `default`. |
| `agent_id` | `morgan`. |
| `session_id` | `MORGAN_SESSION_ID` only for provenance, not as primary namespace. |
| `task_id` | `CodeRun.spec.taskId` when present. |
| `memory_kind` | `decision`, `preference`, `validation`, `failure_pattern`, `provider_fact`, `handoff`. |

If the memory MCP/tool API supports metadata, include these dimensions as metadata. If not, prefix stored content with a compact header:

```text
[project=sigma-one agent=morgan kind=decision task=123]
Decision: ...
```

## Memory retrieval lifecycle

### At startup

When `continueSession=true`:

1. Read workspace identity files.
2. Read recent `/workspace/memory/*.md` entries relevant to the current date/task if present.
3. Query OpenMemory using project and task terms.
4. Summarize retrieved memories into the agent prompt or first Lobster context step.
5. Do not automatically load raw Morgan event streams into prompt context.

When `continueSession=false`:

- Do not resume prior conversation state.
- It is still acceptable to query stable project memories if the task needs architecture context, but label them as reference memories, not session continuation.

When `overwriteMemory=true`:

- Treat prior session memory as non-authoritative unless it is source-controlled or explicitly retrieved as stable OpenMemory.
- Do not erase OpenMemory or workspace memory without a separate explicit destructive action.

### During the run

- ACPX/Lobster may call OpenMemory query/store tools.
- Morgan sidecar should not write OpenMemory directly in v1 unless it is given an explicit MCP/tool token and a clear retention policy.
- Morgan sidecar emits memory candidates as events instead:

```json
{
  "type": "memory_candidate",
  "payload": {
    "kind": "decision",
    "summary": "Use env-driven Morgan sidecar before typed CRD fields.",
    "sensitivity": "internal",
    "recommended_ttl": "long"
  }
}
```

The main agent decides whether to store the candidate.

### At completion

Before writing `/workspace/.agent_done`, the agent should:

1. Write a concise workspace memory summary if material decisions were made.
2. Store at most a small number of high-value OpenMemory items.
3. Redact URLs/tokens/participant data.
4. Include provenance: project, task, CodeRun, and source doc/file.

For Morgan sessions, the completion summary should include:

- Whether the sidecar started.
- Which MCP tools were called.
- Which provider/fallback path was used.
- Where the event/status files are located.
- Any validation failures.

## Skills and persona lifecycle

## Source of truth

Remote skills/personas should come from the skills/persona repo release assets, currently documented as `5dlabs/cto-agent-personas` in `docs/2026-04/remote-skills-and-personas.md`.

Release layout:

```text
hashes.txt
morgan-default.tar.gz
morgan-<project>.tar.gz
_shared.tar.gz
```

Expected extracted layout:

```text
morgan/
  _persona/
    AGENTS.md
    IDENTITY.md
  _config/
    package-manifest.json       optional
  <skill-name>/
    SKILL.md
_shared/
  _persona/
    SOUL.md
    USER.md
    TOOLS.md
```

### Versioning

For production/reproducible CodeRuns, prefer immutable release tags or asset SHAs over a mutable `latest` release.

Policy levels:

| Level | `skillsUrl` style | Use |
|---|---|---|
| Dev | GitHub `latest` release URL | Fast iteration. |
| Staging | Release tag URL, e.g. `/releases/download/v2026.05.03` | Repeatable validation. |
| Production | Tag + expected SHA manifest captured in deployment config | Auditability. |

The current cache code uses `/releases/download/latest/<asset>` derived from the repo URL. Before production, add support for a pinned release tag or an explicit `skillsRelease`/`skillsVersion` value.

### Skill bundle selection

Controller selection inputs:

```yaml
spec:
  skillsUrl: "https://github.com/5dlabs/cto-agent-personas"
  skillsProject: "sigma-one"
```

Resolution order:

1. Try `morgan-<skillsProject>.tar.gz`.
2. If missing and the run allows fallback, try `morgan-default.tar.gz`.
3. If missing and `skillsRequired=true` in a future typed policy, fail the CodeRun before pod creation.
4. Otherwise run with no remote skills and log a visible warning.

Recommended Wave 2B default: missing remote skills are **non-fatal but visible**. Morgan should still be able to run the sidecar stub and base MCP tools without skills.

### Persona precedence

Persona instructions affect safety and identity, so precedence must be deterministic:

1. Source-controlled task instructions and user prompt.
2. Remote Morgan project persona (`morgan/<project>/_persona`) when configured.
3. Remote Morgan default persona (`morgan/_default/_persona`) as merged by release CI.
4. Shared persona (`_shared/_persona`).
5. Harness-generated operational files.

The current template behavior prepends remote `AGENTS.md` to generated `AGENTS.md` and adds other persona files into the ConfigMap. Keep that behavior, but ensure generated red lines still remain present.

### Skill installation targets

Use CLI-native locations:

| CLI | Target |
|---|---|
| Claude | `$WORK_DIR/.claude/skills/<name>/SKILL.md` |
| Codex | `$WORK_DIR/.codex/skills/<name>/SKILL.md` or current Codex-supported equivalent. |
| Factory | Factory-native skill/config location. |
| OpenCode | OpenCode-native skill/config location. |
| Generic ACPX | Add skill summaries to workspace instructions if no native skill loader exists. |

Hermes-specific policy:

- Morgan sidecar does not consume agent skills directly in v1.
- The main ACPX/CLI agent consumes skills and calls Morgan MCP tools.
- If the sidecar later needs behavior packs, use a separate sidecar config bundle, not the CLI skill directory.

## Morgan-specific skill set

Recommended initial Morgan skills:

| Skill | Purpose |
|---|---|
| `morgan-hermes-sidecar` | How to use Morgan MCP tools and workspace streams. |
| `presence-bridge` | Discord/centralized presence rules and safe outbound messaging. |
| `memory-curation` | What to store in OpenMemory vs workspace notes. |
| `meeting-consent` | Disclosure, consent, recording/transcription boundaries. |
| `avatar-provider-routing` | Provider/fallback selection and status interpretation. |

These skills can live in the remote personas repo and be selected for Morgan default/project bundles.

Minimum `morgan-hermes-sidecar/SKILL.md` content should include:

- MCP tools: `morgan_session_start`, `morgan_session_status`, `morgan_say`, `morgan_set_state`, `morgan_events_tail`, `morgan_session_stop`.
- Stream paths: `MORGAN_EVENT_LOG`, `MORGAN_COMMAND_LOG`, `MORGAN_STATUS_FILE`.
- Rule: do not call Discord APIs directly; use the bridge path.
- Rule: summarize memory candidates before storing.

## Package manifest and tool exposure

`templates.rs` can merge a package manifest from the skills cache into client config. For Morgan, use this to declare optional local/MCP tools once supported.

Example `package-manifest.json` in the remote bundle:

```json
{
  "name": "morgan-default",
  "version": "2026.05.03",
  "mcpServers": {
    "morgan": {
      "type": "http",
      "urlEnv": "MORGAN_MCP_URL",
      "requiredTools": [
        "morgan_session_start",
        "morgan_session_status",
        "morgan_say",
        "morgan_set_state",
        "morgan_events_tail",
        "morgan_session_stop"
      ]
    }
  },
  "memoryPolicy": {
    "defaultNamespace": "project",
    "storeRawTranscripts": false,
    "storeMemoryCandidatesOnly": true
  }
}
```

The sidecar/MCP config should still be generated by the controller. The package manifest is advisory and useful for validation/tool documentation.

## Failure policy

### Remote skills fetch failure

Recommended behavior:

- Log clear warning with repo URL, agent, project, and error class.
- Add a visible note to `TOOLS.md` or startup logs: `Remote Morgan skills unavailable; running with base harness instructions only`.
- Continue in stub/basic mode unless future `skillsRequired=true` is set.
- Do not silently use unrelated baked-in skills.

### Hash mismatch

Hash mismatch means possible tampering or bad release publication.

Recommended behavior:

- Fail skill installation for that bundle.
- Do not use cached content unless the cached hash exactly matches the previous accepted hash and the policy explicitly allows stale cache fallback.
- Emit a Kubernetes event / controller warning.
- For Morgan meeting/customer-facing sessions, fail closed rather than run with unverified behavior packs.

### Missing skill content

If a selected skill has no `SKILL.md`:

- Log the missing skill name.
- Continue unless required by the Morgan bundle manifest.
- Validation should fail for release assets that omit required Morgan skills.

### OpenMemory unavailable

- Continue the run.
- Write workspace memory summary to `memory/YYYY-MM-DD.md`.
- Add a `memory_degraded` event to `morgan-events.jsonl` if Morgan memory integration was requested.

## Safety and retention policy

### Consent

Before Morgan joins or records/transcribes a meeting, the agent must have explicit consent context. Store only the consent outcome and scope, not unnecessary personal details.

Consent record example for workspace memory:

```md
- Consent: host approved Morgan AI participant for this demo session; no raw transcript retention requested.
```

### Redaction

Before storing to OpenMemory, redact:

- API keys, OAuth tokens, cookies.
- Signed LiveKit/meeting URLs.
- Email addresses or participant names unless needed and consented.
- Raw transcript passages not necessary for future task performance.

### Retention classes

| Class | Storage | Example | Retention |
|---|---|---|---|
| `ephemeral` | JSONL streams | Turn events, transient provider errors | Run/PVC cleanup policy. |
| `session` | Workspace memory | Summary of validation run | Repo/PVC lifetime. |
| `durable` | OpenMemory | Architecture decision | Until superseded. |
| `forbidden` | None | Secrets/raw unconsented transcripts | Never store. |

## Controller/harness implementation hooks

### Environment variables

Add these to the main container when Morgan sidecar/memory is enabled:

```text
MORGAN_MEMORY_POLICY=curated
MORGAN_MEMORY_NAMESPACE=<projectId or default>
MORGAN_MEMORY_STORE_RAW_TRANSCRIPTS=false
MORGAN_SKILLS_PROJECT=<skillsProject or default>
MORGAN_SKILLS_URL=<skillsUrl if configured>
```

### Startup prompt additions

When Morgan sidecar is enabled, append a short section to `TOOLS.md`:

```md
## Morgan sidecar

- MCP: `$MORGAN_MCP_URL`
- Status: `$MORGAN_STATUS_FILE`
- Events: `$MORGAN_EVENT_LOG`
- Commands: `$MORGAN_COMMAND_LOG`
- Memory policy: curated summaries only; do not store raw transcripts or signed URLs.
```

### Completion hook

At the end of the Lobster workflow, before final `.agent_done`, add an optional memory summary step for Morgan-enabled runs:

```bash
if [ "${MORGAN_AGENT_ENABLED:-false}" = "true" ]; then
  mkdir -p /workspace/memory
  # Agent/CLI should write the content; shell hook may only ensure the directory exists.
fi
```

Do not auto-generate semantic memories in shell. The reasoning agent must curate them.

## Validation plan

### Skills validation

1. Fetch `hashes.txt` from the configured skills repo release.
2. Confirm it contains `morgan-default.tar.gz` and, when applicable, `morgan-<project>.tar.gz`.
3. Verify tarball SHA256 against `hashes.txt`.
4. Extract to a temp dir and assert:
   - `morgan/_persona/AGENTS.md` exists or the merged persona path documented by release CI exists.
   - Required Morgan skills contain `SKILL.md`.
   - Optional `package-manifest.json` is valid JSON if present.
5. Render a Hermes CodeRun template and assert skill setup logs include required Morgan skills.

### Memory validation

1. Run a Morgan-enabled Hermes CodeRun in stub mode.
2. Confirm `MORGAN_MEMORY_POLICY=curated` is present in main container env.
3. Confirm workspace `memory/` directory is available.
4. Simulate a `memory_candidate` event in `morgan-events.jsonl`.
5. Confirm the agent summary stores only a redacted/curated summary, not the raw event payload.
6. If OpenMemory is unavailable, confirm run still completes and workspace memory summary is written.

### Regression validation

- Non-Morgan Hermes CodeRuns do not get Morgan memory env vars.
- Non-Hermes CodeRuns do not get Morgan sidecar-specific memory hooks.
- Existing `hermes-presence-adapter` `presence-inbox.jsonl` fallback remains unchanged.
- Remote skills failure mode is visible in logs and does not masquerade as successful skill loading.

## Open questions for implementation

1. Should remote skills be pinned by a new `skillsRelease` field, or should `skillsUrl` accept a release URL instead of repo URL?
2. Should `skillsRequired` be a CodeRun field or a package-manifest setting?
3. Which CLIs in the current Hermes path support HTTP MCP directly versus needing a local bridge/wrapper?
4. Should OpenMemory namespace enforcement live in the tools server, the prompt/skill policy, or both?
5. What is the PVC cleanup window for `/workspace/runs/<id>/morgan-events.jsonl` when meeting metadata is present?

## Recommended Wave 2B defaults

- Enable Morgan sidecar per CodeRun via env, not typed CRD fields.
- Use stub provider mode first.
- Use generated MCP config, not only environment discovery.
- Treat remote skills as optional but visibly degraded unless marked required later.
- Store only curated memory summaries in OpenMemory.
- Keep raw runtime streams in workspace, not semantic memory.
- Require consent/disclosure before meeting participation or transcript retention.
