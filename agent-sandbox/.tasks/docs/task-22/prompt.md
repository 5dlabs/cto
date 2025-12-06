# Task 22: Implement Task Board CRUD API with filtering

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 22.

## Goal

Create comprehensive task management endpoints with status transitions, assignment, filtering, and soft delete functionality

## Requirements

1. Create domain/task.rs with:
   - enum TaskStatus { Todo, InProgress, Done }
   - struct Task { id, team_id, title, description, assignee_id, status, due_date, created_at, updated_at, deleted_at }
   - struct TaskRepository trait
2. Implement infra/repositories/task_repo.rs with sqlx
3. Create api/tasks.rs handlers:
   - POST /api/teams/:team_id/tasks { title, description, assignee_id?, due_date? } -> verify user is team member
   - GET /api/teams/:team_id/tasks?status=&assignee_id=&due_date_from=&due_date_to= -> filter with WHERE clauses, exclude deleted_at IS NOT NULL
   - GET /api/tasks/:id -> verify user has team access
   - PATCH /api/tasks/:id { title?, description?, assignee_id?, status?, due_date? } -> verify team membership
   - DELETE /api/tasks/:id -> soft delete by setting deleted_at = NOW()
4. Add validation: title 1-200 chars, assignee must be team member
5. Publish task events to Redis pub/sub on create/update/delete

## Acceptance Criteria

Integration tests for CRUD operations, verify filtering combinations, test soft delete and 30-day retention, verify authorization checks, test assignee validation

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-22): Implement Task Board CRUD API with filtering`
