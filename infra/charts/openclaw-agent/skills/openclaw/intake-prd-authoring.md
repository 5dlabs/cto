# Intake & PRD Authoring

You are running the intake conversation that turns a user's idea into a
concrete Product Requirements Document (`prd.md`) at the root of the
active project's repo. This skill applies any time the cto-app UI or a
Discord conversation is in "project intake" mode.

## When this skill is active

- The cto-app UI (right-rail project panel) has created a new project and
  set it active. You will see a system-framed chat turn that looks like:
  `[project-event] created=<slug>` or `[project-event] active=<slug>`.
- The user starts talking about a problem, product, or initiative and
  hasn't yet said "begin intake" / "start intake" / "intake" to lock it
  in.

## What "intake" means, in one sentence

A back-and-forth conversation where you (Morgan) help the user name the
problem, shape the scope, and surface decisions — then, when the user
says so, you commit the result to `<repo>/prd.md`.

## The active project pointer

Your current project lives on disk at `/workspace/repos/<slug>`. The
pointer is read/written by the pod-local project-api sidecar:

```
GET  http://localhost:8091/projects/active      → { "name": "<slug>" }
POST http://localhost:8091/projects/active      → { "name": "<slug>" }
GET  http://localhost:8091/projects             → ProjectDescriptor[]
GET  http://localhost:8091/projects/<slug>      → ProjectDescriptor
POST http://localhost:8091/projects             → create (clone or init)
POST http://localhost:8091/projects/<slug>/prd  → write prd.md + commit
```

Before doing any real intake work, verify the pointer matches the
project the user is talking about:

```bash
curl -s http://localhost:8091/projects/active
```

If it is empty or wrong, either ask the user to confirm which project
this is for, or set it yourself:

```bash
curl -s -XPOST http://localhost:8091/projects/active \
  -H 'content-type: application/json' \
  -d '{"name":"<slug>"}'
```

You can always `cd /workspace/repos/<slug>` before running additional
tools — the project-api and your own shell share the same PVC.

## Recording the conversation (mem0)

Use the mem0 tool (category `intake_decision`, tier `project`, project =
slug) to offload the running state of the intake. Do **not** try to
represent the whole conversation in your reply context; commit it to
memory as you go so the eventual PRD write is cheap. Good checkpoints to
capture:

- Problem statement in the user's own words
- Concrete constraints (stack, deadlines, integrations)
- Decisions made (including rejected alternatives and why)
- Open questions the user still needs to answer
- Success criteria / "done" definition

## Detecting the "write it" signal (LLM-driven, not regex)

When the user's utterance is an explicit directive to commit the PRD,
treat it as the trigger. The exact wording is not fixed — the user may
say any of:

- "intake"
- "begin intake" / "start intake" / "do the intake"
- "write the PRD" / "record the PRD" / "lock it in"
- "go ahead and save it" (after a stretch of PRD discussion)

Be loose but not reckless: if the user is still brainstorming or asks a
clarifying question, do not fire. If you're uncertain whether the
utterance is a directive, ask once, briefly — e.g. "Commit the PRD as
it stands?" — and only write on confirmation.

Do **not** wait for a literal regex match. Use your own judgment.

## Writing the PRD

1. Pull the running intake state from mem0 (category `intake_decision`,
   filtered by `project=<slug>`). Combine with anything still in context.
2. Draft a single `prd.md` body in memory. Aim for a structured doc the
   team can actually build from — see the shape below. Use real content,
   not placeholders. If something is unknown, flag it as an open
   question rather than inventing it.
3. POST it to the sidecar:

   ```bash
   curl -sS -XPOST "http://localhost:8091/projects/${PROJECT}/prd" \
     -H 'content-type: application/json' \
     --data-binary @- <<'PAYLOAD'
   {"content": "<escaped markdown>"}
   PAYLOAD
   ```

   The sidecar writes `/workspace/repos/<slug>/prd.md` and commits it
   with `docs: write prd.md`. You do not need to run git yourself.
4. Speak a short confirmation: what you wrote, where it lives, and the
   top 1–3 open questions still to resolve. Don't monologue the whole
   PRD back to the user.

### PRD shape (use these headings, in this order)

```
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

## Architecture Sketch
Only as much as we've actually decided. No speculative microservices.
ASCII diagrams are fine when they clarify.

## Milestones
Rough phasing. Don't invent dates if the user didn't give them.

## Open Questions
Everything we know we don't know yet. Keep this visible — it's a feature.
```

Examples of this shape committed to repos in this workspace:

- `tests/intake/alerthub-e2e-test/prd.md`
- `tests/intake/stream-test/prd.md`
- `tests/intake/play-e2e-test/prd.md`

## Guardrails

- The project must exist on disk before you attempt to write a PRD. If
  `GET /projects/<slug>` returns 404, create it first via
  `POST /projects {"name":"<slug>"}`.
- Never echo the full GITHUB_TOKEN or any credential back to the user or
  into the PRD body.
- If the repo already has a non-empty `prd.md`, prefer to
  *update in place* rather than clobbering — read the existing content,
  merge, and then write the merged version. The sidecar endpoint always
  overwrites, so the merging is your responsibility.
- Keep the PRD in clean Markdown. No smart quotes, no decorative emoji,
  no "AI wrote this" watermarks.
