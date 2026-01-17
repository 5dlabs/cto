---
title: CTO Lifecycle Test Execution Prompt
description: Single-objective loop prompt for Ralph execution
---

# CTO Platform Lifecycle Test - Execution Prompt

You are executing one objective at a time for the AlertHub lifecycle test.

## CRITICAL: Overnight Run Context

This is an **unattended overnight run**. You must:
1. Use `num_tasks=30` when calling intake (NOT 50) to avoid output truncation
2. Monitor pod status every check - CodeRun status can be stale
3. Report failures immediately with full evidence
4. Continue through the full lifecycle: intake → play → quality → security → testing → integration

## Read First

1. Read `lifecycle-test/pin.md` for stable context and commands.
2. Read `lifecycle-test/pin.lookup.md` for search aliases and pointers.
3. Read `lifecycle-test/implementation-plan.md` for objective ordering and linkage.
4. Work from the repo root: `/Users/jonathonfritz/code/work-projects/5dlabs/cto`.
5. Read the current objective in `lifecycle-test/current-objective.md`.

## Loop Discipline (Single Objective Only)

1. Execute **only** the objective described in `current-objective.md`.
2. After **every** operation, check logs (see `pin.md`).
3. Capture evidence and append to `lifecycle-test/report.json`.
4. Update `lifecycle-test/progress.txt` with a short status note.
5. If any gate fails, **stop**, run cleanup from `lifecycle-test/ralph-cto.json`,
   and do not proceed until the system is clean.

## Output Requirements

- Provide command output snippets as evidence.
- Do not claim success without verification.
- Avoid PII and secrets in logs or evidence; summarize or redact if needed.
- Keep responses concise and focused on the current objective.
