# Acceptance Criteria: Task 29

- [ ] Create endpoints for user data export and account deletion to comply with GDPR right to access and right to be forgotten.
- [ ] Integration tests: export user data, verify completeness. Delete account, verify anonymization and team removal. Test task reassignment. Verify hard delete after 30 days. Test deleted user cannot login. Verify GDPR operations logged.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 29.1: Implement user data export endpoint with comprehensive data collection
- [ ] 29.2: Design and implement JSON export format with metadata
- [ ] 29.3: Create account deletion endpoint with password confirmation
- [ ] 29.4: Implement user anonymization logic with PII scrubbing
- [ ] 29.5: Implement task reassignment to team admins on user deletion
- [ ] 29.6: Implement cascading cleanup of tokens and sessions
- [ ] 29.7: Create background job for hard deletion after 30-day retention
