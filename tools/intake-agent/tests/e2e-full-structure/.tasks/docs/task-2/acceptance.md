# Acceptance Criteria - Task 2

## Task
Build Alert Management Backend Service (Grizz - Go/gRPC)

## Criteria
1. Webhook endpoints accept valid alerts and reject invalid ones
2. alerts are routed to correct channels based on rules
3. escalation policies trigger after configured timeouts
4. on-call schedules return correct personnel
5. alert state transitions work correctly
6. and all CRUD operations persist to database

## Decision Points Requiring Resolution
### Requires Human Approval
- **d3**: Alert ingestion format - custom schema vs industry standard like PagerDuty (api-design)

## Definition of Done
- [ ] All acceptance criteria met
- [ ] Tests passing
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] Decision "d3" approved: Alert ingestion format - custom schema vs industry standard like PagerDuty
