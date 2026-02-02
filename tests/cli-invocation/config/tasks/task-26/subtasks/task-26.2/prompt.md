# Subtask 26.2: Implement role-based authorization and Redis session management

## Parent Task
Task 26

## Subagent Type
implementer

## Agent
redis-deployer

## Parallelizable
Yes - can run concurrently

## Description
Create authorization middleware for role/permission checking and Redis integration for token refresh and session storage

## Dependencies
None

## Implementation Details
Implement role-based access control middleware that checks user permissions against endpoint requirements, Redis client setup for session storage, token refresh logic with Redis persistence, and session invalidation mechanisms. Include Redis connection pooling and error handling.

## Test Strategy
Integration tests with Redis mock and authorization scenario tests
