# Task 12: Set up WebSocket endpoint for real-time updates

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 12.

## Goal

Implement WebSocket connection handling for live task updates and team notifications

## Requirements

1. Add axum WebSocket support and connection upgrade handling
2. Create WebSocket connection manager to track user sessions by team
3. Implement authentication for WebSocket connections using JWT
4. Create message broadcasting system for task updates within teams
5. Handle connection lifecycle (connect, disconnect, heartbeat)
6. Add connection limit enforcement (1000 concurrent)

## Acceptance Criteria

Test WebSocket connection establishment, verify authentication, validate message broadcasting and connection limits

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-12): Set up WebSocket endpoint for real-time updates`
