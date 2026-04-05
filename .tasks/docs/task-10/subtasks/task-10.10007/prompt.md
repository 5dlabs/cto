Implement subtask 10007: Create GDPR data deletion orchestration Job

## Objective
Implement a Kubernetes Job (triggered on-demand) that calls DELETE/anonymization endpoints on each service for a given customer, tracks completion per service, and records the deletion request in the database.

## Steps
1. Create a container image for the GDPR deletion Job (can share the base with the export Job).
2. The Job accepts a `CUSTOMER_ID` environment variable.
3. Workflow:
   a. Insert a new row into `audit.data_deletion_requests` with `customer_id` and `requested_at`.
   b. Call each service's deletion/anonymization endpoint using the respective JWT:
      - Customer Vetting: DELETE /api/v1/vetting/:org_id → expect 204
      - Finance: POST /api/v1/invoices/:customer_id/anonymize → anonymizes PII fields but retains records for tax compliance
      - RMS: POST /api/v1/customers/:customer_id/anonymize → anonymizes customer data in opportunities/projects
      - Social Engine: DELETE /api/v1/photos?customer_id=X → removes all associated photos
      - Equipment Catalog: POST /api/v1/equipment/:customer_id/anonymize (if customer data exists)
   c. Track which services completed successfully in a `services_purged` array.
   d. Update `data_deletion_requests` row with `completed_at` and `services_purged`.
4. Error handling: if a service call fails, log the error, continue with remaining services, and record only successful purges. Allow re-running for failed services.
5. Create ServiceAccount and RBAC for the Job.
6. Insert audit_log entries for each deletion action performed.
7. Create the Job template manifest.

## Validation
Run the Job with a test customer ID. Verify `data_deletion_requests` row is created. Verify Customer Vetting data returns 404 after deletion. Verify Finance records exist but PII fields are nulled. Verify Social Engine photos are removed. Verify `services_purged` array contains all successfully purged service names. Verify `completed_at` is set.