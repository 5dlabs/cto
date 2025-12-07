# Task 9: Build task management system

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 9.

## Goal

Implement full CRUD operations for tasks with filtering and soft delete

## Requirements

1. Create src/handlers/tasks.rs
2. CRUD endpoints: POST, GET, PATCH, DELETE /api/teams/:id/tasks
3. Task fields: title, description, assignee_id, status, due_date, deleted_at
4. Add filtering by status, assignee, due date range
5. Implement soft delete with 30-day retention
6. Add task assignment validation

## Acceptance Criteria

Test all CRUD operations, filtering combinations, soft delete behavior, and authorization

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-9): Build task management system`
