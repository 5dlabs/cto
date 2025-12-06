# Task 9: Build WebSocket endpoint for real-time task updates

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 9.

## Goal

Implement WebSocket connection handling with Redis pub/sub for broadcasting task changes to connected team members

## Requirements

1. Add dependencies: tokio-tungstenite = "0.21", futures-util = "0.3"
2. Create src/api/websocket.rs:
   - GET /api/ws: upgrade_to_websocket(Query<{token: String}>) -> WebSocketUpgrade
   - Validate JWT from query param
   - Store connection in HashMap<UserId, Vec<WebSocket>>
3. Implement Redis pub/sub listener in src/infra/notifications.rs:
   - Subscribe to channel: task_updates:{team_id}
   - On message, broadcast to all connected team members
4. Modify task update handlers to publish events:
   - redis.publish("task_updates:{team_id}", json!({"type": "task_updated", "task": task}))
5. Message format: {"type": "task_created|task_updated|task_deleted", "task": TaskDto, "timestamp": ISO8601}
6. Handle connection cleanup on disconnect

## Acceptance Criteria

Integration test: connect 2 clients to same team, update task via REST API, verify both receive WebSocket message. Test 1000 concurrent connections. Test reconnection handling. Verify messages only sent to team members

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-9): Build WebSocket endpoint for real-time task updates`
