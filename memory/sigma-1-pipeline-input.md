# Sigma-1 Pipeline Input Summary

This document contains all necessary information to run the intake pipeline for the Sigma-1 project once credentials are available.

## Project Information

**Project Name:** sigma-1
**Repository URL:** https://github.com/5dlabs/sigma-1
**Organization:** 5dlabs
**Visibility:** private

## PRD Content (First 50 lines)

```
# Project: Sigma-1 — Unified AI Business Platform for Perception Events

- **Website:** https://sigma-1.com
- **Existing Platform:** https://deployiq.maximinimal.ca

## Vision

Sigma-1 is a lighting and visual production company (Perception Events). This platform replaces their fragmented tools, manual processes, and administrative overhead with a single intelligent agent — **Morgan** — accessible through Signal, phone, and web.

Instead of juggling rental software, spreadsheets, phone calls, accounting tools, and social media apps, everything runs through one interface: send Morgan a message, and it handles the rest.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Sigma-1 Platform                                 │
├─────────────────────────────────────────────────────────────────────┤
│  Clients                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Signal  │  │   Voice  │  │   Web    │  │  Mobile  │        │
│  │  (Morgan)│  │ (ElevenLabs│ │ (Next.js)│  │  (Expo)  │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │             │             │                 │
├───────┴─────────────┴─────────────┴─────────────┴─────────────────┤
│  Backend Services                                                    │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
│  │   Equipment    │  │     RMS        │  │    Finance     │      │
│  │   Catalog      │  │   Service      │  │    Service     │      │
│  │   (Rust/Axum)  │  │   (Go/gRPC)    │  │   (Rust/Axum)  │      │
│  │     Rex        │  │     Grizz      │  │     Rex        │      │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘      │
│          │                    │                    │                 │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
│  │   (Out of      │  │     Social     │  │    Customer    │      │
│  │    Scope)      │  │    Engine      │  │    Vetting     │      │
│  │  (Phase 2)    │  │(Node/Elysia)  │  │  (Rust/Axum)   │      │
│  │                │  │     Nova       │  │     Rex        │      │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘      │
│          │                    │                    │                 │
├──────────┴────────────────────┴────────────────────┴─────────────────┤
│  Infrastructure                                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │
│  │PostgreSQL│ │  Redis  │ │  S3/R2  │ │ ElevenLabs│ │ Twilio  │     │
│  │         │ │         │ │         │ │          │ │         │     │
│  └─────────┘ └─────────┘ └─────────┘ └──────────┘ └─────────┘     │
│  ┌─────────┐ ┌─────────┐                                             │
│  │SignalCLI│ │OpenCorporates│                                         │
│  └─────────┘ └─────────┘                                             │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Design Brief

Create a unified AI business platform for Perception Events that consolidates all business operations into a single intelligent agent called Morgan. The platform should integrate with existing tools and provide a seamless experience across Signal messenger, voice calls, web interface, and mobile applications.

## Core Requirements

1. **Morgan AI Agent** - Central intelligent agent accessible through multiple channels
2. **Equipment Catalog Service** - High-performance product inventory system
3. **Rental Management Service** - Calendar-aware booking and reservation system
4. **Finance Service** - Quote generation, invoicing, and financial reporting
5. **Integration** - Seamless communication between all services
6. **Security** - Enterprise-grade security and authentication
7. **Scalability** - Support for high-concurrency operations
8. **Observability** - Comprehensive monitoring and logging

## Technical Stack

- **Languages:** Rust, Go, Node.js
- **Frameworks:** Axum, gRPC, Elysia
- **Infrastructure:** Kubernetes, PostgreSQL, Redis, S3/R2
- **APIs:** ElevenLabs, Twilio, Signal-CLI
- **Platforms:** Linear (project management), Discord (communication)

## Success Criteria

- Single interface for all business operations
- Reduced administrative overhead by 80%
- Improved customer response time to under 5 minutes
- 99.9% system uptime
- Support for 1000+ concurrent users

## Implementation Constraints

- Must integrate with existing Perception Events workflows
- Should leverage existing equipment database (533+ products)
- Must maintain compatibility with current accounting systems
- Need to support both web and mobile experiences
- Voice interface must provide natural conversation capabilities

## Future Enhancements (Phase 2)

- Social Media Engine for content curation and publishing
- Advanced Customer Vetting Service with background checks
- Machine learning for predictive analytics and recommendations
- Advanced reporting and business intelligence features