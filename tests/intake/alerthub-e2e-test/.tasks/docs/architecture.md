# AlertHub Architecture

## System Overview

AlertHub is a multi-platform notification system with microservices architecture.

## Services

| Service | Agent | Stack | Purpose |
|---------|-------|-------|---------|
| Notification Router | Rex | Rust/Axum | High-perf routing, rate limiting |
| Integration Service | Nova | Bun/Elysia+Effect | Channel delivery (Slack, Discord, etc) |
| Admin API | Grizz | Go/gRPC | Tenant/user management |
| Web Console | Blaze | Next.js 15/React 19 | Admin dashboard |
| Mobile App | Tap | Expo/React Native | Push notifications |
| Desktop Client | Spark | Electron | System tray notifications |
| Infrastructure | Bolt | Kubernetes | Deployment, CRDs |

## Data Flow

```
Client → Notification Router (Rex) → Kafka → Integration Service (Nova) → Channels
                ↓
            PostgreSQL
```

## Infrastructure

- PostgreSQL (CloudNative-PG): Primary data store
- Redis/Valkey: Rate limiting, cache
- Kafka (Strimzi): Event streaming
- MongoDB (Percona): Integration configs
- RabbitMQ: Task queues
- SeaweedFS: Object storage

## Key Patterns

- Effect TypeScript for type-safe error handling (Nova, Blaze)
- gRPC with grpc-gateway for REST (Grizz)
- WebSocket for real-time updates
- JWT authentication with RBAC
