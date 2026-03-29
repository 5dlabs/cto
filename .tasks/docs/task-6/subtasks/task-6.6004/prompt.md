Implement subtask 6004: Implement MCP tools for Catalog and Customer Vetting services

## Objective
Define and implement OpenClaw MCP tools `sigma1_catalog_search`, `sigma1_check_availability`, `sigma1_vet_customer`, and `sigma1_score_lead` to interact with the Equipment Catalog and Customer Vetting services.

## Steps
1. Define tool schemas for `sigma1_catalog_search` and `sigma1_check_availability` that map to the Equipment Catalog Service (Task 2) APIs.2. Define tool schemas for `sigma1_vet_customer` and `sigma1_score_lead` that map to the Customer Vetting Service (Task 5) APIs.3. Implement the backend logic within the OpenClaw agent to make HTTP calls to these services.

## Validation
1. Verify the tools are registered with the OpenClaw agent.2. Use a test prompt that should trigger `sigma1_catalog_search` (e.g., 'What projectors do you have?') and verify the tool is called and returns data.3. Use a test prompt that should trigger `sigma1_vet_customer` (e.g., 'Vet this company') and verify the tool is called.