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
│  │   Rust/Axum    │  │  Node/Fastify  │  │    Go/gRPC     │        │
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

### 2. Integration Service (Node.js/Fastify)

**Agent**: Nova  
**Priority**: High  
**Language**: Node.js 20+  
**Framework**: Fastify 4.x with TypeScript

Handles delivery to external channels (Slack, Discord, email, webhooks).

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
- Retry logic with exponential backoff
- Template rendering (Handlebars)
- Rate limiting per channel
- OAuth2 token refresh for Slack/Discord
- Webhook signature verification

**Data Models**:
```typescript
interface Integration {
  id: string;
  tenantId: string;
  channel: 'slack' | 'discord' | 'email' | 'push' | 'webhook';
  name: string;
  config: ChannelConfig;
  enabled: boolean;
  createdAt: Date;
  updatedAt: Date;
}

interface ChannelConfig {
  // Slack
  webhookUrl?: string;
  botToken?: string;
  channel?: string;
  
  // Email
  smtpHost?: string;
  smtpPort?: number;
  fromAddress?: string;
  
  // Webhook
  url?: string;
  headers?: Record<string, string>;
  secret?: string;
}
```

**Infrastructure Dependencies**:
- MongoDB: Integration configs, templates (flexible schema)
- RabbitMQ: Task queue for delivery jobs
- Kafka: Consume notification events

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

### 4. Web Console (React/Next.js)

**Agent**: Blaze  
**Priority**: High  
**Stack**: Next.js 14+ App Router, React 18, shadcn/ui, TailwindCSS

The primary configuration interface for AlertHub.

**Pages**:
- `/` - Dashboard with notification overview
- `/notifications` - Notification history with filters
- `/integrations` - Manage channel integrations
- `/rules` - Configure notification rules
- `/settings` - Tenant and user settings
- `/analytics` - Delivery metrics and charts

**Core Features**:
- Dark/light theme support
- Real-time notification feed (WebSocket)
- Drag-and-drop rule builder
- Integration wizard with OAuth flows
- Responsive design (mobile-friendly)
- Toast notifications for actions

**Key Components**:
- `<NotificationFeed />` - Real-time notification list
- `<IntegrationCard />` - Integration status and actions
- `<RuleBuilder />` - Visual rule configuration
- `<AnalyticsChart />` - Delivery metrics visualization
- `<SettingsForm />` - User/tenant preferences

**State Management**:
- TanStack Query for server state
- React Hook Form for forms
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
| Integration Service | Node.js, Fastify, Prisma | Node 20+, Fastify 4 |
| Admin API | Go, gRPC, grpc-gateway | Go 1.22+ |
| Web Console | Next.js, React, shadcn/ui | Next.js 14+ |
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

