# Task 21: Implement Team Management CRUD endpoints with RBAC

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 21.

## Goal

Create team management endpoints with role-based access control enforcing owner/admin/member/viewer permissions.

## Requirements

1. Create domain/team.rs with:
   - Team, TeamMember structs
   - TeamRole enum (Owner, Admin, Member, Viewer)
   - Permission checks: can_update_team(role), can_invite(role)
2. Implement infra/team_repository.rs with sqlx queries:
   - create_team(name, description, owner_id) -> Team
   - get_team_by_id(id) -> Option<Team> with member count
   - update_team(id, name, description) -> Result<Team>
   - get_user_role_in_team(user_id, team_id) -> Option<TeamRole>
3. Create api/teams.rs with handlers:
   - POST /api/teams (requires auth) -> creates team, adds creator as owner
   - GET /api/teams/:id (requires member role) -> returns team with member_count
   - PATCH /api/teams/:id (requires admin role) -> updates team
4. Implement RequireTeamRole(role) extractor that:
   - Validates user is team member
   - Checks role meets minimum requirement
   - Returns 403 Forbidden if insufficient permissions

## Acceptance Criteria

Integration tests: create team as user A, verify user A is owner. User B cannot access team. Add user B as member, verify they can GET but not PATCH. Test admin can PATCH. Verify member counts are accurate.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-21): Implement Team Management CRUD endpoints with RBAC`
