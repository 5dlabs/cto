Implement subtask 4012: Implement GDPR export and delete endpoints

## Objective
Implement GET /internal/gdpr/export/:org_id returning all finance data for an org and DELETE /internal/gdpr/delete/:org_id anonymizing org_id on invoices and payroll entries.

## Steps
Create src/handlers/gdpr.rs. GET /internal/gdpr/export/:org_id: validate X-Internal-Key header against INTERNAL_API_KEY env var (401 if missing/wrong). SELECT invoices WHERE org_id=$1, SELECT invoice_line_items for those invoices, SELECT payments for those invoices, SELECT payroll_entries WHERE worker_id=$1 (approximation — org may not map directly; use org_id field if added to payroll). Return JSON with keys invoices, payments, payroll. DELETE /internal/gdpr/delete/:org_id: within sqlx transaction, UPDATE finance.invoices SET org_id=NULL WHERE org_id=$1; UPDATE finance.payroll_entries SET worker_id=NULL WHERE worker_id=$1 (cast org_id as worker_id approximation — clarify data model if needed). INSERT INTO finance.gdpr_deletions (entity='finance', deleted_org_id=$1, deleted_at=NOW()). Return 204.

## Validation
GET /internal/gdpr/export/:id with valid X-Internal-Key returns JSON with invoices array containing seeded records. DELETE /internal/gdpr/delete/:id returns 204. Subsequent GET /api/v1/invoices?org_id=:id returns empty list. finance.gdpr_deletions table has one row for the org. Request without X-Internal-Key returns 401.