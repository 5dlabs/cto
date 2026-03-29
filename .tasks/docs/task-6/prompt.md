Implement task 6: Implement Morgan AI Agent Core (Angie - OpenClaw/MCP)

## Goal
Develop the core Morgan AI agent, integrating Signal, voice (ElevenLabs/Twilio), and web chat capabilities. This task focuses on the agent's ability to understand natural language and utilize backend services via MCP tools.

## Task Context
- Agent owner: angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 1, 2, 3, 4, 5

## Implementation Plan
1. Deploy the `morgan` OpenClaw agent to the `openclaw` namespace, configuring it to use `openai-api/gpt-5.4-pro` as the model.2. Set up Signal-CLI (as a sidecar or separate pod) and integrate it with the Morgan agent for receiving and sending messages/photos.3. Integrate ElevenLabs for natural voice synthesis and Twilio for SIP/PSTN voice calls, configuring necessary API keys and webhooks.4. Define and implement the following MCP tools, ensuring they correctly call the respective backend services:    - `sigma1_catalog_search` (calls Equipment Catalog Service, Task 2)    - `sigma1_check_availability` (calls Equipment Catalog Service, Task 2)    - `sigma1_generate_quote` (calls RMS Service, Task 3)    - `sigma1_vet_customer` (calls Customer Vetting Service, Task 5)    - `sigma1_score_lead` (calls Customer Vetting Service, Task 5)    - `sigma1_create_invoice` (calls Finance Service, Task 4)    - `sigma1_finance_report` (calls Finance Service, Task 4)5. Implement initial skills: `sales-qual` (using `sigma1_vet_customer`, `sigma1_score_lead`) and `quote-gen` (using `sigma1_catalog_search`, `sigma1_check_availability`, `sigma1_generate_quote`).6. Configure Cloudflare Tunnel for secure external access to the Morgan agent's web chat interface and webhooks.

## Acceptance Criteria
1. Verify the Morgan agent pod is running and accessible.2. Send a test message via Signal to Morgan and confirm a response is received.3. Initiate a voice call via Twilio/ElevenLabs and verify Morgan can respond verbally.4. Test `sales-qual` skill: Send a natural language query like 'Can you qualify a new lead for me?' and verify Morgan triggers `sigma1_vet_customer` and `sigma1_score_lead` tools, returning a lead score.5. Test `quote-gen` skill: Ask Morgan to 'Generate a quote for 5 projectors for next week' and verify it uses `sigma1_catalog_search`, `sigma1_check_availability`, and `sigma1_generate_quote` tools, providing a quote ID.6. Confirm Cloudflare Tunnel is correctly routing traffic to the Morgan agent.

## Subtasks
- Deploy Morgan OpenClaw agent and configure LLM: Deploy the `morgan` OpenClaw agent to the `openclaw` Kubernetes namespace and configure it to use `openai-api/gpt-5.4-pro` as its underlying language model.
- Integrate Signal-CLI for messaging: Set up Signal-CLI integration with the Morgan agent for receiving and sending messages and photos.
- Integrate ElevenLabs and Twilio for voice communication: Integrate ElevenLabs for natural voice synthesis and Twilio for SIP/PSTN voice calls, configuring necessary API keys and webhooks.
- Implement MCP tools for Catalog and Customer Vetting services: Define and implement OpenClaw MCP tools `sigma1_catalog_search`, `sigma1_check_availability`, `sigma1_vet_customer`, and `sigma1_score_lead` to interact with the Equipment Catalog and Customer Vetting services.
- Implement MCP tools for RMS and Finance services: Define and implement OpenClaw MCP tools `sigma1_generate_quote`, `sigma1_create_invoice`, and `sigma1_finance_report` to interact with the RMS and Finance services.
- Develop core AI skills and configure Cloudflare Tunnel: Implement the `sales-qual` and `quote-gen` AI skills, and configure Cloudflare Tunnel for secure external access to the Morgan agent's web chat interface and webhooks.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.