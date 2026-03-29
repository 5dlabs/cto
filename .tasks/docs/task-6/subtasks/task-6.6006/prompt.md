Implement subtask 6006: Develop core AI skills and configure Cloudflare Tunnel

## Objective
Implement the `sales-qual` and `quote-gen` AI skills, and configure Cloudflare Tunnel for secure external access to the Morgan agent's web chat interface and webhooks.

## Steps
1. Define the `sales-qual` skill logic to interpret user requests for lead qualification and orchestrate calls to `sigma1_vet_customer` and `sigma1_score_lead`.2. Define the `quote-gen` skill logic to interpret user requests for quotes and orchestrate calls to `sigma1_catalog_search`, `sigma1_check_availability`, and `sigma1_generate_quote`.3. Install and configure `cloudflared` within the Kubernetes cluster.4. Create a Cloudflare Tunnel and configure it to expose the Morgan agent's service (web chat, Signal webhook, Twilio webhook) to the internet via a Cloudflare DNS record.

## Validation
1. Send a natural language query like 'Can you qualify a new lead for me?' and verify Morgan triggers the correct vetting tools and returns a lead score.2. Ask Morgan to 'Generate a quote for 5 projectors for next week' and verify it uses the catalog and quote generation tools, providing a quote ID.3. Verify Cloudflare Tunnel is active and the web chat interface is accessible externally.