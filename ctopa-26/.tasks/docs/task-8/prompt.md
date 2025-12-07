# Task 8: Implement team management API endpoints

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 8.

## Goal

Create REST API endpoints for team CRUD operations and member management

## Requirements

1. Create teams.rs handler module
2. Implement POST /api/teams - create team with name, description
3. Implement GET /api/teams/:id - get team details with member count
4. Implement PATCH /api/teams/:id - update team settings (owner/admin only)
5. Add role-based authorization checks using middleware
6. Implement team member invitation system with UUID tokens
7. Add POST /api/teams/:id/invite endpoint with 7-day expiration

## Acceptance Criteria

Integration tests for all endpoints with different user roles and authorization scenarios

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-8): Implement team management API endpoints`
