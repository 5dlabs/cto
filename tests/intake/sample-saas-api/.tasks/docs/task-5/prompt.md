# Task 5: Build real-time notifications with WebSocket and push integration

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 5.

## Goal

Implement WebSocket connections for live updates, FCM push notifications, email notifications, and user preference controls

## Requirements

1. Setup WebSocket endpoint with axum::extract::ws for 1000 concurrent connections
2. Implement Redis pub/sub for broadcasting task updates across instances
3. Create notification preferences table and user controls
4. Integrate FCM for mobile push notifications
5. Add email notification system for mentions and due date reminders
6. Build connection management with user authentication

```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use redis::aio::PubSub;

struct NotificationService {
    redis_pubsub: PubSub,
    fcm_client: fcm::Client,
    email_client: lettre::SmtpTransport,
}

#[derive(Serialize, Deserialize)]
enum NotificationType {
    TaskCreated,
    TaskUpdated,
    TaskAssigned,
    DueDateReminder,
    Mention,
}

// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade, auth: AuthUser) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, auth.user_id))
}

// Redis pub/sub message
async fn broadcast_task_update(task_id: Uuid, notification_type: NotificationType)
```

## Acceptance Criteria

WebSocket connection tests with 1000+ concurrent connections, Redis pub/sub message delivery verification, FCM integration tests, email notification delivery tests, user preference filtering validation

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Build real-time notifications with WebSocket and push integration`
