# Subtask 9.4: Integrate Kafka Publisher with Notification Service

## Parent Task
Task 9

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Integrate the Kafka producer with Rex's notification service to publish events to alerthub.notifications.created topic on successful notification submission.

## Dependencies
- Subtask 9.2
- Subtask 9.3

## Implementation Details
Modify Rex's notification service handlers to integrate Kafka publisher after successful notification creation. Add event publishing to alerthub.notifications.created topic with proper message structure including notification_id, user_id, notification_type, timestamp, and payload. Implement async publishing to avoid blocking notification response. Add configuration injection and service initialization in Rex's startup sequence. Ensure proper error handling that doesn't affect notification creation success.

## Test Strategy
Integration tests with embedded Kafka and end-to-end notification flow testing

---
*Project: alerthub*
