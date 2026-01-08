# Task 17: Implement Notification Router Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 17.

## Goal

Build the high-performance core notification routing service in Rust using Axum. Handles notification submission, validation, rate limiting, priority queuing, and event streaming to Kafka. Includes WebSocket support for real-time updates and Prometheus metrics.

## Requirements

1. Initialize Rust project with Cargo:

```bash
cargo new notification-router --bin
```

Add dependencies: axum = "0.7", tokio = { version = "1.35", features = ["full"] }, sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono"] }, redis = { version = "0.24", features = ["tokio-comp"] }, rdkafka = "0.36", serde = { version = "1.0", features = ["derive"] }, uuid = { version = "1.6", features = ["v4", "serde"] }, chrono = { version = "0.4", features = ["serde"] }, tower = "0.4", tower-http = { version = "0.5", features = ["cors", "trace"] }, tracing = "0.1", tracing-subscriber = "0.3", prometheus = "0.13"

2. Define data models in src/models.rs:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "lowercase")]
pub enum NotificationStatus { Pending, Processing, Delivered, Failed }

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "channel", rename_all = "lowercase")]
pub enum Channel { Slack, Discord, Email, Push, Webhook }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub channel: Channel,
    pub priority: Priority,
    pub payload: NotificationPayload,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub status: NotificationStatus,
}
```

3. Implement database layer with sqlx:

- Create migrations in migrations/ directory
- CREATE TYPE notification_status AS ENUM ('pending', 'processing', 'delivered', 'failed');
- CREATE TABLE notifications (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, channel VARCHAR(20), priority VARCHAR(20), payload JSONB, metadata JSONB, created_at TIMESTAMPTZ DEFAULT NOW(), status notification_status, INDEX idx_tenant_status (tenant_id, status), INDEX idx_created_at (created_at DESC));
- Implement NotificationRepository with insert, get_by_id, update_status methods

4. Implement Redis rate limiter:

```rust
pub struct RateLimiter { client: redis::Client }
async fn check_rate_limit(&self, tenant_id: Uuid) -> Result<bool, Error> {
    let key = format!("ratelimit:{}:minute", tenant_id);
    let count: i64 = self.client.incr(&key, 1).await?;
    if count == 1 { self.client.expire(&key, 60).await?; }
    Ok(count <= 1000) // 1000 req/min per tenant
}
```

5. Implement deduplication cache:

```rust
async fn check_duplicate(&self, notification_hash: String) -> Result<bool, Error> {
    let key = format!("dedup:{}", notification_hash);
    let exists: bool = self.client.exists(&key).await?;
    if !exists { self.client.set_ex(&key, "1", 300).await?; } // 5 min TTL
    Ok(exists)
}
```

6. Implement Kafka producer:

```rust
pub struct EventProducer { producer: FutureProducer }
async fn publish_notification_event(&self, notification: &Notification) -> Result<(), Error> {
    let payload = serde_json::to_string(notification)?;
    let record = FutureRecord::to("notifications-events").payload(&payload).key(&notification.id.to_string());
    self.producer.send(record, Duration::from_secs(0)).await?;
    Ok(())
}
```

7. Implement Axum routes:

POST /api/v1/notifications:

```rust
async fn create_notification(State(app): State<AppState>, Json(req): Json<CreateNotificationRequest>) -> Result<Json<Notification>, StatusCode> {
    // 1. Validate tenant exists
    // 2. Check rate limit
    // 3. Generate notification ID and hash
    // 4. Check deduplication
    // 5. Insert to PostgreSQL
    // 6. Publish to Kafka
    // 7. Return notification
}
```

POST /api/v1/notifications/batch:

```rust
async fn create_batch(State(app): State<AppState>, Json(reqs): Json<Vec<CreateNotificationRequest>>) -> Result<Json<Vec<Notification>>, StatusCode> {
    if reqs.len() > 100 { return Err(StatusCode::BAD_REQUEST); }
    // Process in parallel with tokio::spawn tasks
}
```

GET /api/v1/notifications/:id:

```rust
async fn get_notification(State(app): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<Notification>, StatusCode>
```

8. Implement WebSocket handler:

```rust
async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app))
}
async fn handle_socket(socket: WebSocket, app: AppState) {
    // Subscribe to Redis pub/sub for tenant notifications
    // Stream updates to WebSocket client
}
```

9. Add Prometheus metrics:

```rust
lazy_static! {
    static ref NOTIFICATIONS_TOTAL: IntCounterVec = register_int_counter_vec!("notifications_total", "Total notifications", &["tenant_id", "channel", "status"]).unwrap();
    static ref NOTIFICATION_DURATION: HistogramVec = register_histogram_vec!("notification_duration_seconds", "Notification processing duration", &["endpoint"]).unwrap();
}
```

GET /metrics:

```rust
async fn metrics() -> String { prometheus::TextEncoder::new().encode_to_string(&prometheus::gather()).unwrap() }
```

10. Add health checks:

GET /health/live:

```rust
async fn liveness() -> StatusCode { StatusCode::OK }
```

GET /health/ready:

```rust
async fn readiness(State(app): State<AppState>) -> StatusCode {
    // Check PostgreSQL, Redis, Kafka connectivity
}
```

11. Main application setup:

```rust
#[tokio::main]
async fn main() {
    let pg_pool = PgPoolOptions::new().connect(&env::var("DATABASE_URL")?).await?;
    let redis_client = redis::Client::open(env::var("REDIS_URL")?)?;
    let kafka_producer = ClientConfig::new().set("bootstrap.servers", &env::var("KAFKA_BROKERS")?).create()?;

    let app = Router::new()
        .route("/api/v1/notifications", post(create_notification))
        .route("/api/v1/notifications/batch", post(create_batch))
        .route("/api/v1/notifications/:id", get(get_notification))
        .route("/api/v1/ws", get(ws_handler))
        .route("/metrics", get(metrics))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { pg_pool, redis_client, kafka_producer });

    axum::Server::bind(&"0.0.0.0:8080".parse()?).serve(app.into_make_service()).await?;
}
```

12. Create Dockerfile:

```dockerfile
FROM rust:1.75-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/notification-router /usr/local/bin/
EXPOSE 8080
CMD ["notification-router"]
```

13. Create Kubernetes manifests:

- Deployment with 3 replicas, resource limits (500m CPU, 512Mi memory)
- HorizontalPodAutoscaler targeting 70% CPU
- Service (ClusterIP) exposing port 8080
- ConfigMap with environment variables from task 16
- Secret with database credentials

## Acceptance Criteria

1. Unit tests for models, rate limiter, deduplication:

```rust
#[tokio::test]
async fn test_rate_limiter_allows_within_limit() {
    let limiter = RateLimiter::new();
    for _ in 0..1000 {
        assert!(limiter.check_rate_limit(Uuid::new_v4()).await.unwrap());
    }
}
```

2. Integration tests for API endpoints:

```rust
#[tokio::test]
async fn test_create_notification_success() {
    let app = create_test_app().await;
    let response = app.oneshot(Request::builder().method("POST").uri("/api/v1/notifications").body(Body::from(json!({"tenant_id": "...", "channel": "slack", ...}).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

3. Test batch endpoint with 100 notifications
4. Test rate limiting by exceeding tenant limit
5. Test deduplication by submitting same notification twice
6. Test WebSocket connection and message streaming
7. Load test with k6: 10,000 req/min sustained for 5 minutes
8. Verify Kafka messages are published correctly
9. Check Prometheus metrics endpoint returns valid data
10. Test health checks return correct status
11. Deploy to Kubernetes and verify pod startup, readiness probes
12. Test cross-service communication with Integration Service consuming Kafka events

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-17): Implement Notification Router Service (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 16
