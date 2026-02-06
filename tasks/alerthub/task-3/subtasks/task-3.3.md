# Subtask 3.3: Implement Database Layer

## Parent Task
Task 3

## Agent
code-implementer

## Parallelizable
Yes

## Description
Create database connection pool and query layer with sqlx.

## Details
- Configure sqlx pool with connection limits
- Create notification repository
- Create user preferences repository
- Implement transactions for multi-step operations
- Add retry logic for transient failures

## Deliverables
- `src/db/mod.rs` - Database module
- `src/db/notification.rs` - Notification queries
- `src/db/preferences.rs` - Preference queries
- `src/db/pool.rs` - Connection management

## Acceptance Criteria
- [ ] Connection pool works
- [ ] Queries execute correctly
- [ ] Transactions rollback on error
