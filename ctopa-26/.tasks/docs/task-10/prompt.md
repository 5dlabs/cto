# Task 10: Build task management API endpoints

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 10.

## Goal

Implement CRUD operations for tasks with filtering, assignment, and status management

## Requirements

1. POST /api/teams/:id/tasks - Create task with validation
2. GET /api/teams/:id/tasks - List tasks with filtering (status, assignee, due_date range)
3. GET /api/teams/:id/tasks/:task_id - Get single task details
4. PATCH /api/teams/:id/tasks/:task_id - Update task (assignee can update status/description)
5. DELETE /api/teams/:id/tasks/:task_id - Soft delete with 30-day retention
6. Add query parameters for pagination and sorting

## Acceptance Criteria

Test all CRUD operations, verify filtering and pagination, validate soft deletion and permission enforcement

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-10): Build task management API endpoints`
