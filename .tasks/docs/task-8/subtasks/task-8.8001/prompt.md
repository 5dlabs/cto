Implement subtask 8001: Create migration plan and rollout strategy documents

## Objective
Write the migration-plan.md with pre-migration checklist, execution steps, duration estimates, success criteria, and failure recovery. Write rollout-strategy.md with all 4 phases, go/no-go criteria, and identified risks.

## Steps
1. Create `docs/hermes/migration-plan.md` with the following sections:
   - **Pre-migration Checklist**: bullet list including infrastructure verification (MinIO bucket, database, network policies), dependency task completion status, monitoring dashboards live
   - **Migration Execution Steps**: numbered steps for triggering artifact migration via admin API (`POST /api/hermes/admin/migrate-artifacts`), alternative CLI command, monitoring via Grafana dashboard link
   - **Duration Estimates**: provide formula based on artifact count × avg size / throughput, with example calculations for 100, 1000, 10000 artifacts
   - **Success Criteria**: all artifacts migrated (count match), zero SHA integrity failures, legacy access paths still functional during transition
   - **Failure Recovery**: idempotent re-run procedure, partial failure handling, escalation contacts table

2. Create `docs/hermes/rollout-strategy.md` with:
   - Phase 1-4 table with environment, flag state, audience, and duration columns
   - Go/no-go criteria for each phase transition (E2E pass rate ≥ 100%, error rate < 0.1%, artifact generation success rate ≥ 99%)
   - Risk register table with 4 identified risks, impact, likelihood, and mitigation columns
   - Sign-off section template for phase transitions

## Validation
Verify both files exist at the specified paths. Check that migration-plan.md contains headings: 'Pre-migration Checklist', 'Migration Execution Steps', 'Duration Estimates', 'Success Criteria', 'Failure Recovery'. Check that rollout-strategy.md contains headings for all 4 phases plus 'Go/No-Go Criteria' and 'Risks'.