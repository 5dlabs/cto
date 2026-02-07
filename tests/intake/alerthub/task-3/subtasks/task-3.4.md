# Subtask 3.4: Implement Notification CRUD Endpoints

## Parent Task
Task 3

## Agent
code-implementer

## Parallelizable
Yes

## Description
Build REST API endpoints for notification operations.

## Details
- Create notification routes (CRUD)
- Implement pagination for lists
- Add filters (status, priority, date range)
- Implement soft delete
- Add rate limiting per user

## Deliverables
- `src/routes/notifications.rs` - Notification routes
- `src/routes/mod.rs` - Route exports

## Acceptance Criteria
- [ ] POST /notifications creates notification
- [ ] GET /notifications lists with pagination
- [ ] GET /notifications/:id returns details
- [ ] PUT /notifications/:id updates
- [ ] DELETE /notifications/:id soft deletes
