# Task 8: Create role-based authorization middleware and permission checks

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 8.

## Goal

Implement granular permission system for owner, admin, member, and viewer roles with route-level authorization guards

## Requirements

1. Define Role enum in src/domain/auth.rs: Owner, Admin, Member, Viewer
2. Create src/api/middleware/authorize.rs:
   - fn require_role(min_role: Role) -> Middleware
   - Extract user_id and team_id from request
   - Query team_members for user's role
   - Compare with required role hierarchy: Owner > Admin > Member > Viewer
   - Return 403 Forbidden if insufficient permissions
3. Define permission matrix:
   - Owner: all operations including team deletion
   - Admin: manage members, tasks, settings
   - Member: create/edit own tasks, view all
   - Viewer: read-only access
4. Apply middleware to route groups in router configuration
5. Create helper function: async fn check_team_access(user_id, team_id, min_role) -> Result<()>

## Acceptance Criteria

Unit test role hierarchy comparison. Integration tests: create team as owner, add member with viewer role, verify viewer cannot create tasks but can read. Test admin can manage members but not delete team

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-8): Create role-based authorization middleware and permission checks`
