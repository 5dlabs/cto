Implement subtask 7004: Register MCP tools for Equipment Catalog and RMS backend services

## Objective
Define and register MCP tool definitions for the Equipment Catalog API and the Rental Management System (RMS) API, mapping request/response schemas.

## Steps
1. For the Equipment Catalog service, define MCP tools: search-equipment, get-equipment-details, check-availability, get-pricing.
2. For the RMS service, define MCP tools: create-rental, update-rental, cancel-rental, get-rental-status, list-rentals, schedule-delivery, schedule-pickup.
3. Each tool definition must include: name, description (for LLM context), input JSON schema, output JSON schema, and the HTTP endpoint/method it maps to.
4. Configure the tool server to resolve service URLs from environment variables (injected from sigma1-infra-endpoints).
5. Register all tools with the OpenClaw agent's tool registry.
6. Implement error mapping: backend HTTP errors → meaningful tool error responses the LLM can interpret.

## Validation
Invoke each MCP tool via the agent's tool execution endpoint with sample inputs; verify correct HTTP calls are made to the backend services (use mock/stub if services aren't live); verify response schemas match expectations.