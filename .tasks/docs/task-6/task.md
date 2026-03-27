## Implement Morgan AI Agent Core (Angie - OpenClaw/MCP)

### Objective
Develop the core Morgan AI agent, integrating Signal, voice (ElevenLabs/Twilio), and web chat capabilities. This task focuses on the agent's ability to understand natural language and utilize backend services via MCP tools.

### Ownership
- Agent: angie
- Stack: OpenClaw/MCP
- Priority: high
- Status: pending
- Dependencies: 1, 2, 3, 4, 5

### Implementation Details
1. Deploy the `morgan` OpenClaw agent to the `openclaw` namespace, configuring it to use `openai-api/gpt-5.4-pro` as the model.
2. Set up Signal-CLI (as a sidecar or separate pod) and integrate it with the Morgan agent for receiving and sending messages/photos.
3. Integrate ElevenLabs for natural voice synthesis and Twilio for SIP/PSTN voice calls, configuring necessary API keys and webhooks.
4. Define and implement the following MCP tools, ensuring they correctly call the respective backend services:
    - `sigma1_catalog_search` (calls Equipment Catalog Service, Task 2)
    - `sigma1_check_availability` (calls Equipment Catalog Service, Task 2)
    - `sigma1_generate_quote` (calls RMS Service, Task 3)
    - `sigma1_vet_customer` (calls Customer Vetting Service, Task 5)
    - `sigma1_score_lead` (calls Customer Vetting Service, Task 5)
    - `sigma1_create_invoice` (calls Finance Service, Task 4)
    - `sigma1_finance_report` (calls Finance Service, Task 4)
5. Implement initial skills: `sales-qual` (using `sigma1_vet_customer`, `sigma1_score_lead`) and `quote-gen` (using `sigma1_catalog_search`, `sigma1_check_availability`, `sigma1_generate_quote`).
6. Configure Cloudflare Tunnel for secure external access to the Morgan agent's web chat interface and webhooks.

### Subtasks
- [ ] Implement Implement Morgan AI Agent Core (Angie - OpenClaw/MCP): Develop the core Morgan AI agent, integrating Signal, voice (ElevenLabs/Twilio), and web chat capabilities. This task focuses on the agent's ability to understand natural language and utilize backend services via MCP tools.