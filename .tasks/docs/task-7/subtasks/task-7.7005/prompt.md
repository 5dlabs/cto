Implement subtask 7005: Implement sales and customer skills: sales-qual, customer-vet, quote-gen, upsell

## Objective
Develop the core customer-facing skills that handle lead qualification, customer vetting, quote generation, and upselling, wired to the corresponding MCP tools.

## Steps
1. Implement the sales-qual skill: define the conversation flow for qualifying leads (budget, timeline, equipment needs), invoke sigma1_score_lead tool, and store qualification results.
2. Implement the customer-vet skill: gather customer information, invoke sigma1_vet_customer tool to run background/credit checks, and present results to the conversation flow.
3. Implement the quote-gen skill: collect equipment selections and rental parameters, invoke sigma1_catalog_search and sigma1_check_availability for validation, then call sigma1_generate_quote to produce a quote. Present the quote to the customer and handle acceptance/modification.
4. Implement the upsell skill: after quote generation, analyze the quote context and suggest complementary equipment or extended rental periods using sigma1_equipment_lookup and sigma1_catalog_search.
5. For each skill, define clear entry/exit conditions, required tool calls, and conversation prompts.
6. Ensure skills can be composed (e.g., sales-qual → customer-vet → quote-gen → upsell as a single flow).

## Validation
Each skill can be triggered independently and produces correct tool calls; sales-qual invokes sigma1_score_lead and returns a qualification score; customer-vet invokes sigma1_vet_customer and returns a vet result; quote-gen produces a valid quote via sigma1_generate_quote; upsell suggests relevant items; the composed flow (qual → vet → quote → upsell) completes without errors.