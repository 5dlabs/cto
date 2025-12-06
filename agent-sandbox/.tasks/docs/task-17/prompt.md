# Task 17: Design and implement database schema with migrations

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 17.

## Goal

Create PostgreSQL schema for teams, users, tasks, invites, and audit tables with proper indexes and constraints

## Requirements

1. Create migration files using sqlx migrate add
2. Schema tables:
   - users: id (uuid), email, password_hash, oauth_provider, created_at, deleted_at
   - teams: id (uuid), name, description, owner_id (fk users), created_at, deleted_at
   - team_members: team_id, user_id, role (enum: owner/admin/member/viewer), joined_at
   - tasks: id (uuid), team_id (fk), title, description, assignee_id (fk users), status (enum), due_date, created_at, updated_at, deleted_at
   - invites: id (uuid), team_id (fk), token, expires_at, created_by (fk users)
3. Add indexes: tasks(team_id, status), tasks(assignee_id), team_members(user_id), invites(token)
4. Add CHECK constraints for status enum values
5. Setup foreign key cascades appropriately

## Acceptance Criteria

Run sqlx migrate run, verify schema with \d commands, test constraint violations, verify indexes exist

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-17): Design and implement database schema with migrations`
