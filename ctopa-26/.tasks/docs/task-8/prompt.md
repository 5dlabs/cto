# Task 8: Build team management API endpoints

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 8.

## Goal

Implement CRUD operations for teams including creation, updates, and member management

## Requirements

1. POST /api/teams - Create team with owner assignment
2. GET /api/teams/:id - Fetch team details with member count and role checking
3. PATCH /api/teams/:id - Update team name/description (admin+ only)
4. DELETE /api/teams/:id - Soft delete team (owner only)
5. GET /api/teams/:id/members - List team members with roles
6. DELETE /api/teams/:id/members/:user_id - Remove member (admin+ only)

## Acceptance Criteria

Test all CRUD operations, verify role-based access, validate member count calculations and soft deletion

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-8): Build team management API endpoints`
