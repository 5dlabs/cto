# Task 24: Implement WebSocket endpoint for real-time task updates

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 24.

## Goal

Create WebSocket connection handler that broadcasts task changes to team members using Redis pub/sub for horizontal scaling.

## Requirements

1. Add dependency: axum-extra = { version = "0.9", features = ["typed-header"] }
2. Create domain/events.rs:
   - TaskEvent enum (Created, Updated, Deleted, StatusChanged)
   - Serialize to JSON for Redis pub/sub
3. Implement infra/pubsub.rs:
   - subscribe_to_team(team_id) -> redis::PubSub
   - publish_task_event(team_id, event: TaskEvent) -> Result<()>
   - Channel format: team:{team_id}:tasks
4. Create api/ws.rs:
   - GET /api/teams/:team_id/ws (upgrade to WebSocket)
   - Validate JWT from query param or Sec-WebSocket-Protocol header
   - Verify user is team member
   - Subscribe to Redis channel for team
   - Forward events to WebSocket client
   - Handle ping/pong for connection health
5. Modify task handlers to publish events after mutations
6. Implement connection manager to track active connections per team
7. Add graceful shutdown handling

## Acceptance Criteria

Integration tests: connect WebSocket as team member, create task via REST API, verify WebSocket receives event. Test unauthorized connection rejected. Test connection survives network interruptions with ping/pong. Load test 1000 concurrent connections. Verify events only sent to team members.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-24): Implement WebSocket endpoint for real-time task updates`
