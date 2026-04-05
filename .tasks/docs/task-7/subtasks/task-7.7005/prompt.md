Implement subtask 7005: Implement sales skills: lead qualification, quote generation, and upsell

## Objective
Implement Morgan's sales-focused skills (sales-qual, quote-gen, upsell) that wire to sigma1_catalog_search, sigma1_check_availability, sigma1_generate_quote, sigma1_score_lead, and sigma1_equipment_lookup MCP tools to handle the full sales conversation flow.

## Steps
Step 1: Implement the sales-qual skill — Morgan asks qualifying questions (event type, date, location, budget, guest count), scores the lead via sigma1_score_lead, and routes high-value leads for priority handling. Step 2: Implement the quote-gen skill — Morgan searches the catalog via sigma1_catalog_search, checks availability via sigma1_check_availability, looks up equipment details via sigma1_equipment_lookup, and generates a quote via sigma1_generate_quote. Step 3: Implement the upsell skill — after initial quote generation, Morgan suggests complementary equipment, premium packages, or add-on services based on the event profile. Step 4: Implement conversation flow logic that naturally transitions between qualification → catalog browsing → quote generation → upsell. Step 5: Handle edge cases: unavailable equipment alternatives, date flexibility suggestions, budget-constrained recommendations. Step 6: Format quotes for display across all channels (Signal, voice summary, web chat with rich formatting).

## Validation
Simulate a full sales conversation where a customer describes an event, Morgan qualifies the lead, searches the catalog, checks availability, generates a quote with line items, and suggests upsells; verify each MCP tool is invoked with correct parameters; quote contains accurate pricing and availability data.