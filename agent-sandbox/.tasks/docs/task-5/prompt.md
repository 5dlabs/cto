# Task 5: Implement team management API endpoints

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 5.

## Goal

Create CRUD operations for teams including creation, retrieval with member count, updates, and invite link generation with 7-day expiry

## Requirements

1. Create src/api/teams.rs with handlers:
   - POST /api/teams: create_team(Json<CreateTeamDto>) -> Json<TeamResponse>
     * Insert into teams table
     * Add creator as owner in team_members
   - GET /api/teams/:id: get_team(Path<Uuid>) -> Json<TeamWithMemberCount>
     * JOIN with team_members to count members
     * Check user has access (is member)
   - PATCH /api/teams/:id: update_team(Path<Uuid>, Json<UpdateTeamDto>)
     * Verify user is owner/admin
   - POST /api/teams/:id/invite: generate_invite(Path<Uuid>)
     * Create UUID token, store in invites table with expires_at = now() + 7 days
     * Return invite URL
2. Define DTOs in src/domain/team.rs
3. Implement authorization checks using role from team_members

## Acceptance Criteria

Integration tests: create team as user A, verify user A can access, user B cannot. Test invite generation and expiry. Test PATCH requires admin role. Verify member count accuracy

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Implement team management API endpoints`
