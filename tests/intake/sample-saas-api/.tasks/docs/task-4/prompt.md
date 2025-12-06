# Task 4: Implement task board with CRUD operations and filtering

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 4.

## Goal

Build comprehensive task management system with status tracking, assignments, filtering, and soft delete with 30-day retention

## Requirements

1. Create tasks table with soft delete (deleted_at column)
2. Implement task CRUD with validation for title, description, assignee, status, due_date
3. Add status enum (todo, in_progress, done) with database constraints
4. Build filtering by status, assignee, due date range using query parameters
5. Implement soft delete with background cleanup job for 30-day retention
6. Add task assignment validation (assignee must be team member)

```rust
#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
enum TaskStatus { Todo, InProgress, Done }

#[derive(sqlx::FromRow, Serialize)]
struct Task {
    id: Uuid,
    team_id: Uuid,
    title: String,
    description: Option<String>,
    assignee_id: Option<Uuid>,
    status: TaskStatus,
    due_date: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

// GET /api/teams/:team_id/tasks with filtering
struct TaskFilters {
    status: Option<TaskStatus>,
    assignee_id: Option<Uuid>,
    due_after: Option<DateTime<Utc>>,
    due_before: Option<DateTime<Utc>>,
}
```

## Acceptance Criteria

CRUD operation tests, filtering functionality validation, soft delete behavior verification, task assignment permission tests, 30-day cleanup job testing

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Implement task board with CRUD operations and filtering`
