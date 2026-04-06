Implement subtask 7005: Implement MCP tools for customer vetting, scoring, invoicing, and finance reporting

## Objective
Build MCP tool-server tools for customer vetting/scoring via the Vetting Engine, invoice generation/lookup via Invoice/Billing, and finance report retrieval for admin queries.

## Steps
1. Define `customer_vet` MCP tool: accepts customer details (name, company, references), calls Vetting Engine API to initiate vetting, returns vetting status/result.
2. Define `customer_score` MCP tool: accepts customer ID, calls Vetting Engine scoring endpoint, returns risk score and recommendation.
3. Define `invoice_create` MCP tool: accepts quote ID or order details, calls Invoice/Billing API to generate an invoice, returns invoice details.
4. Define `invoice_lookup` MCP tool: accepts invoice ID or customer ID, calls Invoice/Billing API, returns invoice status and payment info.
5. Define `finance_report` MCP tool: accepts report type (revenue, outstanding, etc.) and date range, calls Finance API, returns formatted report data.
6. Use endpoint URLs from sigma1-infra-endpoints ConfigMap env vars.
7. Implement input validation and error handling for each tool.
8. Register all tools with the OpenClaw agent's tool registry.

## Validation
Invoke each tool with valid sample data; `customer_vet` triggers a vetting process and returns status; `customer_score` returns a numeric score; `invoice_create` returns a valid invoice; `invoice_lookup` retrieves existing invoices; `finance_report` returns structured report data; invalid inputs produce clear error messages.