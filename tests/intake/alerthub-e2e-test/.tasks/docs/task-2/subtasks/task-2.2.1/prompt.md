# Subtask 2.2.1: Create users and tenants tables with indexes

## Parent Task
Task 2

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Design and create users and tenants PostgreSQL tables with proper constraints and indexes

## Dependencies
None

## Implementation Details
Create users table (id, tenant_id, email, role, preferences, created_at), tenants table (id, name, plan, settings, created_at) with indexes

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
