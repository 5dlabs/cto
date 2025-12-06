# Task 25: Implement email notifications with user preferences

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 25.

## Goal

Set up email notification system for task mentions and due date reminders with user-configurable preferences using background job queue.

## Requirements

1. Add dependencies: lettre = "0.11", tokio-cron-scheduler = "0.9"
2. Create domain/notification.rs:
   - NotificationType enum (Mention, DueDateReminder, TaskAssigned)
   - NotificationPreferences struct (email_enabled, mention_enabled, reminder_enabled)
3. Add notification_preferences column to users table (JSONB)
4. Implement infra/email.rs:
   - EmailService using lettre SMTP
   - send_task_mention(to, task, mentioned_by) -> Result<()>
   - send_due_date_reminder(to, task) -> Result<()>
   - HTML email templates in templates/ directory
5. Create api/notifications.rs:
   - GET /api/users/me/notification-preferences
   - PATCH /api/users/me/notification-preferences
6. Implement background jobs:
   - Parse task description for @mentions on create/update
   - Cron job (runs hourly) to check tasks due in next 24 hours
   - Queue jobs in Redis list: LPUSH email_queue {job_json}
   - Worker process: BRPOP email_queue, send email, retry on failure
7. Add SMTP config: SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD

## Acceptance Criteria

Unit tests for mention parsing. Integration tests: create task with @mention, verify email sent if preference enabled. Test due date reminder job finds tasks due tomorrow. Test preference updates respected. Mock SMTP in tests. Verify email queue processes jobs and retries failures.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-25): Implement email notifications with user preferences`
