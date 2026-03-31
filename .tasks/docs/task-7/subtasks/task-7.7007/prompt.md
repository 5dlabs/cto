Implement subtask 7007: Configure agent skills and persona routing

## Objective
Define and configure all Morgan agent skills (sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin) so the agent can route conversations to the appropriate skill with the correct tool bindings and system prompts.

## Steps
1. Define the `sales-qual` skill: system prompt focused on lead qualification, bound to score_lead and catalog_search tools.
2. Define the `customer-vet` skill: system prompt for customer vetting workflows, bound to vet_customer tool.
3. Define the `quote-gen` skill: system prompt for generating rental quotes, bound to generate_quote, catalog_search, check_availability tools.
4. Define the `upsell` skill: system prompt for suggesting additional equipment/services, bound to catalog_search and equipment_lookup tools.
5. Define the `finance` skill: system prompt for financial queries, bound to create_invoice, finance_report tools.
6. Define the `social-media` skill: system prompt for social content management, bound to social_curate, social_publish tools.
7. Define the `rms` skill(s): system prompt for reservation management, bound to all rms-* tools.
8. Define the `admin` skill: system prompt for administrative queries, bound to all tools with elevated context.
9. Configure skill routing logic: the agent should detect user intent and activate the appropriate skill, or allow explicit skill switching.
10. Test that each skill activates the correct tools and persona when triggered.

## Validation
Send test prompts that target each skill (e.g., 'I need a quote for an excavator' should activate quote-gen, 'Check my reservation status' should activate rms). Verify the agent uses only the tools bound to the active skill. Verify smooth transitions between skills within a single conversation. Confirm admin skill requires appropriate authorization context.