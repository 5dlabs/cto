# Task 20: Implement Notification Router Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 20.

## Goal

Build the high-performance core service that receives, validates, and routes notifications. Handles rate limiting, priority queuing, deduplication, and publishes events to Kafka for downstream processing. Provides WebSocket connections for real-time updates and Prometheus metrics for observability.

## Code Signatures

Implement the following signatures:

```rust
use serde::{Deserialize, Serialize};
   use uuid::Uuid;
   use chrono::{DateTime, Utc};
   use std::collections::HashMap;
   use validator::Validate;

   #[derive(Debug, Serialize, Deserialize, Clone)]
   #[serde(rename_all = "lowercase")]
   pub enum Channel {
       Slack,
       Discord,
       Email,
       Push,
       Webhook,
   }

   #[derive(Debug, Serialize, Deserialize, Clone)]
   #[serde(rename_all = "lowercase")]
   pub enum Priority {
       Critical,
       High,
       Normal,
       Low,
   }

   #[derive(Debug, Serialize, Deserialize, Clone)]
   pub struct NotificationPayload {
       pub title: String,
       pub body: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub metadata: Option<HashMap<String, serde_json::Value>>,
   }

   #[derive(Debug, Serialize, Deserialize, Validate)]
   pub struct CreateNotificationRequest {
       #[validate(length(min = 1))]
       pub tenant_id: String,
       pub channel: Channel,
       pub priority: Priority,
       #[validate]
       pub payload: NotificationPayload,
   }

   #[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
   pub struct Notification {
       pub id: Uuid,
       pub tenant_id: Uuid,
       pub channel: String,
       pub priority: String,
       pub payload: serde_json::Value,
       pub status: String,
       pub created_at: DateTime<Utc>,
   }
```

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};
   use uuid::Uuid;
   use crate::models::Notification;

   pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
       PgPoolOptions::new()
           .max_connections(50)
           .connect(database_url)
           .await
   }

   pub async fn insert_notification(
       pool: &PgPool,
       notification: &Notification,
   ) -> Result<Uuid, sqlx::Error> {
       let rec = sqlx::query!(
           r#"
           INSERT INTO notifications (id, tenant_id, channel, priority, payload, status, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id
           "#,
           notification.id,
           notification.tenant_id,
           notification.channel,
           notification.priority,
           notification.payload,
           notification.status,
           notification.created_at
       )
       .fetch_one(pool)
       .await?;
       Ok(rec.id)
   }
```

```rust
use redis::{AsyncCommands, aio::ConnectionManager};
   use std::time::Duration;

   pub struct RateLimiter {
       client: ConnectionManager,
   }

   impl RateLimiter {
       pub fn new(client: ConnectionManager) -> Self {
           Self { client }
       }

       pub async fn check_limit(
           &mut self,
           tenant_id: &str,
           max_requests: usize,
           window: Duration,
       ) -> Result<bool, redis::RedisError> {
           let key = format!("rate_limit:{}:{}", tenant_id, chrono::Utc::now().timestamp() / window.as_secs() as i64);
           let count: usize = self.client.incr(&key, 1).await?;
           self.client.expire(&key, window.as_secs() as usize).await?;
           Ok(count <= max_requests)
       }
   }
```

```rust
use rdkafka::producer::{FutureProducer, FutureRecord};
   use rdkafka::ClientConfig;
   use std::time::Duration;

   pub fn create_producer(brokers: &str) -> FutureProducer {
       ClientConfig::new()
           .set("bootstrap.servers", brokers)
           .set("message.timeout.ms", "5000")
           .create()
           .expect("Failed to create Kafka producer")
   }

   pub async fn publish_event(
       producer: &FutureProducer,
       topic: &str,
       key: &str,
       payload: &str,
   ) -> Result<(), rdkafka::error::KafkaError> {
       let record = FutureRecord::to(topic)
           .key(key)
           .payload(payload);
       producer.send(record, Duration::from_secs(0)).await.map(|_| ()).map_err(|(e, _)| e)
   }
```

```rust
use axum::{
       extract::{Path, State, WebSocketUpgrade},
       http::StatusCode,
       response::IntoResponse,
       routing::{get, post},
       Json, Router,
   };
   use uuid::Uuid;
   use crate::models::CreateNotificationRequest;
   use crate::AppState;

   pub fn create_router(state: AppState) -> Router {
       Router::new()
           .route("/api/v1/notifications", post(create_notification))
           .route("/api/v1/notifications/:id", get(get_notification))
           .route("/api/v1/ws", get(websocket_handler))
           .route("/health/live", get(health_live))
           .route("/health/ready", get(health_ready))
           .route("/metrics", get(metrics))
           .with_state(state)
   }

   async fn create_notification(
       State(state): State<AppState>,
       Json(payload): Json<CreateNotificationRequest>,
   ) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
       // Rate limit check, dedup check, insert to DB, publish to Kafka
       Ok((StatusCode::ACCEPTED, Json(serde_json::json!({"id": Uuid::new_v4()}))))
   }
```

## Requirements

1. Initialize Rust project with Cargo.toml dependencies:
   - axum = "0.7"
   - tokio = { version = "1.35", features = ["full"] }
   - sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono"] }
   - redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
   - rdkafka = { version = "0.36", features = ["cmake-build"] }
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - uuid = { version = "1.6", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
   - validator = { version = "0.18", features = ["derive"] }
   - tracing = "0.1"
   - tracing-subscriber = "0.3"
   - prometheus = "0.13"
   - tower-http = { version = "0.5", features = ["cors", "trace"] }

2. Define data models in src/models.rs:

3. Implement database layer in src/db.rs:

4. Implement Redis rate limiter in src/rate_limit.rs:

5. Implement Kafka producer in src/kafka.rs:

6. Implement Axum routes in src/routes.rs:

7. Implement WebSocket handler for real-time updates

8. Add Prometheus metrics collection

9. Create main.rs with server initialization

10. Write Dockerfile with multi-stage build

11. Create Kubernetes Deployment and Service manifests

## Acceptance Criteria

1. Unit tests for models, validation, and business logic:
   - Test Channel and Priority enum serialization
   - Test NotificationPayload validation
   - Test rate limiter logic with mock Redis

2. Integration tests with test containers:
   - Spin up PostgreSQL, Redis, Kafka containers
   - Test POST /api/v1/notifications returns 202 with valid ID
   - Test rate limiting blocks after threshold
   - Test deduplication prevents duplicate notifications
   - Test Kafka event publishing
   - Test WebSocket connection and message delivery

3. Load testing with k6:
   - Sustained 10,000 notifications/minute
   - Verify p95 latency < 100ms
   - Test concurrent WebSocket connections (1,000+)

4. End-to-end tests:
   - Submit notification via API
   - Verify record in PostgreSQL
   - Verify event in Kafka topic
   - Verify WebSocket clients receive update

5. Health check validation:
   - GET /health/live returns 200
   - GET /health/ready returns 200 when all dependencies available
   - GET /metrics returns Prometheus format

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-20): Implement Notification Router Service (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 19
