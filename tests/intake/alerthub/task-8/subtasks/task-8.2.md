# Subtask 8.2: Implement Event Tracking

## Parent Task
Task 8

## Agent
code-implementer

## Parallelizable
Yes

## Description
Build event tracking SDK and ingestion API.

## Details
- Create event schema (user_id, event_type, properties, timestamp)
- Implement client SDK for event tracking
- Build API endpoint for event ingestion
- Add event validation
- Implement sampling for high-volume events

## Deliverables
- `src/api/events.ts` - Ingestion API
- `packages/tracking-sdk/` - Client SDK

## Acceptance Criteria
- [ ] Events ingested successfully
- [ ] Validation works
