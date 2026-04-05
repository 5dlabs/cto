Implement task 7: Implement Morgan AI Agent (Angie - OpenClaw/MCP)

## Goal
Develop the Morgan AI agent to handle all customer interactions via Signal, voice (ElevenLabs), and web chat, orchestrating backend service calls and workflows using MCP tools.

## Task Context
- Agent owner: Angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
{"steps": ["Configure OpenClaw agent runtime with MCP tool-server access.", "Integrate Signal-CLI for messaging, ElevenLabs for voice, and web chat widget.", "Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin.", "Wire up MCP tools: sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup.", "Implement lead qualification, customer vetting, quote generation, and approval workflows.", "Ensure Morgan responds within 10 seconds for simple queries.", "Support 500+ concurrent Signal connections.", "Add observability hooks for Prometheus, Loki, Grafana.", "Document agent skills and tool usage."]}

## Acceptance Criteria
Morgan responds to Signal, voice, and web chat within 10 seconds for simple queries; workflows (lead qualification, quote, vetting, invoice) complete end-to-end; MCP tools are invoked and return correct data; supports 500+ concurrent Signal connections in load test.

## Subtasks
- Configure OpenClaw agent runtime with MCP tool-server connectivity: Set up the OpenClaw agent runtime environment, configure the MCP tool-server connection, register all MCP tool definitions (sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_vet_customer, sigma1_score_lead, sigma1_create_invoice, sigma1_finance_report, sigma1_social_curate, sigma1_social_publish, sigma1_equipment_lookup), and verify the agent can discover and invoke tools against backend service endpoints.
- Integrate Signal-CLI for inbound/outbound messaging: Implement the Signal messaging channel for Morgan, connecting to Signal-CLI for receiving and sending messages, handling message parsing, conversation threading, and session management for concurrent users.
- Integrate ElevenLabs for voice interactions: Implement the voice channel for Morgan using ElevenLabs text-to-speech and speech-to-text APIs, enabling voice-based customer interactions with natural-sounding responses.
- Implement web chat widget channel: Build the web chat interface channel that Morgan uses to communicate with website visitors, implementing WebSocket-based real-time messaging, session management, and a lightweight embeddable widget API.
- Implement sales skills: lead qualification, quote generation, and upsell: Implement Morgan's sales-focused skills (sales-qual, quote-gen, upsell) that wire to sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_score_lead, and sigma1_equipment_lookup MCP tools to handle the full sales conversation flow.
- Implement operations skills: customer vetting, finance, and RMS workflows: Implement Morgan's operations-focused skills (customer-vet, finance, rms-*) that wire to sigma1_vet_customer, sigma1_create_invoice, sigma1_finance_report, and RMS-related MCP tools to handle customer verification, invoicing, financial reporting, and rental management workflows.
- Implement social media and admin skills with approval workflows: Implement Morgan's social media skills (social-media) and admin skill using sigma1_social_curate, sigma1_social_publish MCP tools, including content curation, human approval workflows, and administrative command handling.
- Performance optimization and load testing for 500+ concurrent connections: Optimize Morgan's response pipeline to achieve <10s response time for simple queries and validate support for 500+ concurrent Signal connections through load testing and bottleneck identification.
- Add observability hooks for Prometheus, Loki, and Grafana: Instrument the Morgan agent with Prometheus metrics, structured logging for Loki, and create Grafana dashboards for monitoring agent health, performance, and usage patterns.
- Document Morgan agent skills, MCP tool usage, and conversation flows: Create comprehensive documentation covering Morgan's skills, MCP tool mappings, conversation flow diagrams, configuration reference, and operational runbook.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.