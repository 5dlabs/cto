# Subtask task-10.1: Add performance indexes and seed initial data

## Parent Task
Task 10

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Create composite indexes for common queries and insert seed data for system roles and default tenant

## Dependencies
- Subtask 2.2.3

## Implementation Details
Add compound indexes on (tenant_id, status), (tenant_id, created_at). Insert default system roles and admin tenant seed data

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
