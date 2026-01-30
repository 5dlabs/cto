# Implementation Prompt for Task 10

## Context
You are implementing "End-to-End Integration Testing (Grizz - Go)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Create comprehensive integration tests that validate the complete notification flow from submission through delivery across all channels and clients.

## Implementation Details
Build Go test suite that creates test notifications, verifies routing through Rex service, confirms delivery via Nova service, validates WebSocket updates reach web clients, checks push notifications on mobile, and ensures desktop notifications display. Include load testing for throughput requirements.

## Dependencies
This task depends on: task-2, task-3, task-4, task-5, task-6, task-7. Ensure those are complete before starting.

## Testing Requirements
Complete notification flow works end-to-end, all channels deliver test notifications successfully, WebSocket updates reach all connected clients, load tests achieve 10,000 notifications/minute, API response times stay under 100ms p95, and error scenarios are handled gracefully

## Decision Points to Address

The following decisions need to be made during implementation:

### d19: Test environment strategy
**Category**: architecture | **Constraint**: open

Options:
1. dedicated test cluster
2. test namespace in main cluster
3. local development environment

Document your choice and rationale in the implementation.

### d20: Load testing approach
**Category**: performance | **Constraint**: open

Options:
1. k6 for HTTP load testing
2. custom Go load generator
3. cloud-based load testing service

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
