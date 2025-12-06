# Task 25: Implement email notification system with user preferences

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 25.

## Goal

Build email notification service for mentions and due date reminders with user-configurable preferences

## Requirements

1. Add dependencies: lettre = { version = "0.11", features = ["tokio1-native-tls"] }, tera = "1.19"
2. Create domain/notification.rs with:
   - enum NotificationType { Mention, DueDateReminder, TaskAssigned }
   - struct NotificationPreferences { user_id, email_mentions, email_due_dates, email_assignments }
3. Add notification_preferences table to schema
4. Create infra/email.rs with:
   - struct EmailService { smtp_transport: SmtpTransport, templates: Tera }
   - async fn send_email(to: &str, subject: &str, template: &str, context: Context)
5. Create templates in templates/emails/: mention.html, due_date_reminder.html
6. Create infra/jobs/notification_job.rs:
   - Check tasks with due_date within 24 hours, send reminders
   - Run daily via tokio cron
7. On task assignment/mention, check user preferences and send email asynchronously
8. Configure SMTP via env vars: SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD

## Acceptance Criteria

Unit tests with mock SMTP, integration tests for preference checks, verify emails sent on task assignment, test due date reminder job, verify unsubscribe respects preferences

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-25): Implement email notification system with user preferences`
