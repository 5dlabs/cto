# Acceptance Criteria: Task 25

- [ ] Set up email notification system for task mentions and due date reminders with user-configurable preferences using background job queue.
- [ ] Unit tests for mention parsing. Integration tests: create task with @mention, verify email sent if preference enabled. Test due date reminder job finds tasks due tomorrow. Test preference updates respected. Mock SMTP in tests. Verify email queue processes jobs and retries failures.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 25.1: Set up email service with SMTP configuration and lettre integration
- [ ] 25.2: Create HTML email templates for mentions and due date reminders
- [ ] 25.3: Implement notification preferences schema and API endpoints
- [ ] 25.4: Implement mention parsing logic for task descriptions
- [ ] 25.5: Implement Redis-based job queue with LPUSH/BRPOP pattern
- [ ] 25.6: Create background worker process for email sending with retry logic
- [ ] 25.7: Implement cron job for due date reminder scanning
- [ ] 25.8: Integrate preference checking and mention job creation in task workflows
