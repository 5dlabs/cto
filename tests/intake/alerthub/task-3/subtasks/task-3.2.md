# Subtask 3.2: Implement Request Models and Validation

## Parent Task
Task 3

## Agent
code-implementer

## Parallelizable
Yes

## Description
Create request/response models with serde and validation.

## Details
- Define notification models (create, update, list)
- Create user preference models
- Implement validation with validator crate
- Add custom validation rules
- Create error response models

## Deliverables
- `src/models/request.rs` - Request types
- `src/models/response.rs` - Response types
- `src/models/validation.rs` - Custom validators

## Acceptance Criteria
- [ ] Models serialize/deserialize correctly
- [ ] Validation catches invalid input
- [ ] Error responses are well-formed
