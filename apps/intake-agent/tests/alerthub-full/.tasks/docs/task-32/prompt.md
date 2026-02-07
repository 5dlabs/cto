# Task 32: Setup Effect Schema for API validation

## Priority
high

## Description
Create Effect Schema definitions for all API requests and responses with validation

## Dependencies
- Task 31

## Implementation Details
Define Effect Schema models for notifications, integrations, users, and rules. Setup validation and type generation for API interactions.

## Acceptance Criteria
Schemas validate API responses correctly, type inference works in components, validation errors are descriptive and handled properly

## Decision Points
- **d32** [error-handling]: Client-side validation failure handling

## Subtasks
- 1. Define Effect Schema models for core entities [implementer]
- 2. Define Effect Schema models for user and rules entities [implementer]
- 3. Setup API validation infrastructure and type generation [implementer]
- 4. Review Effect Schema implementation for best practices [reviewer]
