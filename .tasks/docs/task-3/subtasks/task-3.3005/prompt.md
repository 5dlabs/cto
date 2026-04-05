Implement subtask 3005: Implement ProjectService with quote-to-project workflow

## Objective
Build the ProjectService gRPC handler supporting project lifecycle, quote generation, quote approval, and status transitions.

## Steps
1. Implement ProjectService gRPC server in /internal/project/. 2. Wire up CreateProject, GetProject, ListProjects, UpdateProjectStatus RPCs to ProjectRepo. 3. Implement GenerateQuote RPC: calculate quote based on equipment list, rental duration, crew costs, delivery fees. Store quote with line items in the quotes table. 4. Implement ApproveQuote RPC: transition quote status to approved, update project status to 'confirmed', trigger any downstream effects (crew reservation, equipment hold). 5. Implement project status state machine: draft → quoted → confirmed → in_progress → completed → archived. Validate transitions. 6. Register service and verify grpc-gateway REST routes.

## Validation
Quote generation produces correct line items and totals for sample equipment lists; quote approval transitions project to confirmed status; invalid status transitions are rejected with appropriate errors; full quote-to-project workflow completes end-to-end.