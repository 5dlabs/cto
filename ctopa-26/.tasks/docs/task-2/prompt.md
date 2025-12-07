# Task 2: Configure PostgreSQL database with sqlx migrations

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 2.

## Goal

Set up PostgreSQL connection pool and create database schema for teams, users, and tasks

## Requirements

1. Configure sqlx connection pool with PostgreSQL 15
2. Create migration files for users table (id, email, name, created_at, updated_at)
3. Create teams table (id, name, description, owner_id, created_at, updated_at)
4. Create team_members table (team_id, user_id, role, joined_at)
5. Create tasks table (id, team_id, title, description, assignee_id, status, due_date, created_at, updated_at, deleted_at)
6. Add foreign key constraints and indexes for performance

## Acceptance Criteria

Run migrations successfully, verify table creation with proper constraints, test connection pool functionality

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Configure PostgreSQL database with sqlx migrations`
