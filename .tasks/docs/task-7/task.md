## Implement Morgan AI Agent (Angie - OpenClaw/MCP)

### Objective
Deploy and configure the Morgan AI agent with Signal, voice (ElevenLabs), and web chat integrations. Connects to all backend services via MCP tools for unified customer interaction.

### Ownership
- Agent: angie
- Stack: OpenClaw/MCP
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 1

### Implementation Details
{"steps": ["Deploy OpenClaw agent as per infra spec, referencing AGENT_ID=morgan and MODEL=openai-api/gpt-5.4-pro.", "Configure Signal-CLI and ElevenLabs voice integration using secrets.", "Implement MCP tool-server access for all backend service tools (catalog, RMS, finance, vetting, social).", "Configure web chat widget endpoint for website integration.", "Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin as per PRD.", "Test all inbound and outbound flows: Signal, voice, web chat.", "Reference all service endpoints from 'sigma1-infra-endpoints' ConfigMap.", "Write end-to-end tests for lead qualification, quote, vetting, and invoice flows."]}

### Subtasks
- [ ] Deploy OpenClaw agent with base configuration and GPT-5.4-pro model setup: Deploy the OpenClaw agent runtime, configure AGENT_ID=morgan with MODEL=openai-api/gpt-5.4-pro, wire up environment variables from the sigma1-infra-endpoints ConfigMap, and verify the agent starts and responds to a basic health check.
- [ ] Implement Signal-CLI integration for inbound/outbound messaging: Configure Signal-CLI as a messaging channel adapter for the Morgan agent, enabling inbound message reception and outbound message sending via the Signal protocol.
- [ ] Implement ElevenLabs voice integration for voice channel: Integrate ElevenLabs TTS/STT with the Morgan agent to enable voice-based customer interactions, including speech-to-text input and text-to-speech output.
- [ ] Implement web chat widget endpoint for website integration: Create the web chat HTTP/WebSocket endpoint that the Sigma-1 website chat widget will connect to, enabling real-time bidirectional messaging with the Morgan agent.
- [ ] Configure MCP tool-server with Equipment Catalog and RMS backend tools: Set up the MCP tool-server and register tools for the Equipment Catalog service (search, lookup, availability) and Rental Management System (create rental, check status, manage returns).
- [ ] Configure MCP tool-server with Finance, Vetting, and Social Media backend tools: Register MCP tools for the Finance service (invoicing, payments), Customer Vetting service (credit checks, verification), and Social Media Engine (content publishing, portfolio).
- [ ] Implement agent skills: sales-qual, customer-vet, and quote-gen: Implement the sales qualification, customer vetting, and quote generation skill definitions within the Morgan agent, including conversation flows, tool orchestration logic, and decision trees.
- [ ] Implement agent skills: upsell, finance, social-media, and admin: Implement the upselling, finance management, social media, and administrative skill definitions within the Morgan agent.
- [ ] End-to-end integration tests for complete multi-channel flows: Write and execute comprehensive end-to-end tests covering the full lead-to-invoice lifecycle across Signal, voice, and web chat channels, validating all skills and MCP tool interactions.