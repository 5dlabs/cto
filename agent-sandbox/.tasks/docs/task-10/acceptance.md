# Acceptance Criteria: Task 10

- [ ] Set up email service integration with template rendering for task mentions and due date reminders, respecting user preferences
- [ ] Unit test mention parsing. Integration test: create task with @mention, verify email sent (use mailhog in dev). Test preference controls prevent emails. Test due date reminder job with mock time
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 10.1: Set up SMTP configuration and email template rendering system
- [ ] 10.2: Create user preferences table and management endpoints
- [ ] 10.3: Implement mention parsing logic in task descriptions
- [ ] 10.4: Build mention notification system with preference checks
- [ ] 10.5: Create due date reminder background job with scheduling
