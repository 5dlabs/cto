# Cursor CLI Integration Roadmap

Use this index to navigate detailed task breakdowns. Each task folder contains a `task.md` with prompt, acceptance criteria, implementation notes, and references.

## Group 0 – Foundations
- [Task 0.1](group-0/task-0.1/task.md): Requirements baseline / discovery.

## Group 1 – Controller & Templates
- [Task 1.1](group-1/task-1.1/task.md): Extend controller types and adapters.
- [Task 1.2](group-1/task-1.2/task.md): Build Cursor handlebars templates.
- [Task 1.3](group-1/task-1.3/task.md): Wire generator + tests.

## Group 2 – Helm, Config, Secrets
- [Task 2.1](group-2/task-2.1/task.md): Helm values & ConfigMap wiring.
- [Task 2.2](group-2/task-2.2/task.md): ExternalSecrets & GitOps updates.

## Group 3 – Runtime Glue
- [Task 3.1](group-3/task-3.1/task.md): Cursor adapter implementation.
- [Task 3.2](group-3/task-3.2/task.md): Job spec + volume mounts.
- [Task 3.3](group-3/task-3.3/task.md): Policy enforcement tests.

## Group 4 – Documentation
- [Task 4.1](group-4/task-4.1/task.md): Project documentation bundle.
- [Task 4.2](group-4/task-4.2/task.md): Ops/CI readme updates.

## Group 5 – Validation & Rollout
- [Task 5.1](group-5/task-5.1/task.md): Validation matrix.
- [Task 5.2](group-5/task-5.2/task.md): Staging rollout.
- [Task 5.3](group-5/task-5.3/task.md): Production enablement.

## Notes
- Keep Group 0 serial; Groups 1, 2, and 4 can run in parallel afterwards, with Group 3 depending on controller/template readiness.
- Capture meeting notes, approvals, and validation evidence inside the respective task folders (`notes.md`, `validation-log.md`, etc.) as work progresses.
