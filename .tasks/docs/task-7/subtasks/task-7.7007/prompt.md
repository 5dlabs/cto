Implement subtask 7007: Configure all Morgan agent skills and conversation routing

## Objective
Configure the Morgan agent's skill definitions for sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, and admin. Set up intent routing so the agent selects the correct skill and tools based on conversation context.

## Steps
1. Define the `sales-qual` skill: system prompt, tool bindings (catalog_search, availability_check), conversation flow for qualifying leads.
2. Define the `customer-vet` skill: prompt, tool bindings (customer_vet, customer_score), flow for vetting customers.
3. Define the `quote-gen` skill: prompt, tool bindings (quote_generate, quote_submit, availability_check), multi-step quote building flow.
4. Define the `upsell` skill: prompt, tool bindings (catalog_search), logic for suggesting related equipment.
5. Define the `finance` skill: prompt, tool bindings (invoice_create, invoice_lookup, finance_report), admin-facing finance queries.
6. Define the `social-media` skill: prompt, tool bindings (social_curate, social_publish), content workflow.
7. Define the `rms-*` skills: prompt, tool bindings (equipment_rms_lookup, equipment_rms_status), equipment management queries.
8. Define the `admin` skill: prompt, elevated tool access, admin-only operations.
9. Configure intent classification / routing logic so inbound messages are dispatched to the correct skill.
10. Set fallback behavior for unrecognized intents.

## Validation
Send test messages exercising each skill: a sales inquiry triggers sales-qual; a vetting request triggers customer-vet; a quote request triggers quote-gen; an upsell scenario triggers upsell; a finance question triggers finance skill; a social media request triggers social-media; an equipment status query triggers rms-*; an admin command triggers admin skill. Verify correct tool invocations in logs for each.