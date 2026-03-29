Implement subtask 2004: Implement POST and GET /api/v1/notifications/:id endpoints

## Objective
Implement the POST /api/v1/notifications endpoint for creating notifications with validation, and the GET /api/v1/notifications/:id endpoint for retrieving a single notification by UUID.

## Steps
1. Create `src/handlers.rs` (or `src/handlers/notifications.rs`):
2. **POST /api/v1/notifications**:
   - Extract `Json<CreateNotificationRequest>` from request body.
   - Validate: title must be non-empty, body must be non-empty. Return `AppError::Validation` if invalid.
   - Generate `Uuid::new_v4()` for the notification ID.
   - Insert into database: `INSERT INTO notifications (id, channel, priority, title, body, status) VALUES ($1, $2, $3, $4, $5, 'pending') RETURNING *`.
   - Return `(StatusCode::CREATED, Json(notification))`.
3. **GET /api/v1/notifications/:id**:
   - Extract `Path(id): Path<Uuid>` from URL.
   - Query: `SELECT * FROM notifications WHERE id = $1`.
   - If found, return `(StatusCode::OK, Json(notification))`.
   - If not found, return `AppError::NotFound("not found".to_string())`.
4. Register both routes on the Axum Router in main.rs:
   - `.route("/api/v1/notifications", post(create_notification))`
   - `.route("/api/v1/notifications/:id", get(get_notification))`
5. Ensure AppState is available via Axum's `State` extractor.

## Validation
Integration tests: POST with valid payload returns 201 with UUID and status=pending. POST with empty title returns 422. POST with empty body returns 422. GET with valid ID returns 200 with matching notification. GET with random UUID returns 404 with `{"error": "not found"}`.