# Task 2: Configure PostgreSQL database schema and migrations

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 2.

## Goal

Design and implement the database schema for teams, users, tasks, and invites with proper indexes and constraints using sqlx migrations

## Requirements

1. Create migrations directory: `sqlx migrate add initial_schema`
2. Define tables:
   - users (id uuid PRIMARY KEY, email varchar UNIQUE, password_hash varchar, created_at timestamptz)
   - teams (id uuid PRIMARY KEY, name varchar, description text, created_at timestamptz, deleted_at timestamptz)
   - team_members (team_id uuid, user_id uuid, role varchar, PRIMARY KEY(team_id, user_id))
   - tasks (id uuid PRIMARY KEY, team_id uuid, title varchar, description text, assignee_id uuid, status varchar, due_date timestamptz, created_at timestamptz, deleted_at timestamptz)
   - invites (id uuid PRIMARY KEY, team_id uuid, token varchar UNIQUE, expires_at timestamptz)
3. Add indexes on foreign keys, deleted_at, status, due_date
4. Create sqlx connection pool in src/infra/database.rs
5. Implement health check query

## Acceptance Criteria

Run `sqlx migrate run` against test database. Verify all tables created with `\dt` in psql. Test connection pool with simple SELECT query

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Configure PostgreSQL database schema and migrations`
