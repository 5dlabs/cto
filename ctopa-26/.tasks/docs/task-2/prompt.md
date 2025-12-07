# Task 2: Setup PostgreSQL database schema and migrations

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 2.

## Goal

Create database schema for teams, users, tasks, and authentication with sqlx migrations

## Requirements

1. Create sqlx migration files in migrations/
2. Define users table: id (UUID), email, name, created_at, updated_at
3. Define teams table: id (UUID), name, description, owner_id (FK), created_at
4. Define team_members table: team_id (FK), user_id (FK), role (enum), joined_at
5. Define tasks table: id (UUID), team_id (FK), title, description, assignee_id (FK), status (enum), due_date, created_at, updated_at, deleted_at
6. Add indexes for performance on foreign keys and status fields

## Acceptance Criteria

Run migrations with `sqlx migrate run` and verify all tables created with proper constraints and indexes

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Setup PostgreSQL database schema and migrations`
