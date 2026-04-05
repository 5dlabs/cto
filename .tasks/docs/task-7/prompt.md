Implement task 7: Implement Morgan AI Agent (Angie - OpenClaw/MCP)

## Goal
Develop the Morgan AI agent to handle all customer interactions via Signal, voice, and web chat, orchestrating backend services via MCP tools.

## Task Context
- Agent owner: Angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
{"steps": ["Configure OpenClaw agent runtime with MCP tool-server access.", "Integrate Signal-CLI for messaging, ElevenLabs for voice, and Twilio for phone/SIP.", "Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin.", "Wire up MCP tools: sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup.", "Implement web chat widget integration for website.", "Ensure Morgan can respond to simple queries in <10s and orchestrate full lead-to-invoice flows.", "Add logging and observability hooks for monitoring."]}

## Acceptance Criteria
Morgan responds to Signal, voice, and web chat within 10s for simple queries; can complete lead qualification, vetting, quote, and invoice flows end-to-end; logs all actions; >80% code coverage on skills and tool integrations.

## Subtasks
- Configure OpenClaw agent runtime and MCP tool-server connection: Set up the OpenClaw agent runtime environment, configure the MCP tool-server endpoint, register all tool definitions (sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup), and verify the agent can discover and invoke tools.
- Integrate Signal-CLI for bidirectional messaging: Set up Signal-CLI for Morgan to receive and send messages via Signal, including message parsing, response routing, and conversation state management.
- Integrate ElevenLabs voice synthesis for voice responses: Implement ElevenLabs TTS integration so Morgan can generate spoken audio responses for voice channel interactions.
- Integrate Twilio for phone/SIP voice channel: Set up Twilio integration for inbound/outbound voice calls and SIP, routing voice interactions through Morgan's agent runtime with ElevenLabs TTS output.
- Implement sales and customer skills: sales-qual, customer-vet, quote-gen, upsell: Develop the core customer-facing skills that handle lead qualification, customer vetting, quote generation, and upselling, wired to the corresponding MCP tools.
- Implement finance skill: invoicing and finance reporting: Develop the finance skill for Morgan to create invoices and generate financial reports using the sigma1_create_invoice and sigma1_finance_report MCP tools.
- Implement social media skills: social-curate and social-publish: Develop skills for Morgan to curate and publish social media content using the sigma1_social_curate and sigma1_social_publish MCP tools.
- Implement RMS and admin skills: Develop skills for rental management system operations (equipment lookup, availability, status) and administrative functions.
- Implement web chat widget integration: Build the web chat widget interface and backend WebSocket/HTTP endpoint that connects website visitors to Morgan's agent runtime.
- Implement end-to-end lead-to-invoice flow and observability: Wire together all skills into the complete lead-to-invoice orchestration flow, add comprehensive logging and observability hooks, and validate the full flow end-to-end.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.