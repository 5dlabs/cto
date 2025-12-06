# Project: TeamSync API

## Vision
A lightweight team collaboration API that enables real-time task management and notifications. Built with Rust/Axum for performance, with a React dashboard for team leads.

## Features

### 1. Team Management
- POST /api/teams - Create team with name, description
- GET /api/teams/:id - Get team details with member count
- PATCH /api/teams/:id - Update team settings
- POST /api/teams/:id/invite - Generate invite link (expires in 7 days)
- **Priority**: High

### 2. Task Board
- CRUD operations for tasks within a team
- Tasks have: title, description, assignee, status (todo/in-progress/done), due_date
- Filter tasks by status, assignee, due date range
- Soft delete with 30-day retention
- **Priority**: High

### 3. Authentication & Authorization
- JWT-based auth with refresh tokens
- OAuth2 integration (Google, GitHub)
- Role-based permissions: owner, admin, member, viewer
- Rate limiting: 100 req/min for authenticated, 20 req/min for anonymous
- **Priority**: High

### 4. Real-time Notifications
- WebSocket endpoint for live task updates
- Push notification integration (FCM for mobile)
- Email notifications for mentions and due date reminders
- User preference controls for notification types
- **Priority**: Medium

### 5. Team Dashboard (React)
- Kanban board view with drag-and-drop
- Team activity feed
- Member management UI
- Dark/light theme support
- **Priority**: Medium

### 6. Deployment & Observability
- Docker multi-stage build
- Kubernetes manifests with HPA
- Prometheus metrics endpoint
- Structured JSON logging with trace IDs
- Health check endpoints (liveness, readiness)
- **Priority**: High

## Technical Context
- **Language**: Rust 1.75+
- **Framework**: Axum 0.7
- **Database**: PostgreSQL 15 with sqlx
- **Cache**: Redis for sessions and rate limiting
- **Frontend**: React 18 + TypeScript + Tailwind
- **Message Queue**: Redis pub/sub for notifications

## Constraints
- API response time < 100ms p95
- Support 1000 concurrent WebSocket connections
- GDPR compliant (data export, deletion)
- Mobile-responsive dashboard

## Non-Goals
- Native mobile apps (use responsive web)
- Video/voice chat
- File attachments > 10MB
- Self-hosted deployment docs



