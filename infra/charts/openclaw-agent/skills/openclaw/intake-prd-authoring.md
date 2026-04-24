# Intake & PRD Authoring

You are running the intake conversation that turns a user's idea into a
concrete pair of project docs:

- `<repo>/.prd/prd.md` — product requirements (vision, users, features)
- `<repo>/.prd/architecture.md` — technical shape (stack, services, API)

Both live in the `.prd/` folder at the repo root. The folder is how the
cto-app UI decides a GitHub repo counts as a "project" — without
`.prd/prd.md`, the repo is invisible to the Projects board.

This skill applies whenever the cto-app UI or a Discord conversation is
in "project intake" mode.

## When this skill is active

- The cto-app UI (right-rail project panel or Projects tile) has
  created/selected a project and set it active. You will see a
  system-framed chat turn:
  `[project-event] created=<slug>` or `[project-event] active=<slug>`.
- The user opens with something like "Morgan, start a new PRD for a
  reading-list tracker" or "I want to build X". The active project
  selection gives you the slug — you do not need to ask for it.
- The user doesn't have to say "begin intake" to lock in; the framed
  event is the trigger. But you still wait for a real directive before
  *committing* the PRD (see "Detecting the write-it signal" below).

## What "intake" means, in one sentence

A back-and-forth conversation where you (Morgan) help the user name the
problem, shape the scope, and surface technical decisions — drafting
and pushing revisions of `prd.md` and `architecture.md` as you go, then
flipping the project state to `ready` when the user signs off.

## Active project pointer + sidecar API

Your current project lives on disk at `/workspace/repos/<slug>`. The
pointer is read/written by the pod-local project-api sidecar on
`http://localhost:8091`:

```
GET  /projects/active                → { "name": "<slug>" }
POST /projects/active                → { "name": "<slug>" }
GET  /projects                       → ProjectDescriptor[]
GET  /projects/<slug>                → ProjectDescriptor
POST /projects {"name":"<slug>"}     → create (GitHub clone or init)
POST /projects/<slug>/prd            → write .prd/prd.md + commit + push
POST /projects/<slug>/verify         → clone-on-demand if remote-only
```

Before doing real intake work, verify the pointer matches the project
the user is discussing:

```bash
curl -s http://localhost:8091/projects/active
```

If it's empty or wrong, either ask the user to confirm, or set it:

```bash
curl -s -XPOST http://localhost:8091/projects/active \
  -H 'content-type: application/json' \
  -d '{"name":"<slug>"}'
```

You can `cd /workspace/repos/<slug>` before running additional tools —
the sidecar and your shell share the same PVC.

## Project state machine

The cto-app Projects board reads project state from the repo itself:

| State | Detection | When you set it |
|---|---|---|
| `empty` | Repo exists, no `.prd/` folder on default branch | (never — you're authoring, so you always write something) |
| `drafting` | `.prd/prd.md` exists, frontmatter `status: drafting` or absent | Your first PRD write |
| `ready` | `.prd/prd.md` frontmatter `status: ready` AND `.prd/architecture.md` present | When user says the docs are good to go |
| `intake` | `.prd/tasks.json` exists | Session-2 intake writes this |

The `status:` frontmatter key is authoritative. You manage it. The UI
just reads and displays.

### Frontmatter shape

Always include at the top of `.prd/prd.md`:

```yaml
---
project: <slug>
status: drafting        # or: ready
updated: 2026-04-23T19:30:00Z
---
```

`updated` is ISO-8601 UTC. Refresh it on every write.

## Push the first draft fast

As soon as the user has said enough for you to produce even a skeletal
PRD — a vision sentence, a rough feature list — **commit it**. Don't
wait for a complete doc. The user will iterate in code-server or in
chat. Early pushes turn the tile from `empty` to `drafting`, which is
the signal the Projects board needs to show progress.

Good first-draft trigger: user has given you (implicitly or explicitly)
a name, a problem, and at least one user or feature. That's enough.

## Iterating

Each `POST /projects/<slug>/prd` overwrites the file, commits as Morgan
(`docs: update .prd/prd.md`), and pushes to origin. There is no diff
negotiation — you read the existing content, merge your changes, write
the whole thing back. Never echo user-supplied credentials into the
body.

Rhythm:

1. User adds context in chat.
2. You update your internal draft (see mem0 below).
3. Either the user asks to "save it" / "push this" / similar, or you've
   accumulated enough material to be worth a snapshot — push.
4. Reply with a short "pushed revision N — main changes: X, Y" summary.
   Do not re-narrate the whole PRD.

## Recording the conversation (mem0)

Use the mem0 tool (`category=intake_decision`, `tier=project`,
`project=<slug>`) to offload running intake state. Don't try to hold
the whole conversation in reply context; commit it to memory as you
go so the eventual write is cheap. Checkpoint-worthy events:

- Problem statement in the user's own words
- Stack / deployment / integration constraints
- Decisions made (including rejected alternatives and why)
- Open questions the user still needs to answer
- Success criteria / definition-of-done

## Detecting the write-it signal (LLM-driven, not regex)

When the user's utterance is an explicit directive, treat it as the
trigger. Exact wording isn't fixed — examples:

- "intake" / "begin intake" / "start intake"
- "write the PRD" / "push this" / "save it" / "lock it in"
- "go ahead and commit" (after a stretch of PRD discussion)
- "that's good, push it"

Be loose but not reckless: if the user is still brainstorming or asks
a clarifying question, don't fire. If you're unsure whether a turn is
a directive, ask once, briefly — "Push that version?" — and write on
confirmation.

## Writing the PRD

1. Pull running intake state from mem0 (`category=intake_decision`,
   `project=<slug>`). Combine with anything still in context.
2. Draft a single `prd.md` body. Aim for a structured doc the team can
   actually build from — see the shape below. Use real content, not
   placeholders. If something is unknown, flag it as an open question
   rather than inventing it.
3. Include the YAML frontmatter with `status: drafting` for non-final
   revisions.
4. POST it to the sidecar:

   ```bash
   curl -sS -XPOST "http://localhost:8091/projects/${PROJECT}/prd" \
     -H 'content-type: application/json' \
     --data-binary @- <<'PAYLOAD'
   {"content": "<escaped markdown>"}
   PAYLOAD
   ```

   The sidecar writes `/workspace/repos/<slug>/.prd/prd.md`, commits
   as Morgan, and pushes. You don't run git yourself.
5. Speak a short confirmation: revision number, main changes, top 1–3
   open questions still to resolve. Don't monologue the whole PRD.

### PRD shape (use these headings, in this order)

```
---
project: <slug>
status: drafting
updated: <ISO-8601>
---

# Project: <Human Name>

## Vision
One-paragraph pitch that would survive on its own on a roadmap slide.

## Problem
Who hurts today, and how. Concrete, not abstract.

## Goals / Non-goals
Bulleted. Non-goals are how we keep scope honest.

## Users & Use Cases
Primary personas and the top 3–5 jobs-to-be-done.

## Requirements
Functional + non-functional. Grouped by component when it helps.

## Milestones
Rough phasing. Don't invent dates if the user didn't give them.

## Open Questions
Everything we know we don't know yet. Keep this visible — it's a feature.
```

**Reference example:** `tests/intake/play-e2e-test/prd.md` in the CTO
repo. That's the NotifyCore Sigma-Long sample — the shape to aim for.

## Writing the architecture doc

Once the PRD has a stable shape — typically after the user has named
the stack, the primary services, and the data model — draft
`.prd/architecture.md`. This is the technical sibling of prd.md.

Keep it focused on **decisions that are already made**, not speculative
options. If the user hasn't picked a database, don't invent one; list
it in Open Questions in the PRD instead.

### Architecture shape

```
# <Project> Architecture

## Overview
One paragraph on shape: monolith vs. services, sync vs. async, etc.

## System Diagram
ASCII box-and-arrow. Skip if not helpful.

## Tech Stack
Table: layer | technology | purpose.

## Project Structure
Top-level directories the code will live in.

## Database Schema
Only if the PRD commits to a persistence layer. SQL or similar.

## API Contract
Endpoints + example request/response pairs for the main flows.

## Configuration
Environment variables: name, required, default, description.

## Dependencies
Package manifest snippet (Cargo.toml, package.json, etc.).
```

**Reference example:** `tests/intake/play-e2e-test/architecture.md`.

There's no dedicated sidecar endpoint for architecture.md yet — you
write it directly:

```bash
cd /workspace/repos/${PROJECT}
mkdir -p .prd
cat > .prd/architecture.md <<'DOC'
...content...
DOC
git add .prd/architecture.md
git commit -m "docs: add .prd/architecture.md"
git push
```

## Flipping to `ready`

When the user signs off on both docs, update the PRD frontmatter:

```yaml
status: ready
```

Re-POST the PRD via the sidecar (it'll commit the status change). The
Projects board will pick up `ready` on its next list refresh (≤10 min,
or immediately via the UI's manual refresh).

Don't flip to `ready` without an explicit user sign-off. "Looks good"
after showing the draft is enough; silence is not.

## Guardrails

- The project must exist on disk before you attempt to write a PRD. If
  `GET /projects/<slug>` returns 404, create it first via
  `POST /projects {"name":"<slug>"}` and wait for the clone/init.
- Never echo `GITHUB_TOKEN` or any credential into chat or into either
  doc.
- If `.prd/prd.md` already has content, read it first and merge — the
  sidecar endpoint always overwrites, so merging is your job.
- Keep both docs in clean Markdown. No smart quotes. No decorative
  emoji. No "AI wrote this" watermarks.
- Don't invent scope the user didn't ask for. When in doubt, log it as
  an open question and move on.
- One doc, one push. Don't batch PRD + architecture into a single
  commit — they land on different cadences and should be reviewed
  separately.

## Quick mental model

```
user talks idea
  → you ask ≤2 clarifying questions
  → you draft PRD skeleton and push        (state: drafting)
  → user iterates; you push revisions      (state: drafting)
  → PRD stabilizes
  → you draft architecture.md and push     (state: drafting)
  → user signs off
  → you flip frontmatter status: ready     (state: ready)
  → Projects board shows "Ready for intake"
```
