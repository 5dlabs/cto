Implement subtask 2002: Data models, enum definitions, and database migration

## Objective
Define all data models (Notification, Channel, Priority, NotificationStatus, CreateNotificationRequest, ListNotificationsQuery) with serde and sqlx derivations, and create the database migration SQL.

## Steps
1. Create `src/models.rs`:
   - `Channel` enum: `email`, `sms`, `push`, `in_app` — derive Serialize, Deserialize with `#[serde(rename_all = "snake_case")]`. Map to/from VARCHAR for sqlx (impl sqlx::Type or use string mapping).
   - `Priority` enum: `low`, `normal`, `high`, `urgent` — same derivations.
   - `NotificationStatus` enum: `pending`, `sent`, `failed`, `cancelled` — same derivations.
   - `Notification` struct: `id: Uuid`, `channel: Channel`, `priority: Priority`, `title: String`, `body: String`, `status: NotificationStatus`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`. Derive Serialize, Deserialize, sqlx::FromRow.
   - `CreateNotificationRequest` struct: `channel: Channel`, `priority: Priority`, `title: String`, `body: String`. Derive Deserialize.
   - `ListNotificationsQuery` struct: `page: Option<u32>`, `per_page: Option<u32>`, `status: Option<NotificationStatus>`. Derive Deserialize.
   - `PaginatedResponse<T>` struct: `data: Vec<T>`, `page: u32`, `per_page: u32`, `total: i64`.
2. Create `migrations/001_create_notifications.sql`:
   - CREATE TABLE notifications with all columns as specified.
   - CREATE INDEX idx_notifications_status_created ON notifications (status, created_at DESC).
3. Ensure sqlx migration runs on startup (already wired in AppState from 2001).

## Validation
`cargo build` compiles with all model types. Enum serialization unit tests verify lowercase JSON output (e.g., `Channel::Email` serializes to `"email"`). Migration SQL is syntactically valid (verified via sqlx migrate check or manual review).