# Subtask 29.1: Implement core analytics data models and gRPC service interface

## Parent Task
Task 29

## Subagent Type
implementer

## Agent
analytics-implementer

## Parallelizable
Yes - can run concurrently

## Description
Create the foundational analytics service with gRPC proto definitions, data models for metrics storage, and basic service structure

## Dependencies
None

## Implementation Details
Define analytics.proto with service methods for metric ingestion and retrieval. Create Go structs for NotificationMetric, DeliveryMetric, and AggregatedStats. Implement basic AnalyticsService struct with stub methods for data ingestion, time-series calculations, and metric retrieval. Set up database schema for metrics storage with appropriate indexes for time-based queries.

## Test Strategy
See parent task acceptance criteria.
