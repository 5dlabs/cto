# AlertHub Design Brief

## Executive Summary

AlertHub is a comprehensive, multi-platform notification system demonstrating enterprise-grade architecture across polyglot microservices. The platform routes notifications through web, mobile, and desktop clients to external channels (Slack, Discord, email, push notifications) with intelligent routing, rate limiting, and user preferences.

**Key objectives:**
- Deliver 10,000 notifications/minute with <100ms p95 latency
- Maintain 99.9% uptime SLA
- Support 1,000 concurrent WebSocket connections
- Demonstrate multi-language backend orchestration (Rust, Bun/TypeScript, Go)
- Provide intuitive frontend experiences across all platforms
- Ensure GDPR compliance and enterprise security

**Platform scope:**
- 3 backend services (Rex, Nova, Grizz) + 3 frontend clients (Blaze, Tap, Spark)
- 7 infrastructure components (PostgreSQL, Redis, Kafka, MongoDB, RabbitMQ, SeaweedFS)
- Full deployment pipeline via Kubernetes/CRDs
- Production-grade observability (Prometheus, Grafana, structured logging)

---

## Key Design Decisions

### 1. **Polyglot Microservices Architecture**

**Decision:** Implement core services in different languages (Rust, TypeScript/Bun, Go)

**Rationale:**
- **Rust (Notification Router/Rex)**: High throughput, low latency, memory safety for critical path
- **TypeScript/Bun with Effect (Integration Service/Nova)**: Type-safe, composable error handling, rapid development
- **Go (Admin API/Grizz)**: gRPC efficiency, operational simplicity, strong standard library

**Tradeoffs:**
- ✓ Demonstrates full CTO platform capabilities
- ✓ Each service uses optimal language for its domain
- ✗ Operational complexity (multiple runtimes, dependency management)
- ✗ Knowledge barrier for unified teams

**Mitigation:**
- Standardized deployment patterns (Kubernetes for all)
- Clear service boundaries (API contracts via OpenAPI/gRPC)
- Shared observability (trace IDs, structured logging)

---

### 2. **Event-Driven Architecture with Dual Messaging**

**Decision:** Use Kafka for cross-service events + RabbitMQ for delivery tasks

**Rationale:**
- **Kafka**: Asymmetric publish-subscribe for event sourcing (notifications.created → multiple subscribers)
- **RabbitMQ**: Reliable task queues for deterministic delivery workflows

**Event Flow:**
```
Notification Router → Kafka → Integration Service → RabbitMQ → Channel Handlers
    (creates)          (events)    (consumes)        (tasks)    (executes)
```

**Tradeoffs:**
- ✓ Decoupled services, independent scaling
- ✓ Event replay capability for auditing
- ✓ Kafka partitions enable parallelism
- ✗ Added operational complexity (2 message brokers)
- ✗ Eventual consistency, not immediate

**Mitigation:**
- Strimzi + RabbitMQ Operator for automated management
- Monitoring of queue depths and consumer lag
- Idempotency keys for duplicate handling

---

### 3. **Effect TypeScript for Type-Safe Error Handling (Nova)**

**Decision:** Build Integration Service with Bun/Elysia + Effect instead of traditional Node.js

**Rationale:**
- Composable error handling with `Effect.catchTag()` for specific error types
- Dependency injection via `Layer` eliminates "callback hell"
- Type-safe retry strategies with `Effect.retry()` + `Schedule`
- Built-in resource management with `Effect.scoped()`

**Example:**
```typescript
// Traditional approach: callback nesting or Promise chains
deliver(integration) 
  .then(validateWebhook)
  .catch(handleError1)
  .then(retry)
  .catch(handleError2)

// Effect approach: composable, type-safe
const deliver = (integration: Integration) =>
  Effect.gen(function*() {
    const slack = yield* SlackService
    const result = yield* slack.deliver(integration)
    return result
  }).pipe(
    Effect.retry(Schedule.exponential("1s").pipe(Schedule.recurs(3))),
    Effect.catchTag("RateLimitError", (e) => Effect.succeed({ queued: true })),
    Effect.catchAll((e) => Effect.fail(new DeliveryError({ cause: e })))
  )
```

**Tradeoffs:**
- ✓ Eliminates entire classes of errors (uncaught promises, unhandled errors)
- ✓ Testable error paths without mocking
- ✓ Performance: minimal allocations, structured concurrency
- ✗ Learning curve (functional programming paradigm)
- ✗ Smaller ecosystem vs. Express/Fastify

**Mitigation:**
- Comprehensive documentation via Effect's LLM documentation
- Elysia's familiar decorator syntax reduces cognitive load
- Gradual migration path (Effect for service layer, traditional routes initially)

---

### 4. **Data Store Segmentation**

**Decision:** Use PostgreSQL, MongoDB, and Redis for different data access patterns

| Database | Data Type | Rationale |
|----------|-----------|-----------|
| **PostgreSQL** | Users, tenants, rules, notifications, audit logs | ACID guarantees, relational queries, compliance |
| **MongoDB** | Integration configs, templates, delivery logs | Schema flexibility, rich document queries, easy scaling |
| **Redis** | Rate limits, sessions, dedup cache, pub/sub | Sub-millisecond access, expiring keys, pub/sub |

**Tradeoffs:**
- ✓ Optimal storage for each access pattern
- ✓ Independent scaling (Redis for cache pressure)
- ✗ Distributed transactions (cross-database consistency)
- ✗ Operational overhead (3 databases to manage)

**Mitigation:**
- Event sourcing via Kafka for eventual consistency
- Clear ownership (Rex owns PostgreSQL notifications, Nova owns MongoDB integrations)
- CloudNative-PG and Percona operators automate backup/recovery

---

### 5. **WebSocket for Real-Time Updates (Rex)**

**Decision:** Implement stateful WebSocket connections in Notification Router for <500ms real-time updates

**Rationale:**
- Direct connection reduces latency vs. polling (1-5 second intervals)
- Server-initiated events (notification.delivered) reduce client complexity
- Redis pub/sub coordinates updates across router instances
- Automatic reconnection with exponential backoff

**Architecture:**
```
WebClient (Blaze) → Router (Rex)
                      ↓
                   Redis Pub/Sub (tenant:123)
                      ↑
            Integration Service (Nova) publishes events
```

**Tradeoffs:**
- ✓ Real-time user experience
- ✓ Reduces API polling load
- ✗ Stateful connections (harder to horizontally scale)
- ✗ Mobile platforms (battery drain from persistent connections)

**Mitigation:**
- Tokio's async I/O handles 10K concurrent connections on single instance
- Redis Pub/Sub pattern scales to multiple router instances
- Mobile app uses FCM push instead of WebSocket

---

### 6. **gRPC for Admin API (Grizz)**

**Decision:** Use gRPC + grpc-gateway for both gRPC clients and REST API users

**Rationale:**
- gRPC clients get low-latency binary serialization
- Web console clients use grpc-gateway-generated REST endpoints
- Protocol Buffers enforce strict API contracts
- Protoc validators prevent invalid data at API boundary

**Dual API Surface:**
```
Web Console (Blaze) → grpc-gateway → gRPC Server (Grizz)
Mobile App (Tap)    → REST API       ↓
Desktop (Spark)                    PostgreSQL
```

**Tradeoffs:**
- ✓ Single service implementation, multiple protocols
- ✓ Type-safe API contracts
- ✗ gRPC learning curve for REST-only developers
- ✗ Debugging complexity (binary format)

**Mitigation:**
- grpc-gateway handles REST translation automatically
- Evans CLI tool for interactive debugging
- Structured logging includes request/response bodies

---

### 7. **Kubernetes-First Infrastructure (Declarative CRDs)**

**Decision:** All stateful systems deployed via Kubernetes CRDs (CloudNative-PG, Strimzi, Percona, RabbitMQ Operator)

**Rationale:**
- GitOps compatibility (Infrastructure as Code)
- Automated backups and failover (CloudNative-PG handles PITR)
- Consistent observability (metrics from operators)
- Simplified upgrades (operator manages version lifecycle)

**Example:**
```yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: alerthub-postgres
spec:
  instances: 1  # → operator creates StatefulSet, PVC, Secrets
  bootstrap:
    initdb:
      database: alerthub
```

**Tradeoffs:**
- ✓ Zero-knowledge operator complexity hidden
- ✓ Declarative state for reproducibility
- ✗ Vendor lock-in to Kubernetes
- ✗ Debugging failures requires kubectl understanding

**Mitigation:**
- Operators are CNCF projects (portable across Kubernetes distributions)
- Fallback to manual management if needed
- Monitoring dashboards abstract operator internals

---

### 8. **Effect Schema for Validation (Frontend & Backend)**

**Decision:** Use Effect.Schema instead of Zod for runtime validation across web console (Blaze) and integration service (Nova)

**Rationale:**
- Single source of truth for request/response shapes
- Composable validators (reduce boilerplate)
- Built-in error messages with path information
- No additional dependency (native to Effect)

**Example:**
```typescript
// Shared between frontend form validation and backend API validation
const CreateIntegrationSchema = Schema.Struct({
  name: Schema.String.pipe(
    Schema.minLength(1, { message: "Name required" }),
    Schema.maxLength(100, { message: "Max 100 chars" })
  ),
  channel: Schema.Literal("slack", "discord", "email", "push", "webhook"),
  webhookUrl: Schema.optional(
    Schema.String.pipe(Schema.pattern(/^https?:\/\//))
  ),
  enabled: Schema.Boolean,
})

// Backend: Elysia POST handler
app.post("/integrations", async (ctx) => {
  const result = await Schema.decode(CreateIntegrationSchema)(ctx.body)
  // Type-safe: result has type CreateIntegration
})

// Frontend: React Hook Form with Effect resolver
const form = useForm({
  resolver: effectResolver(CreateIntegrationSchema),
  defaultValues: { name: "", channel: "slack", enabled: true },
})
```

**Tradeoffs:**
- ✓ Eliminates validator duplication
- ✓ Errors include field paths for forms
- ✗ Schema composition has learning curve
- ✗ Smaller community vs. Zod

**Mitigation:**
- Share schema packages via monorepo (Turborepo)
- Clear documentation with before/after examples

---

## Trade-offs Analysis

### Performance vs. Operational Complexity

| Choice | Performance | Complexity | Verdict |
|--------|-------------|-----------|---------|
| Rust Router | ✓✓✓ 10K msg/s | ✓ Low | Accept |
| Kafka + RabbitMQ | ✓✓ Scalable | ✗✗ High | Trade-off necessary for reliability |
| MongoDB for integrations | ✓✓ Flexible | ✓ Medium | Accept (schema evolution needed) |
| WebSocket for real-time | ✓✓✓ <500ms | ✓ Medium | Accept (kills polling load) |

**Verdict:** Performance wins justify operational investment; mitigated by operators.

---

### Development Speed vs. Type Safety

| Choice | Speed | Safety | Verdict |
|--------|-------|--------|---------|
| Effect TypeScript | ✓ (vs Go) | ✓✓✓ | Accept (catching errors early saves time) |
| gRPC + REST | ✓ (single impl) | ✓✓✓ | Accept (proto contracts prevent bugs) |
| Polyglot services | ✗ (learning curve) | ✓✓ | Trade-off (demonstrates platform) |

**Verdict:** Type safety early catches issues; reduces QA cycle time.

---

### Feature Richness vs. Launch Date

| Feature | Impact | Effort | Included? |
|---------|--------|--------|-----------|
| SMS notifications | Nice-to-have | 3 weeks | ✗ Non-goal |
| Multi-region replication | Enterprise | 4 weeks | ✗ Phase 2 |
| Voice notifications | Nice-to-have | 5 weeks | ✗ Non-goal |
| Custom notification sounds | User delight | 1 week | ✗ Non-goal |
| Self-hosted deployment | Enterprise | 2 weeks | ✗ Non-goal |

**Verdict:** Focus on core (notification routing + delivery); defer nice-to-haves.

---

## Risks

### Risk 1: WebSocket Scalability (MEDIUM)

**Risk:** WebSocket connections are stateful; scaling requires Redis pub/sub coordination or sticky sessions.

**Impact:** If single router instance crashes, 1,000 concurrent connections drop.

**Probability:** Medium (Tokio can handle 10K, but peak traffic spikes)

**Mitigation:**
- **Phase 1:** Single router instance with Redis pub/sub (scales to 10K concurrent)
- **Phase 2:** Multiple router instances behind sticky session load balancer
- **Phase 3:** Stateless mode with client-side reconnection + exponential backoff

**Acceptance:** Launch with single instance; monitor for connection limits.

---

### Risk 2: Effect TypeScript Ecosystem (MEDIUM)

**Risk:** Effect is newer; fewer third-party integrations (AWS SDK, payment processors) vs. Express/Fastify.

**Impact:** Custom adapters needed; slower feature development.

**Probability:** Medium (common integrations are covered; niche ones may not be)

**Mitigation:**
- Use `@effect/platform` HttpClient as universal adapter
- Implement custom Effect wrappers for 3rd-party libs
- Fallback to traditional Elysia handlers if Effect integration missing

**Acceptance:** Proceed; document custom integrations as we hit them.

---

### Risk 3: Data Consistency Across Services (MEDIUM)

**Risk:** Eventual consistency via Kafka events; client sees stale delivery status if notification service updates PostgreSQL before Kafka publishes.

**Impact:** Dashboard shows "Processing" for 1-2 seconds longer than actual (cosmetic but confusing).

**Probability:** Medium (network delays, database lag)

**Mitigation:**
- Idempotency keys prevent duplicate processing
- Notification ID doubles as event correlation ID
- WebSocket subscription includes polling fallback (poll every 5s if no events)

**Acceptance:** Document eventual consistency for UI developers.

---

### Risk 4: MongoDB Schema Evolution (LOW)

**Risk:** Flexible schema in MongoDB can lead to inconsistent integration configs (missing required fields).

**Rationale:** Avoid by using Drizzle ORM type guards + Effect Schema validation.

**Impact:** Runtime errors in Nova when accessing undefined fields.

**Probability:** Low (validation enforced at API boundary)

**Mitigation:**
- Drizzle Schema defines required fields
- Effect Schema validates on read
- Migrations track schema versions

**Acceptance:** Accept; migrations document schema changes.

---

### Risk 5: gRPC Binary Protocol Debugging (LOW)

**Risk:** Debugging gRPC requests is harder than REST (binary format, no curl).

**Impact:** Slower incident resolution, operational training overhead.

**Probability:** Low (grpc-gateway REST layer available)

**Mitigation:**
- grpc-gateway provides REST API for manual testing
- Evans CLI enables interactive gRPC debugging
- Comprehensive logging includes decoded request/response

**Acceptance:** Accept; document debugging procedures.

---

### Risk 6: PostgreSQL as Bottleneck (MEDIUM)

**Risk:** If notification volume scales to 100K/min, single PostgreSQL instance becomes I/O bottleneck.

**Impact:** Insert latency >100ms p95, SLA breach.

**Probability:** Medium (depends on growth; current 10K/min is safe)

**Mitigation:**
- **Phase 1:** Single PostgreSQL instance with async inserts via batch writes
- **Phase 2:** Write-ahead log to object storage; batch aggregation
- **Phase 3:** Partit
