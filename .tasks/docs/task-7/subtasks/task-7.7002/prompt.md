Implement subtask 7002: Configure MCP Tool Server with all 10 tool definitions and JSON Schema mappings

## Objective
Define the MCP Tool Server configuration mapping all 10 backend service tools with their REST/gRPC endpoint URLs, parameter JSON Schemas, response schemas, and authentication headers. Each tool must be individually testable.

## Steps
1. Create MCP Tool Server configuration file(s) with 10 tool definitions:
   - `sigma1_catalog_search`: GET to equipment-catalog service `/api/v1/catalog/products`, params: q (string), category (string, optional). Response: array of product objects.
   - `sigma1_check_availability`: GET to equipment-catalog `/api/v1/catalog/products/{id}/availability`, params: id (string), from (ISO date), to (ISO date). Response: availability object with quantity/conflicts.
   - `sigma1_generate_quote`: POST to RMS `/api/v1/opportunities`, body: customer info, line items, event details. Response: opportunity ID, total, status.
   - `sigma1_vet_customer`: POST to vetting service `/api/v1/vetting/run`, body: org name, contact info. Response: rating (GREEN/YELLOW/RED), details.
   - `sigma1_score_lead`: POST to RMS `/api/v1/opportunities/{id}/score`, params: opportunity ID. Response: score, factors.
   - `sigma1_create_invoice`: POST to finance service `/api/v1/invoices`, body: opportunity ID, line items, terms. Response: invoice ID, PDF URL.
   - `sigma1_finance_report`: GET to finance `/api/v1/finance/reports/{type}`, params: type (revenue/aging/etc.), period. Response: report data.
   - `sigma1_social_curate`: POST to social engine `/api/v1/social/upload`, body: image data/URL, event context. Response: draft ID, curated content.
   - `sigma1_social_publish`: POST to social engine `/api/v1/social/drafts/{id}/approve`. Response: published URLs.
   - `sigma1_equipment_lookup`: GET to equipment API `/api/v1/equipment-api/catalog`. Response: machine-readable catalog.
2. For each tool, define complete JSON Schema for input parameters and expected response format.
3. Configure authentication: each tool call includes API key from `sigma1-service-api-keys` secret in Authorization header.
4. Set per-tool timeout values (catalog/availability: 5s, quote/vetting: 15s, social upload: 30s).
5. Configure base URLs using service DNS names within the cluster (e.g., `http://equipment-catalog.sigma1.svc.cluster.local`).

## Validation
For each of the 10 tools: invoke via MCP Tool Server with valid test parameters and verify a successful HTTP response from the corresponding backend service. Verify JSON Schema validation rejects malformed parameters. Verify auth headers are correctly attached. Verify timeout behavior by simulating a slow backend.