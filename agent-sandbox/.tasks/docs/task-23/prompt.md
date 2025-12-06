# Task 23: Implement scheduled cleanup job for soft-deleted tasks

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 23.

## Goal

Create background job to permanently delete tasks older than 30 days from soft deletion

## Requirements

1. Add dependency: tokio-cron-scheduler = "0.9"
2. Create infra/jobs/cleanup.rs with:
   - async fn cleanup_deleted_tasks(pool: &PgPool) -> Result<u64>
   - SQL: DELETE FROM tasks WHERE deleted_at IS NOT NULL AND deleted_at < NOW() - INTERVAL '30 days'
3. In main.rs, spawn background task:
   - tokio::spawn(async move { loop { cleanup_deleted_tasks(&pool).await; tokio::time::sleep(Duration::from_secs(86400)).await; } })
4. Log cleanup results with count of deleted tasks
5. Add metrics counter for deleted tasks

## Acceptance Criteria

Unit test with mock database, integration test with test data, verify tasks deleted after 30 days, verify tasks within 30 days retained

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-23): Implement scheduled cleanup job for soft-deleted tasks`
