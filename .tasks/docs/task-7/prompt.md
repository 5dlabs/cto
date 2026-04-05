Implement task 7: Deploy Morgan AI Agent with MCP Tools (Angie - OpenClaw/MCP)

## Goal
Deploy and configure the Morgan AI agent using OpenClaw runtime with MCP tool-server connecting to all backend services. Includes Signal-CLI sidecar for messaging, ElevenLabs voice integration, Twilio phone number routing, web chat WebSocket endpoint, and all 11 MCP tools for backend service orchestration.

## Task Context
- Agent owner: angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
1. Configure OpenClaw agent manifest for Morgan:
   - Agent ID: `morgan`
   - Model: `openai-api/gpt-5.4-pro` (or latest available, configurable via env)
   - System prompt defining Morgan's persona: professional, knowledgeable about lighting/visual production, Perception Events brand voice, conversational but efficient
   - Skills configuration: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin
2. Implement MCP Tool Server (HTTP-based tool definitions):
   - `sigma1_catalog_search` — GET /api/v1/catalog/products with query params → Equipment Catalog
   - `sigma1_check_availability` — GET /api/v1/catalog/products/:id/availability?from=&to= → Equipment Catalog
   - `sigma1_generate_quote` — POST /api/v1/opportunities → RMS (create opportunity with line items)
   - `sigma1_vet_customer` — POST /api/v1/vetting/run → Customer Vetting
   - `sigma1_score_lead` — RPC ScoreLead via RMS REST gateway /api/v1/opportunities/:id (reads lead_score)
   - `sigma1_create_invoice` — POST /api/v1/invoices → Finance
   - `sigma1_finance_report` — GET /api/v1/finance/reports/* → Finance
   - `sigma1_social_curate` — POST /api/v1/social/upload → Social Engine
   - `sigma1_social_publish` — POST /api/v1/social/drafts/:id/publish → Social Engine
   - `sigma1_equipment_lookup` — GET /api/v1/equipment-api/catalog → Equipment Catalog (machine-readable)
   - Each tool definition includes: name, description, input JSON schema, output schema, HTTP method/URL template
   - All tool calls include Authorization header with morgan-agent JWT service token
3. Signal-CLI sidecar configuration:
   - Deploy signal-cli-rest-api as sidecar container in Morgan pod
   - Mount persistent volume for Signal-CLI data directory (device registration, keys)
   - Configure webhook: signal-cli → Morgan agent HTTP endpoint for incoming messages
   - Outbound: Morgan agent → signal-cli REST API for sending messages
   - Handle message types: text, image (photo attachments from events), location
4. ElevenLabs voice integration:
   - Configure ElevenLabs Conversational AI agent with Morgan's voice profile
   - WebSocket connection for real-time voice streaming
   - Twilio SIP trunk → ElevenLabs → Morgan agent for phone call routing
   - Voice-to-text and text-to-voice pipeline
5. Twilio configuration:
   - Provision phone number (or configure existing)
   - SIP trunk pointing to ElevenLabs endpoint
   - Fallback: Twilio webhook → Morgan text endpoint if voice unavailable
6. Web chat WebSocket endpoint:
   - `/ws/chat` — WebSocket endpoint for real-time web chat
   - Session management: create session on connect, store in Valkey with 24h TTL
   - Session continuity: accept session token from client, resume conversation
   - Message format: JSON { type: 'user'|'agent', content: string, timestamp: ISO8601 }
   - Streaming responses: send agent response tokens as they arrive from LLM
7. Conversation state management:
   - Store conversation history in Valkey (keyed by session_id/signal_sender)
   - Context window management: keep last 50 messages + system prompt
   - Skill routing: based on intent detection, activate appropriate skill
8. Lead qualification workflow (sales-qual skill):
   - Detect inquiry intent → ask qualifying questions (event type, date, budget range, venue)
   - Check equipment availability via sigma1_check_availability
   - Trigger vetting via sigma1_vet_customer
   - Generate quote via sigma1_generate_quote
   - Send summary to Mike via Signal for approval
9. Social media approval workflow (social-media skill):
   - When drafts are created, send preview to Mike via Signal
   - Parse Mike's response (approve/reject) → call approve/reject endpoint
10. Kubernetes Deployment:
    - Namespace: `sigma1` (or `openclaw` per PRD)
    - Pod with 2 containers: morgan (openclaw-agent), signal-cli (sidecar)
    - PVC: morgan-workspace (10Gi) for agent workspace + signal-cli data
    - envFrom: sigma1-infra-endpoints + sigma1-external-secrets
    - Resource limits: morgan 1Gi memory/500m CPU, signal-cli 512Mi/250m
11. Morgan-specific canary deployment strategy:
    - Blue-green deployment with conversation state in Valkey (survives pod restart)
    - Health check verifies: LLM connectivity, Signal-CLI registered, all MCP tools reachable
    - Rollback preserves Valkey conversation state

## Acceptance Criteria
1. MCP tool connectivity test: each of the 11 MCP tools responds successfully when called with valid test parameters (mock or against running services). 2. Signal integration test: send test message via signal-cli REST API → verify Morgan receives it, processes intent, and sends response back within 10 seconds. 3. Web chat WebSocket test: connect to /ws/chat, send user message, receive streaming agent response within 10 seconds, verify JSON message format. 4. Session continuity test: connect with session token, disconnect, reconnect with same token → verify conversation history preserved. 5. Lead qualification e2e test: simulate Signal conversation with event inquiry → verify Morgan asks qualifying questions, checks availability (via MCP tool), generates quote → verify opportunity created in RMS. 6. Tool authorization test: all MCP tool calls include valid morgan-agent JWT, backend services accept and log the service identity. 7. Health readiness test: /health/ready returns 200 only when LLM endpoint, Signal-CLI, and at least 3 critical MCP tools (catalog_search, generate_quote, create_invoice) are reachable. 8. Conversation context test: send 10 messages in sequence, verify Morgan's responses demonstrate awareness of conversation history.

## Subtasks
- Configure OpenClaw agent manifest with Morgan persona and skills: Create the OpenClaw agent manifest file defining Morgan's identity, LLM model configuration, system prompt with Perception Events brand voice, and skill routing configuration for all agent capabilities (sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms, admin).
- Implement MCP Tool Server — Equipment Catalog tools (catalog_search, check_availability, equipment_lookup): Define and implement MCP tool definitions for the three Equipment Catalog service tools: sigma1_catalog_search, sigma1_check_availability, and sigma1_equipment_lookup, including JSON input/output schemas, HTTP method/URL templates, and authorization header injection.
- Implement MCP Tool Server — RMS tools (generate_quote, score_lead): Define and implement MCP tool definitions for the two RMS (Rental Management System) tools: sigma1_generate_quote and sigma1_score_lead, with JSON schemas and HTTP mappings.
- Implement MCP Tool Server — Customer Vetting tool (vet_customer): Define and implement the MCP tool definition for sigma1_vet_customer, mapping to the Customer Vetting service POST endpoint.
- Implement MCP Tool Server — Finance tools (create_invoice, finance_report): Define and implement MCP tool definitions for the two Finance service tools: sigma1_create_invoice and sigma1_finance_report.
- Implement MCP Tool Server — Social Engine tools (social_curate, social_publish): Define and implement MCP tool definitions for the two Social Engine tools: sigma1_social_curate and sigma1_social_publish.
- Implement MCP Tool Server HTTP client, JWT auth, and tool registry: Build the shared MCP tool server infrastructure: HTTP client wrapper with JWT injection, tool registry that loads all 11 tool definitions, error handling middleware, and request/response logging.
- Deploy Signal-CLI sidecar container with persistent volume and webhook routing: Configure and deploy the signal-cli-rest-api as a sidecar container in the Morgan pod, including persistent volume for device registration data, inbound webhook configuration, and outbound message sending integration.
- Implement ElevenLabs voice integration with WebSocket streaming: Configure the ElevenLabs Conversational AI agent with Morgan's voice profile and implement the WebSocket-based real-time voice streaming pipeline for phone call interactions.
- Configure Twilio SIP trunk and phone number routing to ElevenLabs: Set up Twilio phone number provisioning, SIP trunk configuration pointing to ElevenLabs voice endpoint, and fallback webhook routing to Morgan's text endpoint.
- Implement web chat WebSocket endpoint with session management and streaming responses: Build the /ws/chat WebSocket endpoint for real-time web chat, including Valkey-backed session management with 24h TTL, session continuity via tokens, and streaming LLM response tokens to the client.
- Implement conversation state management and context window handling in Valkey: Build the shared conversation state management layer that stores conversation history in Valkey across all channels (Signal, web chat, voice), manages the 50-message context window, and provides skill routing based on intent detection.
- Implement lead qualification workflow (sales-qual skill): Build the sales-qual skill workflow: detect inquiry intent, guide the customer through qualifying questions (event type, date, budget, venue), check equipment availability, trigger customer vetting, generate a quote, and send approval summary to Mike via Signal.
- Implement social media approval workflow (social-media skill): Build the social-media skill workflow: when social media drafts are created, send previews to Mike via Signal, parse Mike's approve/reject responses, and call the corresponding Social Engine endpoints.
- Create Kubernetes deployment manifest for Morgan multi-container pod: Write the Kubernetes Deployment, PVC, Service, and ConfigMap manifests for the Morgan agent pod with OpenClaw agent container and Signal-CLI sidecar, including resource limits, environment variable injection from sigma1-infra-endpoints and sigma1-external-secrets.
- Implement health check endpoint with LLM, Signal-CLI, and MCP tool connectivity verification: Build the /health/ready and /health/live endpoints that verify connectivity to the LLM endpoint, Signal-CLI sidecar registration status, and reachability of critical MCP tools.
- Implement blue-green deployment strategy with Valkey-persisted conversation state: Configure the Morgan agent deployment for blue-green rollouts where conversation state in Valkey survives pod restarts and version transitions, ensuring zero conversation loss during deployments.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.