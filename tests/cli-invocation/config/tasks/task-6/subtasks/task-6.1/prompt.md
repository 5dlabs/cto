# Subtask 6.1: Configure sqlx database connection pool and environment setup

## Parent Task
Task 6

## Subagent Type
implementer

## Agent
infra-deployer

## Parallelizable
Yes - can run concurrently

## Description
Set up PostgreSQL connection pool using sqlx with proper configuration from environment variables including database URL, pool settings, and connection timeouts.

## Dependencies
None

## Implementation Details
Create database configuration module that reads DATABASE_URL and other DB settings from environment. Initialize sqlx PgPool with appropriate connection limits, timeouts, and SSL settings. Add necessary dependencies to Cargo.toml including sqlx with postgres and runtime features. Implement connection health checks and graceful shutdown handling.

## Test Strategy
Unit tests for configuration parsing and integration tests for connection establishment
