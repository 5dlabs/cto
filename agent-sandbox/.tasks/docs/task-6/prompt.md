# Task 6: Build task board CRUD API with filtering and soft delete

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 6.

## Goal

Implement task management endpoints with create, read, update, delete operations, status/assignee/date filtering, and 30-day soft delete retention

## Requirements

1. Create src/api/tasks.rs with handlers:
   - POST /api/teams/:team_id/tasks: create_task(Path<Uuid>, Json<CreateTaskDto>)
   - GET /api/teams/:team_id/tasks: list_tasks(Path<Uuid>, Query<TaskFilters>)
     * WHERE deleted_at IS NULL AND team_id = $1
     * Apply filters: status IN (...), assignee_id = $2, due_date BETWEEN $3 AND $4
   - GET /api/tasks/:id: get_task(Path<Uuid>)
   - PATCH /api/tasks/:id: update_task(Path<Uuid>, Json<UpdateTaskDto>)
   - DELETE /api/tasks/:id: soft_delete_task(Path<Uuid>)
     * SET deleted_at = NOW() WHERE id = $1
2. Define TaskFilters struct with Optional<Vec<Status>>, Optional<Uuid>, Optional<DateRange>
3. Add background job (separate task) to hard delete tasks where deleted_at < NOW() - 30 days

## Acceptance Criteria

Integration tests: create task, verify in list. Update status, verify change. Soft delete, verify not in default list but exists in DB. Test all filter combinations. Verify only team members can access tasks

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-6): Build task board CRUD API with filtering and soft delete`
