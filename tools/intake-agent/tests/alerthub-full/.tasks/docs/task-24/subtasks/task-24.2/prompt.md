# Subtask 24.2: Create Go domain models and GORM structs

## Parent Task
Task 24

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Define Go structs for tenants, users, and rules entities with proper GORM tags and relationships

## Dependencies
None

## Implementation Details
Create tenant model with fields like ID, name, domain, created_at, updated_at. Create user model with ID, email, password_hash, tenant_id (foreign key), role, status fields. Create rules model with ID, name, description, tenant_id (foreign key), rule_type, configuration JSON field, enabled status. Add proper GORM tags for database mapping, indexes, and foreign key constraints.

## Test Strategy
Unit tests for model validation and GORM tag correctness
