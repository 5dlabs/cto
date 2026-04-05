Implement subtask 7001: Configure OpenClaw agent runtime with MCP tool-server connectivity

## Objective
Set up the OpenClaw agent runtime environment, configure the MCP tool-server connection, register all MCP tool definitions (sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup), and verify the agent can discover and invoke tools against backend service endpoints.

## Steps
Step 1: Initialize the OpenClaw agent project structure with configuration files for the MCP runtime. Step 2: Define MCP tool manifests for all 10 tools, specifying input schemas, output schemas, and backend service endpoint URLs (sourced from the sigma1-infra-endpoints ConfigMap via envFrom). Step 3: Configure the MCP tool-server with authentication credentials (service tokens or mTLS certs, per dp-6 decision). Step 4: Implement a health-check routine that verifies connectivity to each registered MCP tool's backend. Step 5: Write the agent's system prompt and personality configuration for Morgan (professional, sales-oriented, knowledgeable about rental equipment). Step 6: Validate that the agent runtime starts, connects to the tool-server, and can list all registered tools.

## Validation
Agent runtime starts without errors; MCP tool-server is reachable; all 10 tools are listed in the agent's tool registry; a smoke test invoking sigma1_catalog_search returns a valid response from the catalog backend.