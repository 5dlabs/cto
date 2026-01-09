# NotifyCore Architecture

## Overview

NotifyCore is a single-service Rust application built with Axum. It follows a layered architecture pattern with clear separation between API handlers, business logic, and data access.

## System Diagram

```
                    ┌──────────────────┐
                    │   HTTP Clients   │
                    └────────┬─────────┘
                             │
                             ▼
┌────────────────────────────────────────────────────────┐
│                   NotifyCore Service                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │                  Router (Axum)                    │  │
│  │  /api/v1/notifications  │  /health               │  │
│  └──────────────┬───────────────────────────────────┘  │
│                 │                                       │
│  ┌──────────────▼───────────────────────────────────┐  │
│  │              Handlers Layer                       │  │
│  │  - create_notification()                         │  │
│  │  - get_notification()                            │  │
│  │  - list_notifications()                          │  │
│  │  - cancel_notification()                         │  │
│  │  - health_check()                                │  │
│  └──────────────┬───────────────────────────────────┘  │
│                 │                                       │
│  ┌──────────────▼───────────────────────────────────┐  │
│  │            Service Layer                          │  │
│  │  NotificationService                              │  │
│  │  - validate()                                     │  │
│  │  - create()                                       │  │
│  │  - find_by_id()                                   │  │
│  │  - list()                                         │  │
│  │  - cancel()                                       │  │
│  └──────────────┬───────────────────────────────────┘  │
│                 │                                       │
│  ┌──────────────▼───────────────────────────────────┐  │
│  │           Repository Layer                        │  │
│  │  NotificationRepository (sqlx)                    │  │
│  │  - insert()                                       │  │
│  │  - find_by_id()                                   │  │
│  │  - find_all()                                     │  │
│  │  - update_status()                                │  │
│  └──────────────┬───────────────────────────────────┘  │
│                 │                                       │
└─────────────────┼───────────────────────────────────────┘
                  │
                  ▼
         ┌────────────────┐
         │   PostgreSQL   │
         │   (Database)   │
         └────────────────┘
```

## Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| HTTP Server | Axum 0.7 | Routing, middleware, extractors |
| Async Runtime | Tokio 1.x | Async I/O, task scheduling |
| Database | sqlx 0.7 | Async PostgreSQL driver |
| Serialization | serde 1.x | JSON serialization |
| Validation | validator | Request validation |
| Logging | tracing | Structured logging |
| Config | dotenvy | Environment variables |

## Project Structure

```
notify-core/
├── Cargo.toml
├── Dockerfile
├── migrations/
│   └── 001_create_notifications.sql
└── src/
    ├── main.rs              # Entry point, server setup
    ├── config.rs            # Configuration from env
    ├── error.rs             # Error types and handling
    ├── handlers/
    │   ├── mod.rs
    │   ├── notifications.rs # Notification endpoints
    │   └── health.rs        # Health check endpoint
    ├── models/
    │   ├── mod.rs
    │   └── notification.rs  # Domain models
    ├── repository/
    │   ├── mod.rs
    │   └── notification.rs  # Database operations
    └── service/
        ├── mod.rs
        └── notification.rs  # Business logic
```

## Database Schema

```sql
-- migrations/001_create_notifications.sql

CREATE TYPE notification_channel AS ENUM ('email', 'slack', 'webhook');
CREATE TYPE notification_priority AS ENUM ('low', 'normal', 'high', 'critical');
CREATE TYPE notification_status AS ENUM ('pending', 'processing', 'delivered', 'failed', 'cancelled');

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel notification_channel NOT NULL,
    priority notification_priority NOT NULL DEFAULT 'normal',
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    status notification_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
```

## API Contract

### Create Notification

```http
POST /api/v1/notifications
Content-Type: application/json

{
  "channel": "slack",
  "priority": "high",
  "title": "Alert: High CPU Usage",
  "body": "Server cpu-1 is at 95% CPU utilization"
}
```

Response (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "channel": "slack",
  "priority": "high",
  "title": "Alert: High CPU Usage",
  "body": "Server cpu-1 is at 95% CPU utilization",
  "status": "pending",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

### Get Notification

```http
GET /api/v1/notifications/550e8400-e29b-41d4-a716-446655440000
```

Response (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "channel": "slack",
  "priority": "high",
  "title": "Alert: High CPU Usage",
  "body": "Server cpu-1 is at 95% CPU utilization",
  "status": "delivered",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:05Z"
}
```

### List Notifications

```http
GET /api/v1/notifications?page=1&per_page=20&status=pending
```

Response (200 OK):
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 150,
    "total_pages": 8
  }
}
```

### Cancel Notification

```http
DELETE /api/v1/notifications/550e8400-e29b-41d4-a716-446655440000
```

Response (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "cancelled"
}
```

### Health Check

```http
GET /health
```

Response (200 OK):
```json
{
  "status": "healthy",
  "database": "connected",
  "version": "0.1.0"
}
```

## Error Handling

All errors return JSON with consistent structure:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Notification not found",
    "details": null
  }
}
```

HTTP Status Codes:
- 200: Success
- 201: Created
- 400: Bad Request (validation errors)
- 404: Not Found
- 409: Conflict (e.g., cannot cancel delivered notification)
- 500: Internal Server Error

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `PORT` | No | 8080 | Server listen port |
| `RUST_LOG` | No | info | Log level |

## Dependencies (Cargo.toml)

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"
thiserror = "1"
```
