Implement subtask 7005: Register MCP tools for Finance, Vetting, and Social Media backend services

## Objective
Define and register MCP tool definitions for the Finance/Invoicing, Customer Vetting, and Social Media Engine APIs.

## Steps
1. For the Finance service, define MCP tools: create-quote, finalize-quote, generate-invoice, get-invoice-status, process-payment, list-invoices.
2. For the Vetting service, define MCP tools: submit-vet-request, get-vet-status, approve-customer, flag-customer.
3. For the Social Media Engine, define MCP tools: create-post-draft, submit-for-approval, get-approval-status, publish-post, list-portfolio-items.
4. Each tool must include name, description, input/output JSON schemas, and endpoint mapping.
5. Configure service URL resolution from sigma1-infra-endpoints environment variables.
6. Register all tools with the OpenClaw agent's tool registry.
7. Implement error mapping for each service's error codes.

## Validation
Invoke each MCP tool via the agent's tool execution endpoint with sample inputs; verify correct HTTP calls to backend services; verify response schema conformance; test error cases return meaningful messages.