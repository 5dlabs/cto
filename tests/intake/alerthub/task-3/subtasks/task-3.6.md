# Subtask 3.6: Create OpenAPI Documentation

## Parent Task
Task 3

## Agent
documenter

## Parallelizable
Yes

## Description
Generate OpenAPI/Swagger documentation for all endpoints.

## Details
- Add utoipa annotations to handlers
- Document request/response schemas
- Describe authentication requirements
- Generate openapi.yaml file
- Set up Swagger UI at /docs

## Deliverables
- `src/routes/*.rs` with annotations
- `openapi.yaml` - Generated spec
- Swagger UI static files

## Acceptance Criteria
- [ ] /docs shows Swagger UI
- [ ] All endpoints are documented
- [ ] Schemas are accurate
