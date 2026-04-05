Implement subtask 3004: Implement OpportunityService with lead scoring and conflict detection

## Objective
Build the OpportunityService gRPC handler with full CRUD, lead scoring algorithm (GREEN/YELLOW/RED), date/equipment conflict detection, and ConvertToProject RPC.

## Steps
1. Implement the OpportunityService gRPC server in /internal/opportunity/. 2. Wire up CreateOpportunity, ListOpportunities, UpdateOpportunity RPCs to the OpportunityRepo. 3. Implement ScoreOpportunity RPC: calculate lead score based on factors like estimated value, equipment availability, date conflicts, customer history. Return GREEN (high confidence), YELLOW (needs review), RED (conflicts/issues). 4. Implement conflict detection: check for overlapping date ranges on the same equipment across existing projects and opportunities. 5. Implement ConvertToProject RPC: transition an approved opportunity into a new project record, copying relevant fields and updating opportunity status to 'converted'. 6. Register the service with the gRPC server and ensure grpc-gateway routes are connected.

## Validation
Unit tests for lead scoring return correct GREEN/YELLOW/RED for known scenarios; conflict detection identifies overlapping equipment bookings; ConvertToProject creates a valid project and marks opportunity as converted; gRPC and REST endpoints return expected responses.