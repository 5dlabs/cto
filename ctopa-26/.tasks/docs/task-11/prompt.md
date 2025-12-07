# Task 11: Implement real-time notification system

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 11.

## Goal

Create notification service with WebSocket, email, and push notification support

## Requirements

1. Create notifications.rs service module
2. Implement notification types: task_assigned, task_updated, task_due_soon, mention
3. Add email notification using SMTP (lettre crate)
4. Integrate FCM for mobile push notifications
5. Store user notification preferences in database
6. Create notification queue using Redis pub/sub
7. Add background worker for processing notification queue

## Acceptance Criteria

Unit tests for notification logic and integration tests for email/push delivery with mock services

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-11): Implement real-time notification system`
