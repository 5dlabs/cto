# Task 22: Implement invite link generation with expiration

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 22.

## Goal

Create endpoint to generate time-limited invite links for teams with 7-day expiration and single-use token validation.

## Requirements

1. Extend domain/team.rs with InviteLink struct
2. Implement infra/invite_repository.rs:
   - create_invite(team_id, created_by) -> InviteLink with random token
   - get_invite_by_token(token) -> Option<InviteLink> (only if not expired)
   - delete_invite(token) -> Result<()>
3. Add to api/teams.rs:
   - POST /api/teams/:id/invite (requires admin role)
     - Generates UUID token
     - Stores in database with expires_at = now() + 7 days
     - Returns full invite URL: {BASE_URL}/invite/{token}
   - POST /api/invite/{token}/accept (requires auth)
     - Validates token not expired
     - Checks user not already member
     - Adds user to team with Member role
     - Deletes invite token (single use)
4. Use uuid crate for token generation
5. Add scheduled job to clean expired invites (runs daily)

## Acceptance Criteria

Integration tests: admin creates invite, verify token stored with correct expiration. Non-member accepts invite, verify added as member. Test expired token returns 404. Test already-member returns 400. Verify token deleted after acceptance. Test viewer cannot create invite.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-22): Implement invite link generation with expiration`
