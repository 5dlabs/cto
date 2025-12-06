# TeamSync API Architecture

## System Components

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   React UI  │────▶│  Axum API   │────▶│  PostgreSQL │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                    ┌──────▼──────┐
                    │    Redis    │
                    │ (cache/pub) │
                    └─────────────┘
```

## Tech Stack
- **API**: Rust + Axum 0.7
- **Database**: PostgreSQL 15 + sqlx
- **Cache**: Redis 7
- **Frontend**: React 18 + TypeScript + Tailwind

## Key Design Decisions
1. **Stateless API** - JWT auth, no server sessions
2. **Event-driven notifications** - Redis pub/sub for real-time
3. **Soft deletes** - 30-day retention with scheduled cleanup
4. **Rate limiting** - Token bucket via Redis

## Directory Structure
```
/
├── src/
│   ├── api/         # HTTP handlers
│   ├── domain/      # Business logic
│   ├── infra/       # Database, Redis
│   └── main.rs
├── frontend/        # React app
└── infra/           # Docker, K8s manifests
```
