# Subtask 11.11.4: Integrate metrics middleware with Axum routes

## Parent Task
Task 11

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Add metrics collection middleware to all Axum route handlers

## Dependencies
- Subtask 11.11.3

## Implementation Details
Implement Tower middleware layer that records request duration, status codes, and endpoint labels for all routes automatically

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
