# Task 13: Implement team activity feed and member management UI

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 13.

## Goal

Create React components for displaying team activity timeline and managing team members with role assignment

## Requirements

1. Add activity_log table: (id uuid, team_id uuid, user_id uuid, action varchar, entity_type varchar, entity_id uuid, timestamp timestamptz)
2. Create GET /api/teams/:id/activity endpoint:
   - Return last 50 activities with user details
3. Build React components:
   - ActivityFeed.tsx: timeline view with icons for each action type
   - MemberList.tsx: table with member name, role, actions
   - RoleSelector.tsx: dropdown to change member role (owner/admin only)
4. Implement member management endpoints:
   - POST /api/teams/:id/members: add member via invite code
   - PATCH /api/teams/:id/members/:user_id: update role
   - DELETE /api/teams/:id/members/:user_id: remove member
5. Log activities on task create/update/delete, member add/remove
6. Real-time activity updates via WebSocket

## Acceptance Criteria

Integration test: perform actions, verify logged in activity_log. UI test: render activity feed, verify actions displayed. Test role change UI requires admin. Test member removal

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-13): Implement team activity feed and member management UI`
