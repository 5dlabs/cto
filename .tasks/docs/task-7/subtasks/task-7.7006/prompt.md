Implement subtask 7006: Implement skills: sales-qual, customer-vet, quote-gen, upsell

## Objective
Implement the sales-oriented skill set for Morgan: lead qualification (sales-qual), customer vetting orchestration (customer-vet), quote generation (quote-gen), and upsell recommendation (upsell).

## Steps
1. **sales-qual skill**: Define conversation flow for qualifying inbound leads. Extract: company name, project type, equipment needs, timeline, budget range. Use sigma1_catalog_search to validate equipment availability. Score lead and route accordingly.
2. **customer-vet skill**: Orchestrate customer vetting by calling sigma1_customer_vet with collected information. Parse vetting results and communicate status to the customer.
3. **quote-gen skill**: Gather quote parameters (equipment list, duration, delivery location). Call sigma1_create_quote. Format and present the quote to the customer with line items and totals.
4. **upsell skill**: Based on the current equipment selection, recommend complementary items using catalog data. Present upsell suggestions naturally within the conversation.
5. Each skill should be defined as a structured prompt/instruction set that Morgan can activate based on conversation context.
6. Implement skill routing logic: detect user intent and activate the appropriate skill.

## Validation
Simulate a lead qualification conversation; verify all required fields are collected and lead is scored. Trigger customer-vet skill; verify sigma1_customer_vet is called. Request a quote; verify sigma1_create_quote is called and quote is presented. Verify upsell suggestions appear when relevant equipment is selected.