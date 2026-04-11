<identity>
You are angie working on subtask 7003 of task 7.
</identity>

<context>
<scope>
Author agents/morgan/tools.yaml defining all 10 business MCP tools with correct in-cluster service URLs, HTTP methods, parameter schemas, and response field descriptions.
</scope>
</context>

<implementation_plan>
1. Create agents/morgan/tools.yaml. For each tool define: tool_id, display_name, description, http_method, url_template (with {param} placeholders), request_schema (JSON Schema for body or query params), response_schema (JSON Schema describing expected response shape), and timeout_seconds.
2. sigma1_catalog_search: GET http://equipment-catalog-svc.sigma1.svc.cluster.local:8080/api/v1/catalog/products?search={query}. Response schema: array of {name: string, product_id: string, day_rate: number}.
3. sigma1_check_availability: GET .../products/{product_id}/availability?from={from}&to={to}. Params: product_id (path), from/to (ISO8601 query). Response: {quantity_available: integer}.
4. sigma1_generate_quote: POST http://rms-svc.sigma1.svc.cluster.local:8080/api/v1/opportunities. Body schema: {customer_id, line_items: [{product_id, quantity, days}], event_date}.
5. sigma1_vet_customer: POST http://customer-vetting-svc.sigma1.svc.cluster.local:8080/api/v1/vetting/run. Body: {org_name, contact_email, event_description}. Response: {vetting_id: string, status: string}.
6. sigma1_score_lead: GET http://rms-svc.sigma1.svc.cluster.local:8080/api/v1/opportunities/{id}/score. Path param: id. Response: {score: number, tier: string}.
7. sigma1_create_invoice: POST http://finance-svc.sigma1.svc.cluster.local:8080/api/v1/invoices. Body: {opportunity_id, customer_id, line_items}. Response: {invoice_id, total_cents}.
8. sigma1_finance_report: GET http://finance-svc.sigma1.svc.cluster.local:8080/api/v1/finance/reports/revenue?period={period}. Param: period (e.g. 2025-Q1). Response: {total_revenue_cents, invoice_count}.
9. sigma1_social_curate: POST http://social-engine-svc.sigma1.svc.cluster.local:8080/api/v1/social/upload. Multipart body: file + metadata JSON. Response: {draft_id}.
10. sigma1_social_publish: POST .../social/drafts/{draft_id}/publish. Path param: draft_id. Response: {published_url}.
11. sigma1_equipment_lookup: GET http://equipment-catalog-svc.sigma1.svc.cluster.local:8080/api/v1/equipment-api/catalog. Response: array of catalog items.
</implementation_plan>

<validation>
`openclaw validate agents/morgan/tools.yaml` exits 0. Tool count in file equals 10 (excluding GDPR tools which are in separate file). Each tool entry has non-empty url_template, http_method, request_schema, and response_schema.
</validation>