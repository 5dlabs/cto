Implement subtask 7001: Configure OpenClaw agent manifest with Morgan persona and skills

## Objective
Create the OpenClaw agent manifest file defining Morgan's identity, LLM model configuration, system prompt with Perception Events brand voice, and skill routing configuration for all agent capabilities (sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms, admin).

## Steps
1. Create the OpenClaw agent manifest file (e.g., `morgan-agent.yaml` or `agent.json` depending on OpenClaw format).
2. Set Agent ID to `morgan`.
3. Configure model endpoint: `openai-api/gpt-5.4-pro` via environment variable `MORGAN_LLM_MODEL` so it's easily swappable.
4. Write the system prompt:
   - Professional tone, knowledgeable about lighting, visual production, AV equipment
   - Perception Events brand voice: conversational but efficient, not overly formal
   - Include instructions for skill activation triggers (e.g., 'when user asks about equipment availability, use the sales-qual skill')
   - Include guardrails: don't discuss competitors, don't make promises about pricing without generating a quote, escalate to Mike for custom requests
5. Define skills configuration array mapping skill names to their tool sets:
   - `sales-qual`: catalog_search, check_availability, vet_customer, generate_quote, score_lead
   - `finance`: create_invoice, finance_report
   - `social-media`: social_curate, social_publish
   - `admin`: equipment_lookup
6. Configure LLM parameters: temperature (0.7 for conversational), max_tokens, and any stop sequences.
7. Set up environment variable references for API keys (OPENAI_API_KEY) and endpoint URLs from sigma1-infra-endpoints ConfigMap.

## Validation
Validate manifest parses without errors using OpenClaw CLI or schema validator. Verify environment variable placeholders resolve correctly. Send a test prompt to the configured LLM endpoint and confirm a coherent response with Morgan's persona characteristics.