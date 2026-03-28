Implement subtask 6005: Implement MCP tools for RMS and Finance services

## Objective
Define and implement OpenClaw MCP tools `sigma1_generate_quote`, `sigma1_create_invoice`, and `sigma1_finance_report` to interact with the RMS and Finance services.

## Steps
1. Define tool schemas for `sigma1_generate_quote` that maps to the RMS Service (Task 3) API.2. Define tool schemas for `sigma1_create_invoice` and `sigma1_finance_report` that map to the Finance Service (Task 4) APIs.3. Implement the backend logic within the OpenClaw agent to make HTTP calls to these services.

## Validation
1. Verify the tools are registered with the OpenClaw agent.2. Use a test prompt that should trigger `sigma1_generate_quote` (e.g., 'Generate a quote for X') and verify the tool is called.3. Use a test prompt that should trigger `sigma1_create_invoice` (e.g., 'Create an invoice for project Y') and verify the tool is called.