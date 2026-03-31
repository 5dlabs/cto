Implement task 7: Implement Morgan AI Agent (Angie - OpenClaw/MCP)

## Goal
Deploy and configure the Morgan AI agent to handle all customer interactions via Signal, voice (ElevenLabs), and web chat. Integrates with all backend services and exposes MCP tools for business workflows.

## Task Context
- Agent owner: angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
{"steps": ["Deploy OpenClaw agent as per infra spec, using AGENT_ID=morgan and model openai-api/gpt-5.4-pro.", "Configure Signal-CLI and ElevenLabs voice integration.", "Implement tool-server plugins for all MCP tools (catalog search, check availability, generate quote, vet customer, score lead, create invoice, finance report, social curate/publish, equipment lookup).", "Integrate with all backend service APIs using endpoints from ConfigMap.", "Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin.", "Configure web chat widget endpoint for website integration.", "Test end-to-end flows: Signal, voice, web chat."]}

## Acceptance Criteria
Morgan responds to Signal, voice, and web chat within 10 seconds for simple queries. All MCP tools are callable and return correct results. End-to-end flows (lead qualification, quote, vetting, invoice) complete successfully.

## Subtasks
- Deploy OpenClaw agent with AGENT_ID=morgan and gpt-5.4-pro model configuration: Stand up the core OpenClaw agent instance, configure AGENT_ID=morgan, wire up the openai-api/gpt-5.4-pro model, and verify the agent boots and responds to a basic health check. Pull backend service endpoints from the project infra ConfigMap via envFrom.
- Integrate Signal-CLI messaging channel: Configure Signal-CLI as a messaging channel for Morgan so customers can interact via Signal. Set up the Signal-CLI process (sidecar or separate pod), register/link the phone number, and wire inbound/outbound message flow to the OpenClaw agent.
- Integrate ElevenLabs voice channel: Configure ElevenLabs TTS/STT as the voice channel for Morgan, enabling customers to interact via voice calls. Wire speech-to-text input and text-to-speech output through the OpenClaw agent.
- Implement MCP tool-server plugins for catalog and equipment tools: Build MCP tool-server plugins for: catalog_search, check_availability, and equipment_lookup. Each plugin calls the corresponding Equipment Catalog backend service API and returns structured results to the agent.
- Implement MCP tool-server plugins for business workflow tools (quote, vetting, lead scoring, invoice, finance): Build MCP tool-server plugins for: generate_quote, vet_customer, score_lead, create_invoice, and finance_report. Each plugin integrates with the respective backend service API.
- Implement MCP tool-server plugins for social media and RMS tools: Build MCP tool-server plugins for: social_curate, social_publish, and all RMS tools (rms-*). Each plugin integrates with the Social Media and RMS backend service APIs respectively.
- Configure agent skills and persona routing: Define and configure all Morgan agent skills (sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin) so the agent can route conversations to the appropriate skill with the correct tool bindings and system prompts.
- Implement web chat widget endpoint: Create a web chat endpoint that the frontend website can connect to for real-time chat with Morgan. Expose a WebSocket or SSE-based API that the web chat widget will consume.
- End-to-end integration testing across all channels and workflows: Run comprehensive end-to-end tests covering all three channels (Signal, voice, web chat) and all major business workflows (lead qualification → quote → vetting → invoice) to ensure the complete Morgan agent system works together.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.