## Implement Morgan AI Agent (Angie - OpenClaw/MCP)

### Objective
Deploy and configure the Morgan AI agent to handle all customer interactions via Signal, voice (ElevenLabs), and web chat. Integrate with all backend services and tool-server MCP tools.

### Ownership
- Agent: Angie
- Stack: OpenClaw/MCP
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6

### Implementation Details
{"steps": ["Deploy OpenClaw agent in openclaw namespace using provided deployment manifest.", "Configure Signal-CLI sidecar and ElevenLabs/Twilio integration for voice.", "Implement tool-server MCP tools for all backend service actions (catalog search, availability, quote, vetting, scoring, invoice, finance report, social curation/publish, equipment lookup).", "Integrate with all backend APIs using endpoints from sigma1-infra-endpoints ConfigMap.", "Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin.", "Configure web chat widget endpoint for website integration.", "Ensure Morgan responds within 10 seconds for simple queries."]}

### Subtasks
- [ ] Deploy OpenClaw agent base in openclaw namespace: Deploy the OpenClaw agent using the provided deployment manifest into the openclaw namespace. Configure base environment variables, resource limits, and wire up the sigma1-infra-endpoints ConfigMap via envFrom so all backend service URLs are available to the agent runtime.
- [ ] Integrate Signal-CLI sidecar for Signal messaging channel: Configure and deploy a Signal-CLI sidecar container alongside the OpenClaw agent pod to enable sending and receiving Signal messages. Wire the sidecar's REST/JSON-RPC API to the agent's messaging interface.
- [ ] Integrate ElevenLabs and Twilio for voice channel: Configure the voice pipeline: Twilio receives inbound calls and streams audio, ElevenLabs handles text-to-speech for agent responses, and a speech-to-text service transcribes caller input. Wire this pipeline into the OpenClaw agent's conversation flow.
- [ ] Implement MCP tools for catalog search, availability check, and quote generation: Build MCP tool-server tools that allow the Morgan agent to search the equipment catalog, check item availability, and generate/submit rental quotes by calling the appropriate backend APIs.
- [ ] Implement MCP tools for customer vetting, scoring, invoicing, and finance reporting: Build MCP tool-server tools for customer vetting/scoring via the Vetting Engine, invoice generation/lookup via Invoice/Billing, and finance report retrieval for admin queries.
- [ ] Implement MCP tools for social media curation/publishing and RMS equipment lookup: Build MCP tool-server tools for social media content curation and publishing via the Social Media Engine, and equipment data lookup via the RMS integration layer.
- [ ] Configure all Morgan agent skills and conversation routing: Configure the Morgan agent's skill definitions for sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, and admin. Set up intent routing so the agent selects the correct skill and tools based on conversation context.
- [ ] Implement web chat widget endpoint: Create the HTTP/WebSocket endpoint that the website's embedded chat widget will connect to for real-time conversations with the Morgan agent.
- [ ] End-to-end integration testing across all channels and skills: Perform comprehensive end-to-end testing of the Morgan agent across all three communication channels (Signal, voice, web chat) and all configured skills, validating response times and autonomous handling rates.