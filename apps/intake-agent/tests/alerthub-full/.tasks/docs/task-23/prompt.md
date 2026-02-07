# Task 23: Define protobuf schemas for all services

## Priority
high

## Description
Create comprehensive protobuf definitions for TenantService, UserService, RuleService, and AnalyticsService

## Dependencies
- Task 22

## Implementation Details
Define all gRPC services, messages, and enums in protobuf files. Include proper field validation, pagination, and error handling patterns.

## Acceptance Criteria
Protobuf compiles successfully, generates Go code, grpc-gateway annotations create REST endpoints, validation rules work

## Decision Points
- **d23** [api-design]: Pagination strategy for list endpoints

## Subtasks
- 1. Define TenantService and UserService protobuf schemas [implementer]
- 2. Define RuleService and AnalyticsService protobuf schemas [implementer]
- 3. Validate protobuf schema compilation and compatibility [tester]
- 4. Review protobuf schema design and best practices compliance [reviewer]
