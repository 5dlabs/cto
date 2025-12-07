# Task 10: Setup WebSocket infrastructure for real-time updates

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 10.

## Goal

Implement WebSocket endpoint for live task updates and team activity

## Requirements

1. Add axum WebSocket support with tokio-tungstenite
2. Create websocket.rs module with connection management
3. Implement /ws endpoint with authentication via query parameter JWT
4. Create WebSocket connection pool grouped by team_id
5. Add Redis pub/sub for broadcasting updates across server instances
6. Handle WebSocket connection lifecycle (connect, disconnect, heartbeat)
7. Support 1000 concurrent connections with connection limits

## Acceptance Criteria

Load tests for concurrent WebSocket connections and integration tests for message broadcasting

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-10): Setup WebSocket infrastructure for real-time updates`
