# Task 10: Implement email notification system for mentions and due dates

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 10.

## Goal

Set up email service integration with template rendering for task mentions and due date reminders, respecting user preferences

## Requirements

1. Add dependencies: lettre = "0.11", tera = "1.19"
2. Create src/infra/email.rs with SMTP configuration from env vars
3. Add user_preferences table: (user_id uuid PRIMARY KEY, email_mentions bool, email_due_dates bool, email_frequency varchar)
4. Create email templates in templates/:
   - mention_notification.html: "@{user} mentioned you in {task}"
   - due_date_reminder.html: "Task {title} is due in {hours} hours"
5. Implement notification queue in src/domain/notifications.rs:
   - fn send_mention_email(user_id, task_id, mentioner_id)
   - Check user preferences before sending
   - Parse task description for @mentions on task create/update
6. Create background job (tokio::spawn) to check due dates every hour:
   - Query tasks WHERE due_date BETWEEN NOW() AND NOW() + 24 hours
   - Send reminders to assignees if email_due_dates = true

## Acceptance Criteria

Unit test mention parsing. Integration test: create task with @mention, verify email sent (use mailhog in dev). Test preference controls prevent emails. Test due date reminder job with mock time

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-10): Implement email notification system for mentions and due dates`
