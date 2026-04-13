<identity>
You are rex working on subtask 2010 of task 2.
</identity>

<context>
<scope>
Write GET /internal/gdpr/export/:customer_id and DELETE /internal/gdpr/delete/:customer_id handlers that respectively return and remove all catalog-related PII for a given customer from availability_blocks checkout records.
</scope>
</context>

<implementation_plan>
Create src/handlers/gdpr.rs. GET /internal/gdpr/export/:customer_id: SELECT * FROM catalog.availability_blocks WHERE customer_id = $1 (ensure customer_id column exists — add migration 0002_add_customer_id_to_availability_blocks.sql: ALTER TABLE catalog.availability_blocks ADD COLUMN IF NOT EXISTS customer_id UUID). Return JSON array of matching rows. If no rows return empty array (200). DELETE /internal/gdpr/delete/:customer_id: DELETE FROM catalog.availability_blocks WHERE customer_id = $1. Also nullify or anonymize any other PII fields if applicable. Return 204 No Content on success. Note: access control for these endpoints is enforced at network level via Cilium NetworkPolicy (document this); no JWT is required. However add a check: if X-Internal-Request header is not present return 403 as a defense-in-depth measure (Cilium policy is primary enforcement). Wire under /internal/gdpr/ router group. Write migration 0002 file.
</implementation_plan>

<validation>
Insert an availability_block with customer_id = test-uuid. GET /internal/gdpr/export/test-uuid returns HTTP 200 with JSON array containing that block. DELETE /internal/gdpr/delete/test-uuid returns HTTP 204. Subsequent GET /internal/gdpr/export/test-uuid returns HTTP 200 with empty array []. Calling without X-Internal-Request header returns HTTP 403.
</validation>