# Task 23: Implement Task CRUD endpoints with filtering and soft delete

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 23.

## Goal

Create task management endpoints with status/assignee/date range filtering and soft delete with 30-day retention.

## Requirements

1. Create domain/task.rs with:
   - Task struct, TaskStatus enum (Todo, InProgress, Done)
   - TaskFilter struct (status, assignee_id, due_date_start, due_date_end)
2. Implement infra/task_repository.rs:
   - create_task(team_id, title, description, assignee_id, due_date) -> Task
   - get_tasks(team_id, filter: TaskFilter) -> Vec<Task> (WHERE deleted_at IS NULL)
   - get_task_by_id(id) -> Option<Task>
   - update_task(id, updates) -> Result<Task>
   - soft_delete_task(id) -> Result<()> (SET deleted_at = NOW())
   - hard_delete_old_tasks() -> Result<usize> (DELETE WHERE deleted_at < NOW() - INTERVAL '30 days')
3. Create api/tasks.rs:
   - POST /api/teams/:team_id/tasks (requires member role)
   - GET /api/teams/:team_id/tasks?status=todo&assignee=uuid&due_after=date&due_before=date
   - GET /api/tasks/:id (requires team member)
   - PATCH /api/tasks/:id (requires assignee or admin)
   - DELETE /api/tasks/:id (soft delete, requires admin)
4. Validate assignee is team member before creating task
5. Add background job for hard_delete_old_tasks (runs daily)

## Acceptance Criteria

Integration tests: create task, verify in list. Filter by status, assignee, date range. Update task status. Soft delete task, verify not in list but exists in DB. Test hard delete after 30 days. Verify member can create, only admin can delete. Test assignee validation.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-23): Implement Task CRUD endpoints with filtering and soft delete`
