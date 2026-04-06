Implement subtask 5010: Implement GDPR deletion endpoint

## Objective
Build the DELETE /api/v1/gdpr/customer/:id endpoint that removes all vetting data (vetting_results and vetting_requests) for a given org_id and returns confirmation.

## Steps
1. Add route `DELETE /api/v1/gdpr/customer/:id` to the Axum router.
2. Handler accepts org_id as path parameter (UUID).
3. Execute within a database transaction:
   - DELETE FROM vetting_requests WHERE org_id = $1
   - DELETE FROM vetting_results WHERE org_id = $1
4. Return JSON response: { "deleted": true, "org_id": "...", "records_removed": { "vetting_results": N, "vetting_requests": M } }.
5. If no records found, still return 200 with records_removed counts at 0.
6. Log the deletion event with org_id for compliance audit trail (but do NOT log the deleted data).
7. Require API key authentication via shared middleware.

## Validation
Insert vetting_results and vetting_requests rows for an org_id. Call DELETE endpoint. Verify 200 response with correct record counts. Query database to confirm rows are deleted. Test with non-existent org_id: verify 200 with zero counts. Verify deletion is transactional (if one DELETE fails, neither commits).