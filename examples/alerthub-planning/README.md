# AlertHub - Multi-Platform Notification System

A comprehensive notification platform that routes alerts across web, mobile, and desktop clients.

## Project Structure

```
alerthub-e2e-test/
├── .tasks/
│   ├── tasks.json          # Task manifest
│   └── docs/
│       ├── prd.md          # Product Requirements
│       ├── architecture.md # Architecture decisions
│       └── task-N/         # Individual task docs
│           ├── task.md     # Task specification
│           └── prompt.md   # Implementation prompt
├── services/               # Backend services (TBD)
├── apps/                   # Frontend apps (TBD)
└── infra/                  # Infrastructure (TBD)
```

## Generated Tasks

This project was planned using the CTO intake-agent with multi-agent debate:
- **Proposer**: Claude Sonnet (optimist perspective)
- **Critic**: MiniMax M2.1 (pessimist perspective)  
- **Synthesizer**: Claude Sonnet (consensus builder)

See `.tasks/docs/architecture.md` for key architectural decisions.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Notification Router | Rust/Axum |
| Integration Service | Bun/Elysia/Effect |
| Admin API | Go/gRPC |
| Web Console | Next.js 15/React 19/Effect |
| Mobile App | Expo/React Native |
| Desktop Client | Electron |

## Getting Started

1. Review the PRD: `.tasks/docs/prd.md`
2. Check task dependencies: `.tasks/tasks.json`
3. Start with high-priority tasks that have no dependencies

## Task Status

| ID | Task | Priority | Dependencies |
|----|------|----------|--------------|
| 1 | API Contract Definition | high | - |
| 2 | Core Monolith with Service Boundaries | high | 1 |
| 3 | Monitoring and Observability Setup | high | - |
| 4 | External API Circuit Breakers | medium | 2, 3 |
| 5 | WebSocket Real-time Infrastructure | medium | 2 |
| 6 | Web Console Application | medium | 5 |
| 7 | Mobile Application | medium | 4 |
| 8 | Desktop Client | low | 6 |
| 9 | Feature Flag System | medium | 2 |
| 10 | Load Testing and Optimization | low | 5, 3 |
