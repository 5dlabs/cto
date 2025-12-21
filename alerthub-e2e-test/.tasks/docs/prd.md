# Project: AlertHub - Multi-Platform Notification System

## Vision
AlertHub is a comprehensive notification platform for web, mobile, and desktop.

## Features

### 1. Notification Router (Rust/Axum) - Agent: Rex
- POST /api/v1/notifications, GET /api/v1/notifications/:id, WS /api/v1/ws
- Rate limiting, priority queues, deduplication
- Deps: PostgreSQL, Redis, Kafka

### 2. Integration Service (Bun/Elysia + Effect) - Agent: Nova
- Slack, Discord, Email, Push, Webhook delivery
- Deps: MongoDB, RabbitMQ, Kafka

### 3. Admin API (Go/gRPC) - Agent: Grizz
- TenantService, UserService, RuleService, AnalyticsService
- Deps: PostgreSQL, Redis

### 4. Web Console (Next.js + Effect) - Agent: Blaze
- Next.js 15, React 19, shadcn/ui, TailwindCSS, Effect

### 5. Mobile App (Expo) - Agent: Tap
- Expo SDK 50+, NativeWind, push notifications

### 6. Desktop Client (Electron) - Agent: Spark
- Electron 28+, system tray, native notifications

### 7. Infrastructure - Agent: Bolt
- PostgreSQL, Redis, Kafka, MongoDB, RabbitMQ