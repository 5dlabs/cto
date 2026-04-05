Implement task 7: Implement Morgan AI Agent (Angie - OpenClaw/MCP)

## Goal
Deploy and configure the Morgan AI agent to handle Signal, voice, and web chat, orchestrating all backend services via MCP tools.

## Task Context
- Agent owner: angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
{"steps":["Deploy OpenClaw agent in Kubernetes using the provided deployment manifest, referencing AGENT_ID, MODEL, and workspace volume.","Configure Signal-CLI and ElevenLabs integrations for messaging and voice.","Register all MCP tools (sigma1_catalog_search, sigma1_check_availability, etc.) with tool-server, mapping to backend service endpoints.","Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin.","Integrate with Twilio for phone/SIP.","Ensure Morgan can respond to queries within 10 seconds and handle 500+ concurrent Signal connections.","Test end-to-end flows: Signal message → Morgan → backend action → confirmation."]}

## Acceptance Criteria
Morgan responds to Signal, voice, and web chat within 10 seconds. All MCP tools are accessible and functional. End-to-end flows (lead qualification, quote, vetting, invoice) complete successfully. Handles 500+ concurrent Signal connections.

## Subtasks
- Deploy OpenClaw agent in Kubernetes with workspace volume and configuration: Create and apply the Kubernetes deployment manifest for the OpenClaw Morgan agent, including AGENT_ID, MODEL env vars, workspace persistent volume, resource requests/limits, and service account.
- Integrate Signal-CLI for messaging channel: Deploy and configure Signal-CLI as a sidecar or separate pod, link it to the Morgan agent so inbound Signal messages are forwarded to the agent and responses are sent back via Signal.
- Integrate ElevenLabs for voice synthesis: Configure the Morgan agent to use ElevenLabs API for text-to-speech voice responses, including voice selection, streaming audio output, and error handling.
- Integrate Twilio for phone and SIP connectivity: Configure Twilio for inbound/outbound phone calls and SIP trunking, connecting voice calls to the Morgan agent with ElevenLabs voice output.
- Register all MCP tools with tool-server mapping to backend service endpoints: Register every MCP tool (sigma1_catalog_search, sigma1_check_availability, sigma1_customer_vet, sigma1_create_quote, sigma1_submit_invoice, sigma1_social_post, etc.) with the tool-server, mapping each to the correct backend service API endpoint.
- Implement skills: sales-qual, customer-vet, quote-gen, upsell: Implement the sales-oriented skill set for Morgan: lead qualification (sales-qual), customer vetting orchestration (customer-vet), quote generation (quote-gen), and upsell recommendation (upsell).
- Implement skills: finance, social-media, rms-*, admin: Implement the operational skill set for Morgan: finance reporting and invoicing (finance), social media management (social-media), rental management system operations (rms-*), and administrative functions (admin).
- Set up web chat endpoint for Morgan agent: Expose a WebSocket (or HTTP streaming) endpoint from the Morgan agent for the web chat frontend to connect to, enabling real-time conversational interaction via browser.
- Performance optimization and load testing for 500+ concurrent Signal connections: Optimize the Morgan agent and Signal-CLI setup to achieve <10 second response times and handle 500+ concurrent Signal connections, conducting load tests to validate.
- End-to-end conversation flow testing across all channels and skills: Validate complete end-to-end flows across Signal, voice, and web chat channels, covering all skill domains (sales, vetting, quoting, invoicing, social, RMS, admin).

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.