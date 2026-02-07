# Subtask 32.2: Define Effect Schema models for user and rules entities

## Parent Task
Task 32

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Create Effect Schema definitions for users and rules entities with comprehensive validation and type generation

## Dependencies
None

## Implementation Details
Implement Effect Schema models for users (id, email, name, role, preferences) and rules (id, name, conditions, actions, enabled status). Include validation for email format, role enums, and rule condition syntax. Setup proper type inference and validation error handling.

## Test Strategy
Unit tests for schema validation and type inference
