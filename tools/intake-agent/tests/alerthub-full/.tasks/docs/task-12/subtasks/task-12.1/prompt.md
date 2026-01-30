# Subtask 12.1: Set up Prometheus crate integration and basic metrics endpoint

## Parent Task
Task 12

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Add prometheus crate dependency and implement the basic /metrics endpoint handler in Axum with initial registry setup

## Dependencies
None

## Implementation Details
Add prometheus and prometheus-hyper crates to Cargo.toml. Create metrics module with PrometheusHandle struct. Implement /metrics GET endpoint handler that returns metrics in Prometheus format. Set up basic metric registry and ensure endpoint responds with proper content-type headers.

## Test Strategy
Unit tests for endpoint response format and basic registry functionality
