# AlertHub Architecture

## Overview

Build AlertHub with a phased approach starting with core services and progressive feature rollout. Use proven technologies (TypeScript, React, Rust) with careful evaluation of Effect TypeScript in non-critical paths. Implement comprehensive monitoring from day one, design for graceful degradation, and maintain strong API contracts between services. Focus on MVP delivery with robust error handling and user experience fundamentals before scaling to full microservices architecture.

## Key Decisions

### Agreed Points
- Effect TypeScript needs careful evaluation due to emerging status and limited ecosystem
- Real-time WebSocket connections require robust error handling and reconnection logic
- API contracts must be defined early for parallel development coordination
- Comprehensive monitoring and observability are essential from the start
- User experience and frontend architecture deserve equal priority with backend services
- Modern cloud infrastructure provides auto-scaling capabilities to leverage

### Tradeoffs
- Start with monolithic core then extract services - delays microservices benefits but reduces integration complexity
- Use Effect TypeScript selectively in non-critical paths - limits innovation but reduces production risk
- Implement WebSocket with fallback to polling - ensures reliability while maintaining real-time experience
- Conservative initial scaling with monitoring-driven optimization - may over-provision but prevents outages
- Contract-first API development - slows initial development but prevents integration issues

## Tech Stack

- **Backend**: Rust (Notification Router), Bun/Elysia/Effect (Integration Service), Go/gRPC (Admin API)
- **Frontend**: Next.js 15 + React 19 + Effect (Web), Expo (Mobile), Electron (Desktop)
- **Infrastructure**: Kubernetes with CloudNative-PG, Strimzi Kafka, Percona MongoDB
- **Messaging**: Redis pub/sub for WebSocket scaling, Kafka for event streaming

## Services

| Service | Agent | Tech Stack | Priority |
|---------|-------|------------|----------|
| Notification Router | Rex | Rust/Axum | High |
| Integration Service | Nova | Bun/Elysia/Effect | High |
| Admin API | Grizz | Go/gRPC | High |
| Web Console | Blaze | Next.js/React/Effect | High |
| Mobile App | Tap | Expo/React Native | Medium |
| Desktop Client | Spark | Electron | Medium |
