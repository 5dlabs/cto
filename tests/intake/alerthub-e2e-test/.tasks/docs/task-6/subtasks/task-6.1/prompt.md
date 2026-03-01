# Subtask 6.1: Define Core Notification Struct

## Parent Task
Task 6

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Create the primary Notification struct with all required fields, serde serialization, and sqlx database mapping traits

## Dependencies
None

## Implementation Details
Implement the main Notification struct with fields like id, title, content, channel, priority, status, created_at, updated_at. Add serde Serialize/Deserialize derives and sqlx FromRow/Type traits for database operations. Include validation attributes and proper field types.

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
