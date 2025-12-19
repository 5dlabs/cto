# Project: AlertHub - Multi-Platform Notification System

## Vision

AlertHub is a comprehensive notification platform that routes alerts across web, mobile, and desktop clients. It supports multiple delivery channels (Slack, Discord, email, push notifications) with intelligent routing, rate limiting, and user preferences. Built as a microservices architecture to demonstrate multi-agent orchestration across different tech stacks.

## Architecture Overview

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

---

## Features

### 1. Notification Router Service (Rust/Axum)

**Agent**: Rex  
**Priority**: High  
**Language**: Rust 1.75+  
**Framework**: Axum 0.7

The high-performance core that receives, validates, and routes notifications.

**Endpoints**:
- `POST /api/v1/notifications` - Submit a new notification
- `POST /api/v1/notifications/batch` - Submit batch notifications (up to 100)
- `GET /api/v1/notifications/:id` - Get notification status
- `GET /api/v1/notifications/:id/events` - Get delivery events
- `WS /api/v1/ws` - WebSocket for real-time notification updates

**Core Features**:
- Rate limiting per tenant (configurable via Redis)
- Priority queue processing (critical, high, normal, low)
- Deduplication with configurable TTL
- Dead letter queue for failed deliveries
- Prometheus metrics endpoint (`/metrics`)
- Health checks (`/health/live`, `/health/ready`)

**Data Models**:
```rust
struct Notification {
    id: Uuid,
    tenant_id: Uuid,
    channel: Channel, // slack, discord, email, push, webhook
    priority: Priority,
    payload: NotificationPayload,
    metadata: HashMap<String, Value>,
    created_at: DateTime<Utc>,
    status: NotificationStatus,
}

enum NotificationStatus {
    Pending,
    Processing,
    Delivered,
    Failed { reason: String, attempts: u32 },
}
```

**Infrastructure Dependencies**:
- PostgreSQL: Notification persistence, tenant data
- Redis: Rate limiting, deduplication cache
- Kafka: Event streaming to integration service

---

### 2. Integration Service (Bun/Elysia + Effect)

**Agent**: Nova  
**Priority**: High  
**Runtime**: Bun 1.1+  
**Framework**: Elysia 1.x with Effect TypeScript

Handles delivery to external channels (Slack, Discord, email, webhooks). Built with **Effect** for type-safe error handling, composable services, and robust retry logic.

**AI Documentation**: Reference `https://effect.website/llms.txt` for Effect patterns.

**Endpoints**:
- `POST /api/v1/integrations` - Create new integration
- `GET /api/v1/integrations` - List integrations for tenant
- `GET /api/v1/integrations/:id` - Get integration details
- `PATCH /api/v1/integrations/:id` - Update integration
- `DELETE /api/v1/integrations/:id` - Delete integration
- `POST /api/v1/integrations/:id/test` - Test integration connectivity

**Supported Channels**:
- **Slack**: Incoming webhooks, Bot API
- **Discord**: Webhooks
- **Email**: SMTP (SendGrid, AWS SES)
- **Push**: FCM for mobile
- **Webhook**: Custom HTTP endpoints

**Core Features**:
- **Effect Services** for channel integrations (SlackService, DiscordService, EmailService)
- **Effect.retry** with exponential backoff schedule for delivery
- **Effect Schema** for request/response validation (replaces Zod)
- **@effect/platform** HttpClient for outbound requests
- Template rendering with Effect error handling
- Rate limiting per channel via Effect Semaphore
- OAuth2 token refresh with Effect.cached
- Webhook signature verification

**Technology Stack**:
| Component | Technology |
|-----------|------------|
| Runtime | Bun 1.1+ |
| Framework | Elysia 1.x |
| Type System | Effect + TypeScript 5.x |
| Validation | Effect Schema |
| HTTP Client | @effect/platform |
| Database | Drizzle ORM (MongoDB) |
| Queue Consumer | Effect Stream + kafkajs |

**Data Models** (Effect Schema):
```typescript
import { Schema } from "effect"

// Channel types as literal union
const Channel = Schema.Literal("slack", "discord", "email", "push", "webhook")

// Integration schema with Effect
const Integration = Schema.Struct({
  id: Schema.String,
  tenantId: Schema.String,
  channel: Channel,
  name: Schema.String,
  config: Schema.Union(
    Schema.Struct({ webhookUrl: Schema.String, channel: Schema.optional(Schema.String) }), // Slack
    Schema.Struct({ webhookUrl: Schema.String }), // Discord
    Schema.Struct({ smtpHost: Schema.String, smtpPort: Schema.Number, fromAddress: Schema.String }), // Email
    Schema.Struct({ url: Schema.String, headers: Schema.optional(Schema.Record({ key: Schema.String, value: Schema.String })), secret: Schema.optional(Schema.String) }) // Webhook
  ),
  enabled: Schema.Boolean,
  createdAt: Schema.Date,
  updatedAt: Schema.Date,
})
type Integration = Schema.Schema.Type<typeof Integration>

// Error types as tagged unions
class SlackDeliveryError extends Schema.TaggedError<SlackDeliveryError>("SlackDeliveryError")({
  message: Schema.String,
  statusCode: Schema.optional(Schema.Number),
}) {}

class RateLimitError extends Schema.TaggedError<RateLimitError>("RateLimitError")({
  retryAfter: Schema.Number,
}) {}
```

**Effect Service Pattern**:
```typescript
import { Context, Effect, Layer, Schedule } from "effect"
import { HttpClient } from "@effect/platform"

// Define service interface
class SlackService extends Context.Tag("SlackService")<
  SlackService,
  {
    deliver: (integration: Integration, message: Message) => Effect.Effect<DeliveryResult, SlackDeliveryError>
  }
>() {}

// Implement with retry logic
const SlackServiceLive = Layer.succeed(
  SlackService,
  SlackService.of({
    deliver: (integration, message) =>
      HttpClient.request.post(integration.config.webhookUrl).pipe(
        HttpClient.request.jsonBody(message),
        HttpClient.client.fetchOk,
        Effect.retry(Schedule.exponential("1 second").pipe(Schedule.compose(Schedule.recurs(3)))),
        Effect.mapError((e) => new SlackDeliveryError({ message: e.message }))
      )
  })
)
```

**Elysia Route Handler**:
```typescript
import { Elysia } from "elysia"
import { Effect, Layer } from "effect"

const app = new Elysia()
  .post("/api/v1/integrations/:id/deliver", async ({ params, body }) => {
    const program = Effect.gen(function* () {
      const slack = yield* SlackService
      const result = yield* slack.deliver(integration, body)
      return result
    })
    
    return Effect.runPromise(
      program.pipe(Effect.provide(SlackServiceLive))
    )
  })
```

**Infrastructure Dependencies**:
- MongoDB: Integration configs, templates (flexible schema)
- RabbitMQ: Task queue for delivery jobs (consumed via Effect Stream)
- Kafka: Consume notification events (Effect Stream adapter)

---

### 3. Admin API (Go/gRPC)

**Agent**: Grizz  
**Priority**: High  
**Language**: Go 1.22+  
**Framework**: gRPC with grpc-gateway for REST

Management API for tenants, users, rules, and analytics.

**gRPC Services**:
```protobuf
service TenantService {
  rpc CreateTenant(CreateTenantRequest) returns (Tenant);
  rpc GetTenant(GetTenantRequest) returns (Tenant);
  rpc UpdateTenant(UpdateTenantRequest) returns (Tenant);
  rpc ListTenants(ListTenantsRequest) returns (ListTenantsResponse);
}

service UserService {
  rpc CreateUser(CreateUserRequest) returns (User);
  rpc GetUser(GetUserRequest) returns (User);
  rpc UpdateUser(UpdateUserRequest) returns (User);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  rpc UpdatePreferences(UpdatePreferencesRequest) returns (UserPreferences);
}

service RuleService {
  rpc CreateRule(CreateRuleRequest) returns (NotificationRule);
  rpc GetRule(GetRuleRequest) returns (NotificationRule);
  rpc UpdateRule(UpdateRuleRequest) returns (NotificationRule);
  rpc ListRules(ListRulesRequest) returns (ListRulesResponse);
  rpc DeleteRule(DeleteRuleRequest) returns (Empty);
}

service AnalyticsService {
  rpc GetNotificationStats(StatsRequest) returns (NotificationStats);
  rpc GetDeliveryMetrics(MetricsRequest) returns (DeliveryMetrics);
}
```

**REST Endpoints** (via grpc-gateway):
- `POST /api/v1/tenants` - Create tenant
- `GET /api/v1/tenants/:id` - Get tenant
- `GET /api/v1/users` - List users
- `POST /api/v1/rules` - Create notification rule
- `GET /api/v1/analytics/stats` - Get notification statistics

**Core Features**:
- JWT authentication with refresh tokens
- Role-based access control (owner, admin, member, viewer)
- Notification rules engine (filter by source, type, severity)
- Analytics aggregation (daily/weekly/monthly)
- Audit logging

**Data Models**:
```go
type Tenant struct {
    ID          uuid.UUID
    Name        string
    Plan        string // free, pro, enterprise
    Settings    TenantSettings
    CreatedAt   time.Time
    UpdatedAt   time.Time
}

type NotificationRule struct {
    ID          uuid.UUID
    TenantID    uuid.UUID
    Name        string
    Conditions  []RuleCondition
    Actions     []RuleAction
    Enabled     bool
    Priority    int
}

type RuleCondition struct {
    Field    string // source, type, severity, metadata.*
    Operator string // eq, ne, gt, lt, contains, regex
    Value    string
}
```

**Infrastructure Dependencies**:
- PostgreSQL: Tenants, users, rules, audit logs
- Redis: Session cache, analytics aggregation

---

### 4. Web Console (React/Next.js + Effect)

**Agent**: Blaze  
**Priority**: High  
**Stack**: Next.js 15 App Router, React 19, shadcn/ui, TailwindCSS, Effect

The primary configuration interface for AlertHub. Built with **Effect** for type-safe data fetching, error handling, and schema validation.

**AI Documentation**: Reference `https://effect.website/llms.txt` for Effect patterns.

**Pages**:
- `/` - Dashboard with notification overview
- `/notifications` - Notification history with filters
- `/integrations` - Manage channel integrations
- `/rules` - Configure notification rules
- `/settings` - Tenant and user settings
- `/analytics` - Delivery metrics and charts

**Core Features**:
- Dark/light theme support
- Real-time notification feed (WebSocket with Effect Stream)
- Drag-and-drop rule builder
- Integration wizard with OAuth flows
- Responsive design (mobile-friendly)
- Toast notifications for actions (Effect error mapping)

**Technology Stack**:
| Component | Technology |
|-----------|------------|
| Framework | Next.js 15 App Router |
| UI Library | React 19 |
| Components | shadcn/ui |
| Styling | TailwindCSS 4 |
| Type System | Effect + TypeScript 5.x |
| Validation | Effect Schema |
| Data Fetching | TanStack Query + Effect |
| Forms | React Hook Form + Effect Schema resolvers |
| Animations | anime.js |

**Key Components**:
- `<NotificationFeed />` - Real-time notification list (Effect Stream for WebSocket)
- `<IntegrationCard />` - Integration status and actions
- `<RuleBuilder />` - Visual rule configuration
- `<AnalyticsChart />` - Delivery metrics visualization (recharts)
- `<SettingsForm />` - User/tenant preferences with Effect Schema validation

**Effect Integration Patterns**:

**Schema Validation** (replaces Zod):
```typescript
import { Schema } from "effect"

// API response schema
const NotificationSchema = Schema.Struct({
  id: Schema.String,
  status: Schema.Literal("pending", "processing", "delivered", "failed"),
  channel: Schema.Literal("slack", "discord", "email", "push", "webhook"),
  priority: Schema.Literal("critical", "high", "normal", "low"),
  payload: Schema.Struct({
    title: Schema.String,
    body: Schema.String,
    metadata: Schema.optional(Schema.Record({ key: Schema.String, value: Schema.Unknown })),
  }),
  createdAt: Schema.Date,
})
type Notification = Schema.Schema.Type<typeof NotificationSchema>

// Form validation schema
const CreateIntegrationSchema = Schema.Struct({
  name: Schema.String.pipe(Schema.minLength(1), Schema.maxLength(100)),
  channel: Schema.Literal("slack", "discord", "email", "push", "webhook"),
  webhookUrl: Schema.optional(Schema.String.pipe(Schema.pattern(/^https?:\/\//))),
  enabled: Schema.Boolean,
})
```

**Data Fetching with Effect + TanStack Query**:
```typescript
import { Effect, Schema } from "effect"
import { useQuery } from "@tanstack/react-query"

// Effect-powered fetch with validation
const fetchNotifications = Effect.tryPromise({
  try: () => fetch("/api/notifications").then((r) => r.json()),
  catch: () => new ApiError({ message: "Failed to fetch notifications" }),
}).pipe(
  Effect.flatMap(Schema.decodeUnknown(Schema.Array(NotificationSchema))),
  Effect.catchTag("ParseError", (e) => Effect.fail(new ValidationError({ message: e.message })))
)

// React hook wrapping Effect
function useNotifications() {
  return useQuery({
    queryKey: ["notifications"],
    queryFn: () => Effect.runPromise(fetchNotifications),
  })
}
```

**Form Validation with Effect Schema**:
```typescript
import { useForm } from "react-hook-form"
import { effectResolver } from "@hookform/resolvers/effect-ts"

function CreateIntegrationForm() {
  const form = useForm({
    resolver: effectResolver(CreateIntegrationSchema),
    defaultValues: { name: "", channel: "slack", enabled: true },
  })
  
  // Form submission with Effect error handling
  const onSubmit = (data: CreateIntegration) =>
    Effect.runPromise(
      createIntegration(data).pipe(
        Effect.tap(() => toast.success("Integration created")),
        Effect.catchAll((e) => Effect.sync(() => toast.error(e.message)))
      )
    )
}
```

**WebSocket with Effect Stream**:
```typescript
import { Effect, Stream } from "effect"

const notificationStream = Stream.async<Notification, WebSocketError>((emit) => {
  const ws = new WebSocket("/api/ws")
  ws.onmessage = (event) => {
    const result = Schema.decodeUnknownSync(NotificationSchema)(JSON.parse(event.data))
    emit.single(result)
  }
  ws.onerror = () => emit.fail(new WebSocketError({ message: "Connection failed" }))
  return Effect.sync(() => ws.close())
})
```

**State Management**:
- TanStack Query + Effect for server state
- React Hook Form + Effect Schema for forms
- Zustand for UI state

---

### 5. Mobile App (Expo/React Native)

**Agent**: Tap  
**Priority**: Medium  
**Stack**: Expo SDK 50+, React Native, NativeWind

Receive push notifications and manage preferences on mobile.

**Screens**:
- `HomeScreen` - Recent notifications feed
- `NotificationDetailScreen` - Full notification with actions
- `IntegrationsScreen` - View connected channels
- `SettingsScreen` - Notification preferences
- `ProfileScreen` - User profile and logout

**Core Features**:
- Push notification registration (FCM/APNs)
- Biometric authentication (Face ID, fingerprint)
- Offline notification caching
- Pull-to-refresh
- Deep linking to notification details
- App badge count

**Navigation**:
- Bottom tab navigator (Home, Integrations, Settings, Profile)
- Stack navigation for detail screens

---

### 6. Desktop Client (Electron)

**Agent**: Spark  
**Priority**: Medium  
**Stack**: Electron 28+, React, TailwindCSS

System tray application for desktop notifications.

**Features**:
- System tray icon with unread count badge
- Native desktop notifications
- Quick action menu from tray
- Mini notification popup
- Keyboard shortcuts
- Auto-start on system boot
- Cross-platform (Windows, macOS, Linux)

**Windows**:
- Main window: Full notification feed
- Mini window: Quick view popup
- Settings window: Preferences and account

**Tray Menu**:
- Show/Hide main window
- Recent notifications (last 5)
- Mute for 1 hour / until tomorrow
- Preferences
- Quit

---

### 7. Deployment & Infrastructure

**Agent**: Bolt  
**Priority**: High

**Kubernetes Resources**:
- Deployments with HPA for each service
- Services (ClusterIP for internal, LoadBalancer for ingress)
- ConfigMaps for non-sensitive config
- Secrets for credentials
- PersistentVolumeClaims for stateful data
- Network policies for service isolation

**Infrastructure CRDs**:
```yaml
# PostgreSQL (CloudNative-PG)
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: alerthub-postgres
  namespace: databases
spec:
  instances: 1
  storage:
    size: 10Gi
  bootstrap:
    initdb:
      database: alerthub
      owner: alerthub_user

# Redis/Valkey
apiVersion: redis.redis.opstreelabs.in/v1beta2
kind: Redis
metadata:
  name: alerthub-valkey
  namespace: databases
spec:
  kubernetesConfig:
    image: valkey/valkey:7.2-alpine

# Kafka (Strimzi)
apiVersion: kafka.strimzi.io/v1beta2
kind: Kafka
metadata:
  name: alerthub-kafka
  namespace: kafka
spec:
  kafka:
    replicas: 1
    listeners:
      - name: plain
        port: 9092
        type: internal

# MongoDB (Percona)
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDB
metadata:
  name: alerthub-mongodb
  namespace: databases
spec:
  replsets:
    - name: rs0
      size: 1

# RabbitMQ
apiVersion: rabbitmq.com/v1beta1
kind: RabbitmqCluster
metadata:
  name: alerthub-rabbitmq
  namespace: messaging
spec:
  replicas: 1
```

**Observability**:
- Prometheus metrics from all services
- Structured JSON logging with trace IDs
- Grafana dashboards for monitoring
- Health check endpoints

---

## Technical Context

| Component | Technology | Version |
|-----------|------------|---------|
| Notification Router | Rust, Axum, tokio, sqlx | Rust 1.75+, Axum 0.7 |
| Integration Service | Bun, Elysia, Effect, Drizzle | Bun 1.1+, Elysia 1.x, Effect 3.x |
| Admin API | Go, gRPC, grpc-gateway | Go 1.22+ |
| Web Console | Next.js, React, shadcn/ui, Effect | Next.js 15, React 19, Effect 3.x |
| Mobile App | Expo, React Native, NativeWind | Expo SDK 50+ |
| Desktop Client | Electron, React | Electron 28+ |
| Database | PostgreSQL (CloudNative-PG) | PostgreSQL 15 |
| Cache | Redis/Valkey | Valkey 7.2 |
| Message Queue | Kafka (Strimzi), RabbitMQ | Kafka 3.8, RabbitMQ 3.12 |
| Document Store | MongoDB (Percona) | MongoDB 7.0 |
| Object Storage | SeaweedFS | Latest |

---

## Constraints

- API response time < 100ms p95
- Support 10,000 notifications/minute sustained throughput
- WebSocket connections: 1,000 concurrent
- Push notification delivery < 5 seconds
- 99.9% uptime SLA
- GDPR compliant (data export, deletion)

---

## Non-Goals

- SMS notifications (use third-party)
- Voice calls
- Video/voice chat
- Self-hosted deployment documentation
- Multi-region deployment (single cluster)
- Custom notification sounds on mobile

---

## Success Criteria

1. All backend services build, pass tests, and deploy successfully
2. All frontend applications build and deploy successfully
3. End-to-end notification flow works (submit → route → deliver → display)
4. Infrastructure operators provision resources correctly
5. WebSocket real-time updates function across web and desktop
6. Mobile push notifications deliver successfully
7. Admin API CRUD operations work correctly
8. Monitoring dashboards show accurate metrics

