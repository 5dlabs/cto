## Implement Morgan AI Agent (Angie - OpenClaw/MCP)

### Objective
Deploy and configure the Morgan AI agent to handle all customer interactions via Signal, voice, and web chat, orchestrating backend services through MCP tools.

### Ownership
- Agent: angie
- Stack: OpenClaw/MCP
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6

### Implementation Details
{"steps": ["Deploy OpenClaw agent using Kubernetes deployment, referencing 'sigma1-infra-endpoints' for all service URLs and credentials", "Configure Signal-CLI, ElevenLabs, and Twilio integrations for messaging and voice", "Register MCP tools for all backend services (catalog, RMS, finance, vetting, social)", "Implement skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin", "Set up web chat widget endpoint for website integration", "Ensure Morgan responds within 10 seconds for simple queries and orchestrates quote-to-invoice workflows", "Write integration tests for all major flows (lead qualification, quote, vetting, invoice, social approval)"]}

### Subtasks
- [ ] Deploy OpenClaw agent on Kubernetes with sigma1-infra-endpoints configuration: Create the Kubernetes Deployment, Service, and ConfigMap references for the OpenClaw Morgan agent, pulling all service URLs and credentials from the sigma1-infra-endpoints ConfigMap and associated Secrets.
- [ ] Integrate Signal-CLI for bidirectional messaging: Configure Signal-CLI as a sidecar or companion pod to enable Morgan to send and receive Signal messages from customers.
- [ ] Integrate ElevenLabs voice synthesis and Twilio phone channel: Set up ElevenLabs for text-to-speech and Twilio for inbound/outbound phone calls so Morgan can handle voice interactions.
- [ ] Register MCP tools for Equipment Catalog and RMS backend services: Define and register MCP tool definitions for the Equipment Catalog API and the Rental Management System (RMS) API, mapping request/response schemas.
- [ ] Register MCP tools for Finance, Vetting, and Social Media backend services: Define and register MCP tool definitions for the Finance/Invoicing, Customer Vetting, and Social Media Engine APIs.
- [ ] Implement sales flow skills: sales-qual, customer-vet, quote-gen, and upsell: Implement the core sales-oriented agent skills that handle the lead qualification → vetting → quote generation → upsell pipeline.
- [ ] Implement finance, social-media, RMS, and admin skills: Implement the remaining agent skills for finance operations, social media management, rental management, and administrative tasks.
- [ ] Implement web chat widget WebSocket endpoint: Create the WebSocket/HTTP endpoint that the website frontend will use to embed Morgan as a real-time chat widget.
- [ ] Write end-to-end integration tests for complete workflows: Create integration tests covering the full lead → vet → quote → invoice flow and other major multi-skill workflows across all communication channels.