# Task 9: Implement task management API endpoints

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 9.

## Goal

Create REST API endpoints for task CRUD operations with filtering and soft delete

## Requirements

1. Create tasks.rs handler module
2. Implement POST /api/teams/:team_id/tasks - create task
3. Implement GET /api/teams/:team_id/tasks - list tasks with filters (status, assignee, due_date)
4. Implement GET /api/tasks/:id, PATCH /api/tasks/:id, DELETE /api/tasks/:id
5. Add soft delete with deleted_at timestamp and 30-day cleanup job
6. Implement query parameter parsing for filters
7. Add validation for task status enum (todo/in-progress/done)

## Acceptance Criteria

Integration tests for CRUD operations, filtering, and soft delete functionality with cleanup verification

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-9): Implement task management API endpoints`
