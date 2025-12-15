# Project: AlertHub Lite - Simplified Notification System

## Vision

AlertHub Lite is a minimal notification platform with a Rust backend and React frontend. This PRD tests the CTO platform E2E workflow with the smallest viable scope: one backend service, one frontend app, and basic infrastructure.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      AlertHub Lite                          │
├─────────────────────────────────────────────────────────────┤
│  Frontend                                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Web Console (React/Next.js)             │  │
│  │                      Blaze Agent                     │  │
│  └───────────────────────────┬──────────────────────────┘  │
│                              │                              │
├──────────────────────────────┼──────────────────────────────┤
│  Backend                     │                              │
│  ┌───────────────────────────┴──────────────────────────┐  │
│  │           Notification Service (Rust/Axum)           │  │
│  │                      Rex Agent                       │  │
│  └───────────────────────────┬──────────────────────────┘  │
│                              │                              │
├──────────────────────────────┼──────────────────────────────┤
│  Infrastructure              │                              │
│  ┌─────────────┐    ┌────────┴────────┐                    │
│  │ PostgreSQL  │    │     Redis       │                    │
│  │ (CNPG)      │    │    (Valkey)     │                    │
│  └─────────────┘    └─────────────────┘                    │
│                      Bolt Agent                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Features

### 1. Notification Service (Rust/Axum)

**Agent**: Rex
**Priority**: High
**Language**: Rust 1.75+
**Framework**: Axum 0.7

A simple notification service that stores and retrieves notifications.

**Endpoints**:
- `POST /api/v1/notifications` - Create a notification
- `GET /api/v1/notifications` - List notifications (paginated)
- `GET /api/v1/notifications/:id` - Get notification by ID
- `DELETE /api/v1/notifications/:id` - Delete notification
- `GET /health` - Health check

**Core Features**:
- Store notifications in PostgreSQL
- Cache recent notifications in Redis
- Rate limiting (10 requests/second per IP)
- Structured JSON logging with tracing

**Data Models**:
```rust
struct Notification {
    id: Uuid,
    title: String,
    message: String,
    priority: Priority, // low, normal, high, critical
    created_at: DateTime<Utc>,
    read: bool,
}

enum Priority {
    Low,
    Normal,
    High,
    Critical,
}
```

**Infrastructure Dependencies**:
- PostgreSQL: Notification storage
- Redis: Recent notifications cache, rate limiting

---

### 2. Web Console (React/Next.js)

**Agent**: Blaze
**Priority**: High
**Stack**: Next.js 14+ App Router, React 18, shadcn/ui, TailwindCSS

A simple web interface to view and manage notifications.

**Pages**:
- `/` - Dashboard with notification list
- `/notifications/:id` - Notification detail view
- `/settings` - Basic settings page

**Core Features**:
- Dark/light theme toggle
- Real-time notification list (polling every 5s)
- Create notification form
- Mark as read/unread
- Delete notifications
- Toast notifications for actions

**Key Components**:
- `<NotificationList />` - Paginated list of notifications
- `<NotificationCard />` - Single notification display
- `<CreateNotificationForm />` - Form to create new notification
- `<ThemeToggle />` - Dark/light mode switch

**State Management**:
- TanStack Query for server state
- React Hook Form for forms

---

### 3. Infrastructure Setup

**Agent**: Bolt
**Priority**: High

Provision the required infrastructure using Kubernetes operators.

**Components**:

```yaml
# PostgreSQL (CloudNative-PG)
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: alerthub-db
  namespace: alerthub
spec:
  instances: 1
  storage:
    size: 1Gi
  bootstrap:
    initdb:
      database: alerthub
      owner: alerthub_user

# Redis/Valkey
apiVersion: redis.redis.opstreelabs.in/v1beta2
kind: Redis
metadata:
  name: alerthub-cache
  namespace: alerthub
spec:
  kubernetesConfig:
    image: valkey/valkey:7.2-alpine
  storage:
    volumeClaimTemplate:
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 1Gi
```

**Output**:
Create ConfigMap `alerthub-infra-config` with connection details for other agents:
- `DATABASE_URL`
- `REDIS_URL`

---

## Technical Context

| Component | Technology | Version |
|-----------|------------|---------|
| Backend | Rust, Axum, tokio, sqlx | Rust 1.75+, Axum 0.7 |
| Frontend | Next.js, React, shadcn/ui | Next.js 14+ |
| Database | PostgreSQL (CloudNative-PG) | PostgreSQL 15 |
| Cache | Redis/Valkey | Valkey 7.2 |

---

## Constraints

- API response time < 200ms p95
- Support 100 notifications/minute
- Single environment (no staging/production split)

---

## Non-Goals

- Authentication/authorization (all endpoints public for testing)
- Multiple tenants
- Real-time WebSocket updates
- Mobile app
- Email/push delivery

---

## Success Criteria

1. Backend service compiles and passes tests
2. Frontend app builds successfully
3. Infrastructure provisioned and healthy
4. API endpoints work correctly
5. Frontend can list/create/delete notifications
6. All Linear issues show activity updates

