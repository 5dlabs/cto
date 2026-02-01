# Subtask 32.1: Define Effect Schema models for core entities

## Parent Task
Task 32

## Subagent Type
implementer

## Agent
implementer-agent

## Parallelizable
Yes - can run concurrently

## Description
Create Effect Schema definitions for notifications and integrations entities with proper validation rules and type safety

## Dependencies
None

## Implementation Details
Implement Effect Schema models for notifications (id, type, message, timestamp, read status) and integrations (id, name, type, config, enabled status). Include validation for required fields, data types, and business rules. Setup proper imports and exports for reusability across the application.

## Test Strategy
Unit tests for schema validation with valid/invalid data cases
