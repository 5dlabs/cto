# Task 29: Implement GDPR compliance with data export and deletion

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 29.

## Goal

Create endpoints for user data export and account deletion to comply with GDPR right to access and right to be forgotten.

## Requirements

1. Create api/gdpr.rs:
   - GET /api/users/me/export (requires auth)
     - Collect all user data: profile, teams, tasks, notifications
     - Generate JSON export file
     - Include metadata: export_date, data_version
   - DELETE /api/users/me (requires auth + password confirmation)
     - Soft delete user (set deleted_at)
     - Anonymize user data (replace email with deleted_{uuid}@example.com)
     - Remove from all teams
     - Reassign owned tasks to team admins
     - Delete device tokens, OAuth tokens, refresh tokens
     - Schedule hard delete after 30 days
2. Implement infra/gdpr.rs:
   - export_user_data(user_id) -> Result<UserDataExport>
   - anonymize_user(user_id) -> Result<()>
3. Create background job for hard deletion:
   - Runs daily, deletes users where deleted_at < NOW() - 30 days
   - Cascade delete user's created tasks, comments, etc.
4. Add audit log for GDPR operations
5. Ensure all user queries filter out deleted_at IS NULL

## Acceptance Criteria

Integration tests: export user data, verify completeness. Delete account, verify anonymization and team removal. Test task reassignment. Verify hard delete after 30 days. Test deleted user cannot login. Verify GDPR operations logged.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-29): Implement GDPR compliance with data export and deletion`
