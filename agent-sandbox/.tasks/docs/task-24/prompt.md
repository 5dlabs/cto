# Task 24: Implement WebSocket endpoint for real-time task updates

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 24.

## Goal

Create WebSocket server for live task notifications using Redis pub/sub as message broker

## Requirements

1. Add dependencies: axum-extra = { version = "0.9", features = ["ws"] }, futures = "0.3"
2. Create api/websocket.rs with:
   - GET /api/ws -> upgrade to WebSocket, authenticate via query param ?token=xxx
   - struct WsConnection { user_id, team_ids: Vec<Uuid>, sender: mpsc::Sender }
   - On connection: subscribe to Redis channels team:{team_id}:tasks for user's teams
3. Create infra/pubsub.rs:
   - async fn publish_task_event(redis: &ConnectionManager, team_id: Uuid, event: TaskEvent)
   - async fn subscribe_to_team(redis: &ConnectionManager, team_id: Uuid) -> Receiver<TaskEvent>
4. Message format: { type: "task.created"|"task.updated"|"task.deleted", task: Task }
5. Integrate with task API: publish events on create/update/delete
6. Handle connection limits: max 1000 concurrent connections, reject new connections if exceeded
7. Implement ping/pong for connection health

## Acceptance Criteria

WebSocket client tests, verify events published on task operations, test connection limit enforcement, verify user only receives events for their teams, load test with 1000 connections

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-24): Implement WebSocket endpoint for real-time task updates`
