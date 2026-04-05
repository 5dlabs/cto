Implement subtask 10006: Create GDPR data export orchestration Job

## Objective
Implement a Kubernetes Job (triggered on-demand) that queries each service's API for customer data, aggregates results into a JSON file, uploads to R2 with a signed URL (7-day expiry), and records the export request in the database.

## Steps
1. Create a container image for the GDPR export Job (lightweight, e.g., alpine + curl + jq, or a small compiled binary).
2. The Job accepts a `CUSTOMER_ID` environment variable.
3. Workflow:
   a. Insert a new row into `audit.data_export_requests` with `customer_id` and `requested_at`.
   b. Call each service's customer data API endpoint using the respective service JWT token:
      - Equipment Catalog: GET /api/v1/equipment?customer_id=X
      - RMS: GET /api/v1/opportunities?customer_id=X, GET /api/v1/projects?customer_id=X
      - Finance: GET /api/v1/invoices?customer_id=X
      - Customer Vetting: GET /api/v1/vetting/:org_id
      - Social Engine: GET /api/v1/photos?customer_id=X
   c. Aggregate all responses into a single JSON structure: `{ customer_id, exported_at, data: { equipment: [...], opportunities: [...], ... } }`
   d. Upload JSON to R2 bucket at path `gdpr-exports/{customer_id}/{timestamp}.json`
   e. Generate a signed URL with 7-day expiry
   f. Update `data_export_requests` row with `completed_at`, `export_url`, `expires_at`
4. Create ServiceAccount with appropriate RBAC to read service token Secrets.
5. Mount R2 credentials from existing Secrets.
6. Configure error handling: if any service call fails, record partial export and mark which services succeeded in the metadata.
7. Create a Job template manifest that can be triggered via `kubectl create job --from=cronjob/gdpr-export` or directly.

## Validation
Run the Job with a test customer ID. Verify `data_export_requests` row is created. Verify API calls are made to all services (check audit logs or service logs). Verify JSON file is uploaded to R2 at the correct path. Verify signed URL is accessible and returns the aggregated JSON. Verify `completed_at` and `export_url` are populated in the database.