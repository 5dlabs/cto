# Task 8: Implement team management API

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 8.

## Goal

Create team CRUD operations with member management and invite system

## Requirements

1. Create src/handlers/teams.rs
2. POST /api/teams - create team with owner role
3. GET /api/teams/:id - team details with member count
4. PATCH /api/teams/:id - update settings (owner/admin only)
5. POST /api/teams/:id/invite - generate 7-day expiring invite links
6. Implement role-based authorization middleware

## Acceptance Criteria

Test team creation, updates, member management, and invite link generation with proper authorization

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-8): Implement team management API`
