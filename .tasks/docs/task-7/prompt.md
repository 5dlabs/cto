Implement task 7: Implement Morgan AI Agent (Angie - OpenClaw/MCP)

## Goal
Configure and deploy the Morgan AI agent as the central intelligence orchestrating all customer interactions via Signal, voice (ElevenLabs/Twilio), and web chat. Morgan uses MCP tools to call all backend services and implements skills for lead qualification, vetting, quoting, upselling, finance, social media, and admin workflows.

## Task Context
- Agent owner: angie
- Stack: OpenClaw/MCP
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
1. Create OpenClaw agent configuration for Morgan:
   - Agent ID: `morgan`
   - Model: `openai-api/gpt-5.4-pro` (or latest available; fallback to gpt-4o)
   - System prompt: Define Morgan's persona — professional, knowledgeable about lighting/visual production equipment, represents Sigma-1/Perception Events. Tone: helpful but efficient, knows when to escalate to Mike.
   - Workspace: PVC `morgan-workspace` for conversation state and file handling.
2. MCP Tool Server configuration — define 10 tools with REST/gRPC endpoint mappings:
   - `sigma1_catalog_search` → GET `/api/v1/catalog/products?q={query}&category={cat}` on equipment-catalog service
   - `sigma1_check_availability` → GET `/api/v1/catalog/products/{id}/availability?from={from}&to={to}`
   - `sigma1_generate_quote` → POST `/api/v1/opportunities` on RMS service (via grpc-gateway REST)
   - `sigma1_vet_customer` → POST `/api/v1/vetting/run` on vetting service
   - `sigma1_score_lead` → POST `/api/v1/opportunities/{id}/score` (ScoreLead via RMS REST gateway)
   - `sigma1_create_invoice` → POST `/api/v1/invoices` on finance service
   - `sigma1_finance_report` → GET `/api/v1/finance/reports/{type}?period={period}`
   - `sigma1_social_curate` → POST `/api/v1/social/upload` on social engine
   - `sigma1_social_publish` → POST `/api/v1/social/drafts/{id}/approve`
   - `sigma1_equipment_lookup` → GET `/api/v1/equipment-api/catalog` (machine-readable endpoint)
   Each tool includes JSON Schema for parameters and expected response format.
3. Skills configuration:
   - `sales-qual`: Multi-turn conversation flow — identify event type, date, venue, budget, equipment needs. Use catalog_search and check_availability tools. End with quote generation or escalation.
   - `customer-vet`: Triggered when new customer detected. Calls sigma1_vet_customer, waits for result, interprets GREEN/YELLOW/RED and adjusts behavior (RED: require deposit, escalate to Mike).
   - `quote-gen`: Assemble line items from catalog, calculate totals, create opportunity via RMS. Send quote summary back to customer.
   - `upsell`: After quote, suggest insurance, delivery services, additional equipment bundles based on event type.
   - `finance`: Handle invoice queries, generate invoices from confirmed projects, report on revenue/aging.
   - `social-media`: Receive event photos, trigger curation, send drafts to Mike for approval via Signal.
   - `rms-*`: Project status queries, checkout/checkin coordination, crew scheduling requests.
   - `admin`: Calendar queries (via RMS Google Calendar), email drafting, document references.
4. Signal integration:
   - Signal-CLI sidecar (configured in Task 1) exposes REST API at localhost:8080.
   - Morgan listens for incoming Signal messages via Signal-CLI REST API polling or webhook.
   - Outbound messages sent via Signal-CLI REST API `POST /v2/send`.
   - Support: text messages, photo receiving (for social pipeline), photo sending (for quote previews).
   - Handle group messages vs direct messages.
5. Voice integration:
   - ElevenLabs Conversational AI for voice synthesis/recognition.
   - Twilio SIP trunk for PSTN connectivity.
   - Voice calls routed: Twilio → ElevenLabs → Morgan (text) → ElevenLabs (speech) → Twilio → caller.
   - Configure Twilio webhook to forward calls to ElevenLabs endpoint.
6. Web chat:
   - Morgan exposes WebSocket endpoint for real-time chat from the Next.js frontend.
   - Protocol: JSON messages with `{type, content, metadata}` structure.
   - Support conversation history persistence in morgan-workspace PVC.
7. Performance:
   - Simple queries (catalog search, availability) must respond < 10 seconds end-to-end.
   - Implement tool call parallelization where tools are independent.
8. Kubernetes Deployment:
   - Namespace: `openclaw` (existing OpenClaw namespace)
   - Signal-CLI sidecar container with resource limits (512Mi, 500m CPU), restart policy Always.
   - Morgan agent container with workspace PVC mount.
   - Service API keys for all backend services from `sigma1-service-api-keys` secret.
   - ElevenLabs API key, Twilio credentials from secrets.
   - Cloudflare Tunnel service for external Signal/voice access.
9. Account rotation strategy for Signal-CLI (per open question #1): Configure secondary Signal number as fallback. Monitor for account ban signals (HTTP 403/rate limiting from Signal servers). Alert via Grafana if primary account becomes unavailable.

## Acceptance Criteria
1. MCP tool connectivity test: for each of the 10 tools, invoke with valid parameters and verify successful response from the corresponding backend service (requires all 5 backend services running). 2. Signal round-trip test: send a test message via Signal-CLI REST API, verify Morgan receives it, processes it, and sends a response back within 10 seconds. 3. Skill test — sales-qual: simulate a multi-turn conversation ("I need lighting for a wedding on June 15"), verify Morgan calls catalog_search, check_availability, and offers to generate a quote. 4. Skill test — customer-vet: trigger vetting for a test org, verify Morgan calls sigma1_vet_customer and correctly interprets GREEN/YELLOW/RED result. 5. Voice integration test: make a test call via Twilio, verify ElevenLabs processes speech and Morgan responds with relevant content. 6. Web chat WebSocket test: connect via WebSocket, send message, verify JSON response with correct structure within 10 seconds. 7. Performance test: 10 concurrent simple queries (catalog search), verify all respond within 10 seconds. 8. Signal-CLI health monitoring: verify liveness probe detects unhealthy Signal-CLI sidecar and pod restarts.

## Subtasks
- Configure OpenClaw agent definition for Morgan (persona, model, system prompt): Create the core OpenClaw agent configuration file defining Morgan's identity, model binding, system prompt with persona instructions, and workspace PVC reference. This is the foundational agent definition that all skills and tools attach to.
- Configure MCP Tool Server with all 10 tool definitions and JSON Schema mappings: Define the MCP Tool Server configuration mapping all 10 backend service tools with their REST/gRPC endpoint URLs, parameter JSON Schemas, response schemas, and authentication headers. Each tool must be individually testable.
- Implement sales-qual skill (multi-turn lead qualification conversation flow): Build the sales-qual skill that drives multi-turn conversations to qualify leads by identifying event type, date, venue, budget, and equipment needs, using catalog_search and check_availability tools, ending with quote generation or escalation.
- Implement customer-vet skill (vetting integration with GREEN/YELLOW/RED interpretation): Build the customer-vet skill that triggers automatically for new customers, calls the vetting service, interprets the GREEN/YELLOW/RED result, and adjusts Morgan's behavior accordingly (deposit requirements, escalation to Mike).
- Implement quote-gen and upsell skills: Build the quote-gen skill that assembles line items from catalog data, calculates totals, creates an opportunity via RMS, and sends a quote summary. Build the upsell skill that suggests additional services/equipment after quote generation.
- Implement finance, social-media, rms, and admin skills: Build the remaining skills: finance (invoice creation, reports), social-media (photo curation pipeline with Mike approval), rms (project status, checkout/checkin, crew scheduling), and admin (calendar, email, documents).
- Implement Signal messaging integration (inbound/outbound via Signal-CLI REST API): Build the Signal channel adapter that connects Morgan to the Signal-CLI sidecar REST API for receiving and sending messages, handling photos, and distinguishing group vs direct messages.
- Implement voice integration pipeline (ElevenLabs Conversational AI + Twilio SIP): Configure the voice call pipeline: Twilio receives PSTN calls, forwards to ElevenLabs for speech-to-text, Morgan processes the text and generates a response, ElevenLabs synthesizes speech, and Twilio plays it back to the caller.
- Implement web chat WebSocket endpoint with conversation persistence: Build the WebSocket endpoint that the Next.js frontend connects to for real-time chat with Morgan, including JSON message protocol and conversation history persistence on the workspace PVC.
- Create Kubernetes deployment manifests for Morgan agent with Signal-CLI sidecar: Write the Kubernetes Deployment, Service, PVC, and related manifests to deploy Morgan in the openclaw namespace with the Signal-CLI sidecar container, workspace PVC, API key secrets, and health probes.
- Implement Signal account rotation and health monitoring: Configure secondary Signal number as failover, implement monitoring for account ban signals (HTTP 403/rate limiting), and set up Grafana alerts for Signal-CLI health degradation.
- Implement tool call parallelization and end-to-end performance validation: Configure MCP tool call parallelization for independent tool invocations and validate the <10 second end-to-end response time target for simple queries across all channels.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.