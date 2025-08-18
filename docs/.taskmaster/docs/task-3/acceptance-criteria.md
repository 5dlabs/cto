# Acceptance Criteria

- On successful completion of the Play workflow for a task, the task directory is moved to `.completed/task-<id>`
- Idempotent: skip move if already archived
- Logged action with task id and target path
