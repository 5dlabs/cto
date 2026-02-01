# Subtask 21.2: Implement Update and Delete integration endpoints

## Parent Task
Task 21

## Subagent Type
implementer

## Agent
code-implementer

## Parallelizable
Yes - can run concurrently

## Description
Create PUT/PATCH /integrations/:id endpoint for updating integrations and DELETE /integrations/:id endpoint for deleting integrations with proper tenant isolation

## Dependencies
None

## Implementation Details
Implement Elysia routes for updating existing integrations (PUT/PATCH /integrations/:id) and deleting integrations (DELETE /integrations/:id). Include Effect validation schemas for update payloads, tenant-based authorization checks, proper error handling for validation failures, not found cases, and unauthorized access. Ensure tenant isolation by verifying integration ownership before update/delete operations.

## Test Strategy
See parent task acceptance criteria.
