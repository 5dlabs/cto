# Task 11: Implement WebSocket real-time notifications

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 11.

## Goal

Setup WebSocket endpoint for live task updates and team activity

## Requirements

1. Add axum WebSocket support and tokio-tungstenite
2. Create src/websocket/ module
3. Implement connection management with Redis pub/sub
4. Add authentication for WebSocket connections
5. Broadcast task updates, assignments, and status changes
6. Handle connection cleanup and reconnection

## Acceptance Criteria

Test WebSocket connections, authentication, message broadcasting, and connection management

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-11): Implement WebSocket real-time notifications`
