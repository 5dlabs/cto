# Task 2: Setup PostgreSQL database schema

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 2.

## Goal

Create database schema for teams, users, tasks, and authentication tables

## Requirements

1. Create migrations/001_initial.sql with tables: users, teams, team_members, tasks, refresh_tokens
2. Add sqlx migration runner in main.rs
3. Define schema with proper indexes and constraints
4. Add created_at, updated_at timestamps
5. Setup foreign key relationships and cascade rules

## Acceptance Criteria

Run migrations successfully and verify all tables created with proper constraints

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Setup PostgreSQL database schema`
