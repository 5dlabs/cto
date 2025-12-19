# Linear Issue Content for AlertHub E2E Test

## Issue Title
`[PRD] AlertHub - Multi-Platform Notification System`

## Labels
- `prd` (required for intake trigger)

## Description

**Repository:** https://github.com/5dlabs/alerthub-e2e-test

---

## PRD

### Vision

AlertHub is a comprehensive notification platform that routes alerts across web, mobile, and desktop clients. It supports multiple delivery channels (Slack, Discord, email, push notifications) with intelligent routing, rate limiting, and user preferences. Built as a microservices architecture to demonstrate multi-agent orchestration across different tech stacks.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         AlertHub Platform                            │
├─────────────────────────────────────────────────────────────────────┤
│  Clients                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                          │
│  │   Web    │  │  Mobile  │  │ Desktop  │                          │
│  │ (Blaze)  │  │  (Tap)   │  │ (Spark)  │                          │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                          │
│       │             │             │                                  │
├───────┴─────────────┴─────────────┴─────────────────────────────────┤
│  Backend Services                                                    │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐        │
│  │  Notification  │  │  Integration   │  │    Admin       │        │
│  │    Router      │  │    Service     │  │     API        │        │
│  │    (Rex)       │  │    (Nova)      │  │   (Grizz)      │        │
│  │   Rust/Axum    │  │ Bun/Elysia+Eff │  │    Go/gRPC     │        │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘        │
│          │                   │                   │                  │
├──────────┴───────────────────┴───────────────────┴──────────────────┤
│  Infrastructure                                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │PostgreSQL│ │  Redis  │ │  Kafka  │ │   S3    │ │ MongoDB │       │
│  │         │ │         │ │         │ │SeaweedFS│ │         │       │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘       │
│  ┌─────────┐                                                        │
│  │RabbitMQ │                                                        │
│  └─────────┘                                                        │
└─────────────────────────────────────────────────────────────────────┘
```

### Features Summary

1. **Notification Router Service (Rex - Rust/Axum)** - High-performance core for receiving, validating, and routing notifications
2. **Integration Service (Nova - Bun/Elysia+Effect)** - Channel delivery with type-safe error handling
3. **Admin API (Grizz - Go/gRPC)** - Tenant/user management with RBAC
4. **Web Console (Blaze - Next.js+Effect)** - Primary configuration interface
5. **Mobile App (Tap - Expo)** - Push notifications and preferences
6. **Desktop Client (Spark - Electron)** - System tray notifications

### Technical Stack

| Component | Technology | Agent |
|-----------|------------|-------|
| Notification Router | Rust, Axum, tokio, sqlx | Rex |
| Integration Service | Bun, Elysia, Effect, Drizzle | Nova |
| Admin API | Go, gRPC, grpc-gateway | Grizz |
| Web Console | Next.js, React, shadcn/ui, Effect | Blaze |
| Mobile App | Expo, React Native, NativeWind | Tap |
| Desktop Client | Electron, React | Spark |

### Infrastructure Requirements (Bolt)

- PostgreSQL (CloudNative-PG)
- Redis/Valkey (Redis Operator)
- Kafka (Strimzi)
- MongoDB (Percona)
- RabbitMQ (RabbitMQ Operator)
- SeaweedFS (S3-compatible storage)

### Success Criteria

1. All backend services build, pass tests, and deploy successfully
2. All frontend applications build and deploy successfully
3. End-to-end notification flow works (submit → route → deliver → display)
4. Infrastructure operators provision resources correctly
5. WebSocket real-time updates function across web and desktop
6. Mobile push notifications deliver successfully
7. Admin API CRUD operations work correctly
8. Monitoring dashboards show accurate metrics

---

## Architecture Reference

See full architecture document: https://github.com/5dlabs/cto/blob/main/tests/intake/alerthub-e2e-test/architecture.md

---

## Notes for Agent (Talos)

This PRD is designed to exercise **all 12 CTO platform agents**:

**Backend Implementation:**
- Rex (Rust) - Notification Router
- Grizz (Go) - Admin API  
- Nova (Bun/Elysia+Effect) - Integration Service

**Frontend Implementation:**
- Blaze (Next.js+Effect) - Web Console
- Tap (Expo) - Mobile App
- Spark (Electron) - Desktop Client

**Infrastructure:**
- Bolt - PostgreSQL, Redis, Kafka, MongoDB, RabbitMQ, SeaweedFS

**Support Agents:**
- Cleo - Quality review (per language)
- Cipher - Security analysis (per language)
- Tess - Testing (per language)
- Atlas - Integration and merge

**Effect TypeScript:**
Both Nova and Blaze must use Effect (https://effect.website/) for:
- Type-safe error handling
- Schema validation (replacing Zod)
- Service composition
- Retry logic with schedules

Generate cto-config.json with:
```json
{
  "agents": {
    "nova": { "stack": "elysia-effect", "tools": { "hints": ["elysia", "effect", "drizzle"] } },
    "blaze": { "stack": "shadcn-effect", "tools": { "hints": ["nextjs", "effect", "tailwindcss", "shadcn"] } }
  }
}
```


