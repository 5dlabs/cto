Implement subtask 2005: Implement GET /api/v1/notifications (list) and DELETE /api/v1/notifications/:id (cancel) endpoints

## Objective
Implement the paginated list endpoint with optional status filtering and the cancel endpoint with conflict detection for non-pending notifications.

## Steps
1. **GET /api/v1/notifications** (list):
   - Extract `Query<ListNotificationsQuery>` from URL params.
   - Compute: `page = query.page.unwrap_or(1).max(1)`, `per_page = query.per_page.unwrap_or(20).min(100)`, `offset = (page - 1) * per_page`.
   - Build dynamic query: if `status` filter is provided, add `WHERE status = $1`. Use sqlx query builder or conditional query.
   - Count total: `SELECT COUNT(*) FROM notifications [WHERE status = $1]`.
   - Fetch page: `SELECT * FROM notifications [WHERE status = $1] ORDER BY created_at DESC LIMIT $2 OFFSET $3`.
   - Return `Json(PaginatedResponse { data, page, per_page, total })`.
2. **DELETE /api/v1/notifications/:id** (cancel):
   - Extract `Path(id): Path<Uuid>`.
   - Query current notification: `SELECT * FROM notifications WHERE id = $1`.
   - If not found, return `AppError::NotFound`.
   - If status != Pending, return `AppError::Conflict("only pending notifications can be cancelled")`.
   - Update: `UPDATE notifications SET status = 'cancelled', updated_at = NOW() WHERE id = $1 RETURNING *`.
   - Return `(StatusCode::OK, Json(updated_notification))`.
3. Register routes:
   - `.route("/api/v1/notifications", get(list_notifications))` (alongside existing post)
   - `.route("/api/v1/notifications/:id", delete(cancel_notification))` (alongside existing get)
4. Consider using `.route("/api/v1/notifications", get(list).post(create))` and `.route("/api/v1/notifications/:id", get(get_one).delete(cancel))` for cleanliness.

## Validation
Integration tests: List with default pagination returns `{data, page: 1, per_page: 20, total}` structure. List with status filter returns only matching notifications. List with per_page=200 clamps to 100. Cancel a pending notification returns 200 with status=cancelled. Cancel a non-pending (e.g., already cancelled) notification returns 409. Cancel with unknown ID returns 404.