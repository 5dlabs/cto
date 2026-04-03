# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma-1 Agent Delegation E2E
> 
> ## Goal
> Validate the full intake pipeline end-to-end with agent delegation enabled, confirming:
> 1. Linear issues are created with the correct agent assignee (delegate_id)
> 2. Hermes research integration produces content in the deliberation path
> 3. Design snapshot PR surfacing works
> 4. Discord and Linear bridge notifications fire correctly
> 
> ## Context
> The PM server now resolves agent hints to Linear user IDs at issue creation time via `resolve_agent_delegates()`. Previously all issues were created unassigned. This run validates the full delegation flow from PRD → deliberation → task generation → issue creation with assigned agents.
> 
> ## Requirements
> - Pipeline completes through all stages (deliberation, task generation, issue creation)
> - At least 5 tasks generated with agent assignments
> - Linear issues show assigned agents (not just "agent:pending" labels)
> - Research memos include Hermes-sourced content where NOUS_API_KEY is available
> - PR created in target repo with task scaffolds
> 
> ## Target Repository
> - Organization: 5dlabs
> - Repository: sigma-1
> - Visibility: private
> 
> ## Acceptance Criteria
> - [ ] Pipeline completes without fatal errors
> - [ ] Linear session created with issues
> - [ ] Issues have delegate_id set (visible as assignee in Linear)
> - [ ] PR created in sigma-1 with generated artifacts
> - [ ] Discord notification posted for pipeline start/complete
> 

## 2. Project Scope

- Total tasks identified: 10
- Task 1: Provision Dev Infrastructure for Sigma-1 E2E Pipeline (Bolt - Kubernetes/Helm) — Set up the development infrastructure required for the Sigma-1 agent delegation E2E pipeline, including namespaces, secrets, and a ConfigMap aggregating service endpoints.
- Task 2: Extend PM Server for Agent Delegation in Linear Issues (Nova - Bun/Elysia) — Update the PM server to ensure that agent hints are resolved to Linear user IDs at issue creation, assigning the correct delegate_id for each generated task.
- Task 3: Integrate Hermes Research for Deliberation Path (Nova - Bun/Elysia) — Enable Hermes research integration in the deliberation path, ensuring that research memos include Hermes-sourced content when NOUS_API_KEY is available.
- Task 4: Implement Design Snapshot PR Surfacing (Blaze - React/Next.js) — Enable surfacing of design snapshot PRs in the web frontend, allowing users to view and interact with design deltas generated during the pipeline.
- Task 5: Implement Discord Notification for Pipeline Start/Complete (Nova - Bun/Elysia) — Add Discord notification hooks to the pipeline, posting messages at pipeline start and completion to the configured Discord channel.
- Task 6: Implement Linear-Discord Bridge for Issue Notifications (Nova - Bun/Elysia) — Ensure that notifications about Linear issue creation are bridged to Discord, so that new issues and their assignees are announced in real time.
- Task 7: Generate PR in Sigma-1 Repo with Task Scaffolds (Nova - Bun/Elysia) — Automate the creation of a pull request in the 5dlabs/sigma-1 repository containing generated task scaffolds for the E2E pipeline run.
- Task 8: End-to-End Pipeline Integration Test (Tess - Test frameworks) — Develop and execute an E2E integration test that validates the full pipeline from PRD intake to issue creation, research memo inclusion, PR surfacing, and notifications.
- Task 9: Production Hardening: HA Scaling, CDN, TLS, Ingress (Bolt - Kubernetes/Helm) — Harden the production deployment by enabling high-availability scaling, configuring CDN, TLS, and ingress for all services in the Sigma-1 pipeline.
- Task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm) — Finalize production hardening by implementing RBAC, secret rotation policies, and audit logging for the Sigma-1 pipeline infrastructure.

## 3. Resolved Decisions

### [D1] Decision 1

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D2] Decision 2

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output


## 4. Architecture Overview

- Deliberation status: completed
- Debate turns: 2
- Elapsed minutes: 15
- Decision points considered: 6
- Decisions resolved: 2

## 5. Design Intake Summary

mode=ingest_plus_stitch; has_frontend=true; targets=web; stitch_status=failed; stitch_reason=

## 6. Open Questions

- Use the deliberation record and task decomposition to resolve any remaining implementation details not captured above.


