Implement subtask 7001: Configure OpenClaw agent runtime and MCP tool-server connection

## Objective
Set up the OpenClaw agent runtime environment, configure the MCP tool-server endpoint, register all tool definitions (sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup), and verify the agent can discover and invoke tools.

## Steps
1. Initialize the OpenClaw agent project and configure the runtime entry point.
2. Define the MCP tool-server connection (endpoint URL, auth credentials from infra ConfigMap).
3. Register all 10 MCP tool definitions with their input/output schemas: sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup.
4. Implement the tool invocation adapter that translates agent intents to MCP tool calls and parses responses.
5. Configure Morgan's system prompt with personality, role boundaries, and tool usage instructions.
6. Add structured logging for all tool invocations (tool name, latency, success/failure).
7. Verify the agent can list available tools and execute a basic tool call (e.g., sigma1_catalog_search) against a mock or live backend.

## Validation
Agent runtime starts without errors; all 10 tools are discoverable via MCP; a test tool call to sigma1_catalog_search returns a valid response; tool invocation logs are emitted with correct structure.