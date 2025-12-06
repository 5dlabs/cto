# Task 17: Design and implement database schema with migrations

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 17.

## Goal

Create PostgreSQL schema for users, teams, team_members, tasks, and invite_links tables with proper indexes and constraints. Implement soft delete pattern for tasks.

## Requirements

1. Create migrations directory: migrations/
2. Migration 001_create_users.sql:
   - id (UUID PK), email (UNIQUE), password_hash, oauth_provider, oauth_id, created_at, updated_at
3. Migration 002_create_teams.sql:
   - id (UUID PK), name, description, owner_id (FK users), created_at, updated_at
4. Migration 003_create_team_members.sql:
   - id (UUID PK), team_id (FK teams), user_id (FK users), role (ENUM: owner, admin, member, viewer), joined_at
   - UNIQUE constraint on (team_id, user_id)
5. Migration 004_create_tasks.sql:
   - id (UUID PK), team_id (FK teams), title, description, assignee_id (FK users), status (ENUM: todo, in_progress, done), due_date, deleted_at (nullable), created_at, updated_at
   - Index on (team_id, deleted_at, status)
6. Migration 005_create_invite_links.sql:
   - id (UUID PK), team_id (FK teams), token (UNIQUE), expires_at, created_by (FK users), created_at
7. Run migrations with sqlx migrate run

## Acceptance Criteria

Run sqlx migrate run and verify all tables exist with correct schema using psql. Test foreign key constraints by attempting invalid inserts. Verify indexes exist with EXPLAIN ANALYZE queries.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-17): Design and implement database schema with migrations`
