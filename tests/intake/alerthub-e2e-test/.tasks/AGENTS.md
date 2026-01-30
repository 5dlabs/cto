# AlertHub Agent Context

## Project Overview

AlertHub is a multi-platform notification system with microservices architecture.
This project demonstrates multi-agent orchestration across different tech stacks.

## Architecture Summary

**Data Flow:**
```
Client → Notification Router (Rex/Rust) → Kafka → Integration Service (Nova/Bun) → Channels
```

**Services:**
| Service | Agent | Stack | Status |
|---------|-------|-------|--------|
| Infrastructure | Bolt | Kubernetes | Pending |
| Notification Router | Rex | Rust/Axum | Pending |
| Integration Service | Nova | Bun/Elysia+Effect | Pending |
| Admin API | Grizz | Go/gRPC | Pending |
| Web Console | Blaze | Next.js 15/React 19 | Pending |
| Mobile App | Tap | Expo/React Native | Pending |
| Desktop Client | Spark | Electron | Pending |

## Key Technologies

- **Effect TypeScript**: Used in Nova and Blaze for type-safe error handling
- **gRPC + grpc-gateway**: Admin API provides both gRPC and REST
- **WebSocket**: Real-time updates from Rex to web/desktop clients
- **Kafka**: Event streaming between services
- **Kubernetes Operators**: CloudNative-PG, Strimzi, Percona for databases

## Constraints

- API response time < 100ms p95
- 10,000 notifications/minute throughput
- 1,000 concurrent WebSocket connections
- 99.9% uptime SLA

## Current Phase: Intake

Tasks have been generated from the PRD. Ready for execution phase.

## Notes

- Effect.website/llms.txt for Effect patterns
- Mobile uses FCM/APNs for push
- Desktop cross-platform: Windows, macOS, Linux
