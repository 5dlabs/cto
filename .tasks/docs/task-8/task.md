## Document Migration and Rollout Plan (Atlas - CI/CD platforms)

### Objective
Create comprehensive documentation covering the migration plan, rollout strategy, rollback procedures, infrastructure sequencing, and operational runbooks for the Hermes pipeline — including ArgoCD promotion workflow configuration for staging-to-production deployment.

### Ownership
- Agent: atlas
- Stack: CI/CD platforms
- Priority: medium
- Status: pending
- Dependencies: 1, 2, 3, 4, 5, 6, 7

### Implementation Details
Step-by-step implementation:

1. **Migration plan document** (`docs/hermes/migration-plan.md`):
   - Pre-migration checklist: infrastructure verified, MinIO bucket provisioned, database migrations applied
   - Migration execution steps: trigger artifact migration (CLI or admin API), monitor progress via Grafana dashboard
   - Expected duration estimates based on artifact count and average size
   - Success criteria: all artifacts migrated, zero integrity failures, legacy access paths preserved
   - Failure recovery: re-run migration (idempotent), escalation contacts

2. **Rollout strategy document** (`docs/hermes/rollout-strategy.md`):
   - Phase 1: Dev environment — full feature enabled, used for development validation
   - Phase 2: Staging environment — feature-flagged Hermes path enabled, E2E tests must pass
   - Phase 3: Production canary — HERMES_ENABLED=true for <10% traffic (if applicable) or internal users only
   - Phase 4: Production GA — full rollout after 72-hour canary observation window
   - Go/no-go criteria for each phase transition: E2E pass rate, error rate thresholds, artifact generation success rate
   - Rollout risks identified:
     a. MinIO bucket isolation failure (risk: artifact cross-contamination with GitLab)
     b. Database migration lock contention during peak hours
     c. Headless browser OOM in artifact generation under load
     d. Session auth incompatibility with new RBAC claims for existing users

3. **Rollback procedures** (`docs/hermes/rollback-procedures.md`):
   - Immediate rollback: Set `HERMES_ENABLED=false` via ConfigMap update → ArgoCD sync → Hermes routes deregistered within 60 seconds
   - Database rollback: Reverse migration script (drop `hermes_artifacts` table, drop `deliberations` table — additive tables only, no ALTER TABLE rollback needed per D6 recommendation)
   - MinIO cleanup: Document bucket deletion procedure (only if full rollback needed)
   - Rollback trigger conditions: link to Grafana dashboard alert thresholds from Task 6
   - Post-rollback verification checklist

4. **Infrastructure sequencing** (`docs/hermes/infrastructure-sequence.md`):
   - Visual dependency diagram (Mermaid format) showing task execution order
   - Critical path: Task 1 → Task 2 → Task 3 → Task 7 → Task 9 → Task 10
   - Parallel execution windows: Tasks 4, 5, 6 can run in parallel after Task 3
   - Estimated timeline per task

5. **ArgoCD promotion workflow:**
   - Document the ArgoCD Application CR configuration from Task 1
   - Promotion flow: staging sync (automatic) → E2E tests pass (Task 7 GitHub Actions) → production sync (manual approval via ArgoCD UI or CLI)
   - Create ArgoCD sync wave annotations for the Hermes services
   - Document `argocd app sync hermes-backend-production` command and prerequisites

6. **Operational runbook** (`docs/hermes/runbook.md`):
   - Common issues and resolutions:
     a. Deliberation stuck in `processing` state → check headless browser pod logs, restart if OOM
     b. Artifact upload failures → check MinIO health dashboard, verify credentials
     c. High latency on artifact retrieval → check presigned URL TTL, MinIO performance
   - Monitoring commands: useful LogQL queries for Hermes troubleshooting
   - Escalation matrix

7. **ADR (Architecture Decision Record):** Create `docs/hermes/adr/` directory with ADRs for each resolved decision (D1-D5, D7-D9) documenting context, decision, and consequences.

### Subtasks
- [ ] Create migration plan and rollout strategy documents: Write the migration-plan.md with pre-migration checklist, execution steps, duration estimates, success criteria, and failure recovery. Write rollout-strategy.md with all 4 phases, go/no-go criteria, and identified risks.
- [ ] Create rollback procedures document: Write rollback-procedures.md covering immediate feature flag rollback, database rollback, MinIO cleanup, trigger conditions linked to Grafana alerts, and post-rollback verification checklist.
- [ ] Create infrastructure sequencing diagram and ArgoCD promotion workflow documentation: Write infrastructure-sequence.md with a Mermaid dependency diagram, critical path analysis, parallel execution windows, and timeline estimates. Document the ArgoCD promotion workflow including Application CR references, sync wave annotations, and promotion commands.
- [ ] Create operational runbook with LogQL queries: Write runbook.md covering common Hermes operational issues, resolution steps, validated LogQL queries for troubleshooting, and an escalation matrix.
- [ ] Create Architecture Decision Records (ADRs) for decisions D1-D5 and D7-D9: Create the ADR directory and write 8 individual ADR documents covering each resolved architectural decision, following the standard ADR format with Status, Context, Decision, and Consequences sections.