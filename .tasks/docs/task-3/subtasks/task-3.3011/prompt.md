Implement subtask 3011: Implement GDPR export and delete endpoints

## Objective
Implement GET /internal/gdpr/export/:customer_id and DELETE /internal/gdpr/delete/:customer_id on the grpc-gateway mux. Export returns all customer data; delete anonymizes customer_id fields and logs to audit schema.

## Steps
Create internal/gdpr/handler.go with a plain net/http handler (not gRPC, registered directly on the gateway mux or a separate internal mux). GET /internal/gdpr/export/:customer_id: query opportunities, projects, and inventory_transactions WHERE customer_id = $1; serialize to JSON response. DELETE /internal/gdpr/delete/:customer_id: within a pgx transaction, UPDATE rms.opportunities SET customer_id = NULL WHERE customer_id = $1; UPDATE rms.projects SET customer_id = NULL WHERE customer_id = $1; INSERT INTO audit.gdpr_deletions (entity='rms', deleted_customer_id=$1, deleted_at=NOW()). Return 204 on success. Ensure audit schema and gdpr_deletions table exist (add migration if needed). Protect endpoints with a simple internal API key header check (X-Internal-Key must match INTERNAL_API_KEY env var).

## Validation
GET /internal/gdpr/export/:id with valid X-Internal-Key returns JSON containing opportunities and projects arrays. DELETE /internal/gdpr/delete/:id returns 204; subsequent GET /api/v1/opportunities?customer_id=:id returns empty list; audit.gdpr_deletions has one row for the deleted id. Request without X-Internal-Key returns 401.