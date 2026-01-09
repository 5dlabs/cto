# Project: NotifyCore - Simple Notification Router

## Vision

NotifyCore is a lightweight notification routing service that receives, validates, and dispatches notifications to various channels. Built with Rust and Axum for high performance and reliability, it serves as the core routing layer for notification delivery. This is a minimal E2E test project designed to validate the CTO platform's intake and play workflow with a single agent (Rex).

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    NotifyCore Service                    │
├─────────────────────────────────────────────────────────┤
│  API Layer (Axum)                                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   Submit    │  │   Query     │  │   Health    │    │
│  │  Endpoint   │  │  Endpoints  │  │   Check     │    │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘    │
│         │                │                              │
│  ┌──────┴────────────────┴──────┐                      │
│  │      Notification Service     │                      │
│  │   (validation, routing)       │                      │
│  └──────────────┬───────────────┘                      │
│                 │                                        │
├─────────────────┴───────────────────────────────────────┤
│  Storage Layer                                          │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │ PostgreSQL  │  │    Redis    │                      │
│  │ (persist)   │  │  (cache)    │                      │
│  └─────────────┘  └─────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

---

## Service Specification

**Agent**: Rex  
**Priority**: High  
**Language**: Rust 1.75+  
**Framework**: Axum 0.7

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/notifications` | Submit a new notification |
| GET | `/api/v1/notifications/:id` | Get notification by ID |
| GET | `/api/v1/notifications` | List notifications with pagination |
| DELETE | `/api/v1/notifications/:id` | Cancel a pending notification |
| GET | `/health` | Health check endpoint |

### Data Models

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub channel: Channel,
    pub priority: Priority,
    pub title: String,
    pub body: String,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Email,
    Slack,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Pending,
    Processing,
    Delivered,
    Failed,
    Cancelled,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub channel: Channel,
    pub priority: Priority,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<NotificationStatus>,
}
```

### Core Features

1. **Notification Submission**: Accept notifications via REST API with validation
2. **Status Tracking**: Track notification lifecycle from pending to delivered/failed
3. **Pagination**: List notifications with cursor-based pagination
4. **Cancellation**: Allow cancelling pending notifications
5. **Health Checks**: Readiness and liveness endpoints

### Infrastructure Dependencies

| Component | Purpose | Connection |
|-----------|---------|------------|
| PostgreSQL | Notification persistence | `DATABASE_URL` env var |
| Redis | Status caching (optional) | `REDIS_URL` env var |

### Configuration

Environment variables:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string (optional)
- `PORT` - Server port (default: 8080)
- `RUST_LOG` - Log level (default: info)

---

## Technical Context

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.75+ |
| Framework | Axum | 0.7 |
| Async Runtime | Tokio | 1.x |
| Database | sqlx | 0.7 |
| Serialization | serde | 1.x |
| Tracing | tracing | 0.1 |

---

## Constraints

- API response time < 50ms p95
- Support 1,000 notifications/minute
- Graceful shutdown handling
- Structured JSON logging

---

## Non-Goals

- Message queue integration (Kafka, RabbitMQ)
- External channel delivery (actual Slack/email sending)
- Authentication/authorization
- Rate limiting
- Multi-tenancy

---

## Success Criteria

1. Service builds and passes `cargo test`
2. All 5 API endpoints respond correctly
3. Notifications persist to PostgreSQL
4. Health check returns 200 OK
5. Structured logging with tracing
6. Docker container builds successfully
