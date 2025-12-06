# Task 32: Implement GDPR compliance features

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 32.

## Goal

Add data export and deletion endpoints to comply with GDPR requirements for user data portability and right to be forgotten

## Requirements

1. Create api/gdpr.rs:
   - GET /api/users/me/export -> generate JSON export of all user data
   - POST /api/users/me/delete -> initiate account deletion
2. Data export includes:
   - User profile (email, created_at)
   - Teams owned/member of
   - All tasks created or assigned
   - Activity history
   - Format: { "user": {...}, "teams": [...], "tasks": [...], "activity": [...] }
3. Account deletion process:
   - Soft delete user (set deleted_at)
   - Anonymize user data in tasks (set assignee_id to NULL, created_by to 'deleted_user')
   - Remove from all teams
   - Delete OAuth tokens
   - Delete device tokens
   - Schedule permanent deletion after 30 days
4. Add deletion confirmation: require password re-entry
5. Send email confirmation of deletion request
6. Create infra/jobs/gdpr_cleanup.rs: permanently delete users after 30 days

## Acceptance Criteria

Test data export completeness, verify account deletion anonymizes data correctly, test 30-day deletion delay, verify email confirmation sent, test deletion cancellation

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-32): Implement GDPR compliance features`
