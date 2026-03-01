# Subtask task-10.6: Implement NotificationPayload Struct

## Parent Task
Task 10

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Create NotificationPayload struct for handling dynamic notification content with flexible data structure

## Dependencies
None

## Implementation Details
Define NotificationPayload struct with fields for storing variable notification data (title, body, metadata, custom fields). Implement serde serialization for JSON storage, sqlx database mapping, and validation traits. Support both structured and unstructured payload formats.

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
