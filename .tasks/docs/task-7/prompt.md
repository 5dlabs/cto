<identity>
You are angie, the OpenClaw/MCP implementation agent. You own task 7 end-to-end.
</identity>

<context>
<task_overview>
Task 7: Configure Morgan AI Agent (Angie - OpenClaw/MCP)
Configure and deploy the Morgan AI agent on OpenClaw with all 12 MCP tools (10 business tools + 2 GDPR tools) pointing to the Rust, Go, and Node.js backend services. Define agent skills, system prompt, Signal-CLI integration, ElevenLabs voice config, and web chat widget endpoint. Morgan becomes the single intelligent interface for all customer interactions.
Priority: high
Dependencies: 2, 3, 4, 5, 6
</task_overview>
</context>

<implementation_plan>
1. Create OpenClaw agent configuration at agents/morgan/agent.yaml. Set agent_id: morgan, model: openai-api/gpt-4o (use available model, update when gpt-5.4-pro available), namespace: openclaw.
2. System prompt (agents/morgan/system-prompt.md): Morgan is the AI agent for Sigma-1/Perception Events, a lighting and visual production company. Persona: professional, efficient, friendly. Core responsibilities listed with decision trees for lead qualification, quote generation, vetting, invoicing, social media approval.
3. MCP tool-server configuration (agents/morgan/tools.yaml). Define tools pointing to in-cluster service URLs:
   - sigma1_catalog_search: GET http://equipment-catalog-svc.sigma1.svc.cluster.local:8080/api/v1/catalog/products?search={query}
   - sigma1_check_availability: GET http://equipment-catalog-svc.sigma1.svc.cluster.local:8080/api/v1/catalog/products/{product_id}/availability?from={from}&to={to}
   - sigma1_generate_quote: POST http://rms-svc.sigma1.svc.cluster.local:8080/api/v1/opportunities
   - sigma1_vet_customer: POST http://customer-vetting-svc.sigma1.svc.cluster.local:8080/api/v1/vetting/run
   - sigma1_score_lead: GET http://rms-svc.sigma1.svc.cluster.local:8080/api/v1/opportunities/{id}/score
   - sigma1_create_invoice: POST http://finance-svc.sigma1.svc.cluster.local:8080/api/v1/invoices
   - sigma1_finance_report: GET http://finance-svc.sigma1.svc.cluster.local:8080/api/v1/finance/reports/revenue?period={period}
   - sigma1_social_curate: POST http://social-engine-svc.sigma1.svc.cluster.local:8080/api/v1/social/upload
   - sigma1_social_publish: POST http://social-engine-svc.sigma1.svc.cluster.local:8080/api/v1/social/drafts/{draft_id}/publish
   - sigma1_equipment_lookup: GET http://equipment-catalog-svc.sigma1.svc.cluster.local:8080/api/v1/equipment-api/catalog
   - sigma1_gdpr_export: orchestrates GET /internal/gdpr/export/:customer_id on all 5 services, aggregates results
   - sigma1_gdpr_delete: orchestrates DELETE /internal/gdpr/delete/:customer_id on all 5 services, logs to audit schema
4. Skills configuration (agents/morgan/skills/): Define skill manifests for sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-checkout, rms-checkin, admin. Each skill references relevant MCP tools and includes decision logic in the skill prompt.
5. Signal-CLI integration: configure OpenClaw Signal adapter to connect to http://signal-cli-svc.signal.svc.cluster.local:7583 JSON-RPC. Set phone_number from TWILIO_PHONE_NUMBER secret. Configure receive_loop: true.
6. ElevenLabs voice: configure voice_id from ELEVENLABS_API_KEY secret. Set voice model to eleven_turbo_v2 for low latency. Twilio SIP trunk configuration: Morgan receives inbound calls via Twilio webhook POST /voice/inbound, responds with TwiML, bridges to ElevenLabs.
7. Web chat: configure OpenClaw web chat adapter on port 3000. Expose as ClusterIP service morgan-chat-svc:3000. Web chat widget JS snippet to be embedded by Blaze in Task 8.
8. Kubernetes: Morgan Deployment in openclaw namespace, 1 replica, PVC morgan-workspace (10Gi) at /workspace. envFrom sigma1-infra-endpoints + all API key secrets. Liveness probe: GET /health/live on OpenClaw admin port. Cloudflare Tunnel annotation for external Signal webhook access.
9. Test all MCP tool connections by running Morgan in interactive CLI mode and issuing one request per tool. Verify tool responses are correctly parsed.
</implementation_plan>

<acceptance_criteria>
1. kubectl exec into Morgan pod and run morgan tool-test sigma1_catalog_search query=lights — response contains product array with name and day_rate fields. 2. sigma1_check_availability for a known product_id with valid date range returns availability object with quantity_available field. 3. sigma1_generate_quote with customer_id and line items creates opportunity in RMS — verify via GET /api/v1/opportunities/:id returns status=pending. 4. sigma1_vet_customer with valid org data returns 202 and polling returns final_score within 60s. 5. sigma1_create_invoice creates invoice in Finance — verify via GET /api/v1/invoices/:id returns total_cents > 0. 6. sigma1_gdpr_export returns aggregated JSON containing data from at least 3 services. 7. Signal-CLI connectivity: send test message to Morgan's phone number via Signal; verify Morgan pod logs show incoming message received. 8. Web chat endpoint: POST http://morgan-chat-svc:3000/chat with {message: 'hello'} returns {reply: <non-empty string>} within 10 seconds.
</acceptance_criteria>

<subtasks>
- Create OpenClaw agent.yaml and system-prompt.md for Morgan: Author the core OpenClaw agent configuration file at agents/morgan/agent.yaml and the system prompt at agents/morgan/system-prompt.md defining Morgan's persona, responsibilities, and decision trees.
- Define all 9 skill manifests in agents/morgan/skills/: Create individual YAML skill manifest files for each of the 9 Morgan skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-checkout, rms-checkin, and admin.
- Configure MCP tools.yaml for 10 business tools: Author agents/morgan/tools.yaml defining all 10 business MCP tools with correct in-cluster service URLs, HTTP methods, parameter schemas, and response field descriptions.
- Configure GDPR orchestration tools (sigma1_gdpr_export and sigma1_gdpr_delete): Define the two GDPR MCP tools in a dedicated agents/morgan/tools-gdpr.yaml that fan out to all 5 backend services and aggregate or delete data, with audit logging for the delete operation.
- Configure Signal-CLI adapter in OpenClaw for Morgan: Set up the OpenClaw Signal messaging adapter pointing to the signal-cli-svc JSON-RPC endpoint, with phone number from secret and receive loop enabled.
- Configure ElevenLabs voice adapter and Twilio TwiML webhook in OpenClaw: Set up ElevenLabs TTS voice configuration using the eleven_turbo_v2 model and define the Twilio inbound call TwiML webhook handler so Morgan can receive and respond to phone calls.
- Configure OpenClaw web chat adapter and expose morgan-chat-svc: Enable the OpenClaw web chat adapter on port 3000, serving the chat API endpoint and the embeddable widget JavaScript, and create the ClusterIP Kubernetes Service.
- Create Morgan Kubernetes Deployment with PVC, secrets, and Cloudflare Tunnel annotation: Author the Kubernetes Deployment manifest for Morgan in the openclaw namespace, including the 10Gi PVC, envFrom references to all required secrets and ConfigMaps, liveness probe, and Cloudflare Tunnel annotation for the Signal webhook.
- Execute end-to-end MCP tool connectivity tests for all 12 tools: Run one interactive CLI test request per MCP tool to verify every tool correctly reaches its backend service, receives a valid response, and is correctly parsed by Morgan.
- Verify Signal message receipt and web chat endpoint via integration test: Send a real Signal test message to Morgan's number and validate round-trip receipt in pod logs. Also validate the web chat /chat endpoint returns a coherent reply.
</subtasks>