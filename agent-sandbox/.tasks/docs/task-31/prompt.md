# Task 31: Implement team activity feed and member management UI

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 31.

## Goal

Create React components for team activity timeline and member management interface with role assignment.

## Requirements

1. Create backend endpoint in api/teams.rs:
   - GET /api/teams/:id/activity -> recent events (task created, member joined, etc.)
   - GET /api/teams/:id/members -> list members with roles
   - PATCH /api/teams/:id/members/:user_id -> update member role (admin only)
   - DELETE /api/teams/:id/members/:user_id -> remove member (admin only)
2. Store activity events in new table: team_events
   - Columns: id, team_id, event_type, actor_id, metadata (JSONB), created_at
   - Index on (team_id, created_at DESC)
3. Create React components:
   - src/components/ActivityFeed.tsx (timeline of events)
   - src/components/MemberList.tsx (table with avatars, roles)
   - src/components/MemberRoleSelect.tsx (dropdown for role change)
   - src/components/InviteModal.tsx (generate invite link)
4. Implement pagination for activity feed (infinite scroll)
5. Add confirmation dialog for member removal
6. Display member roles with badges (Owner, Admin, Member, Viewer)
7. Disable role changes for owners and self
8. Add route: /teams/:id/members

## Acceptance Criteria

Integration tests: fetch activity feed, verify events ordered by date. Test member list displays correct roles. Admin updates member role, verify API call and UI update. Test remove member with confirmation. Verify non-admin cannot change roles. Test invite link generation and copy to clipboard.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-31): Implement team activity feed and member management UI`
