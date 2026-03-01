# Subtask task-14.1: Implement WebSocket Connection Management Core

## Parent Task
Task 14

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Create the foundational WebSocket connection handling infrastructure including connection lifecycle, heartbeat/ping mechanisms, and basic connection state management.

## Dependencies
None

## Implementation Details
Implement WebSocket upgrade handler in Axum, connection struct with metadata (user_id, tenant_id, connection_id), heartbeat timer management, ping/pong frame handling, graceful connection cleanup on disconnect, and connection state tracking. Include error handling for connection failures and timeout scenarios.

## Test Strategy
Unit tests for connection lifecycle, heartbeat functionality, and error scenarios

---
*Project: alerthub*
