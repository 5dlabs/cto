---
title: CTO Lifecycle Pin Lookup Index
description: Search aliases and pointer map for Ralph loops
---

# Pin Lookup Index (AlertHub)

Use this lookup index to improve search hits and reduce invention. Each entry
maps a concept to aliases and pointers so the agent can resolve the right files
and commands quickly.

## intake
- aliases: PRD ingestion, task generation, Morgan, intake CodeRun, tasks.json
- pointers:
  - docs/workflow-lifecycle-checklist.md (Intake gates)
  - lifecycle-test/ralph-cto.json (phase: intake)
  - tests/intake/alerthub-e2e-test/prd.md
  - tests/intake/alerthub-e2e-test/architecture.md

## play
- aliases: play workflow, task CodeRun, tool config, Linear dialog
- pointers:
  - docs/workflow-lifecycle-checklist.md (Play gates)
  - lifecycle-test/ralph-cto.json (phase: play)
  - docs/heal-play.md

## quality
- aliases: Cleo review, lint checks, code quality gate
- pointers:
  - docs/workflow-lifecycle-checklist.md (Quality gates)
  - lifecycle-test/ralph-cto.json (phase: quality)

## security
- aliases: Cipher scan, vulnerability review, security gate
- pointers:
  - docs/workflow-lifecycle-checklist.md (Security gates)
  - lifecycle-test/ralph-cto.json (phase: security)

## testing
- aliases: Tess, test strategy, automated tests, CI tests
- pointers:
  - docs/workflow-lifecycle-checklist.md (Testing gates)
  - lifecycle-test/ralph-cto.json (phase: testing)

## integration
- aliases: Atlas, merge, rebase, PR checks, integration gate
- pointers:
  - docs/workflow-lifecycle-checklist.md (Integration gates)
  - lifecycle-test/ralph-cto.json (phase: integration)

## deploy
- aliases: Bolt, deploy task, manifests, health checks
- pointers:
  - docs/workflow-lifecycle-checklist.md (Deploy gates)
  - lifecycle-test/ralph-cto.json (phase: deploy)
  - infra/gitops/

## postflight
- aliases: telemetry, Linear timeline, post-deploy verification
- pointers:
  - docs/workflow-lifecycle-checklist.md (PostFlight gates)
  - lifecycle-test/ralph-cto.json (phase: postflight)
  - docs/heal-play.md (observability expectations)

## observability
- aliases: logs, evidence, report, metrics
- pointers:
  - lifecycle-test/report.json
  - lifecycle-test/progress.txt
  - /tmp/cto-launchd/*.log
