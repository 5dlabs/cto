# Task 28: Implement team activity feed and member management UI

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 28.

## Goal

Create activity timeline showing recent team actions and member management interface with role assignment

## Requirements

1. Create components:
   - src/components/ActivityFeed.tsx: timeline of recent task/member events
   - src/components/MemberList.tsx: table of team members with roles
   - src/components/InviteModal.tsx: generate and display invite link
2. Add activity_log table to schema: id, team_id, user_id, action (enum), entity_type, entity_id, created_at
3. Create API endpoint GET /api/teams/:id/activity -> return last 50 events
4. Log activities in task/member operations: task.created, task.updated, member.joined, member.role_changed
5. Member management features:
   - Display member list with role badges
   - Owner/admin can change member roles via PATCH /api/teams/:team_id/members/:user_id
   - Owner/admin can remove members via DELETE /api/teams/:team_id/members/:user_id
6. Implement invite link generation: click button, show modal with copyable link
7. Style with Tailwind: use timeline component for activity feed

## Acceptance Criteria

Component tests for activity feed rendering, test role change authorization, verify activity logging on operations, E2E test for invite flow

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-28): Implement team activity feed and member management UI`
