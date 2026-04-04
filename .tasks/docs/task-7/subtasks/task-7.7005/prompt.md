Implement subtask 7005: Implement quote-gen and upsell skills

## Objective
Build the quote-gen skill that assembles line items from catalog data, calculates totals, creates an opportunity via RMS, and sends a quote summary. Build the upsell skill that suggests additional services/equipment after quote generation.

## Steps
1. `quote-gen` skill:
   a. Accept gathered product/event data from sales-qual conversation state.
   b. Assemble line items: product name, quantity, daily/weekly rate, rental period calculation.
   c. Calculate subtotals, delivery fees (if applicable), damage waiver/insurance, tax estimates.
   d. Call `sigma1_generate_quote` (POST to RMS /api/v1/opportunities) with the assembled quote data.
   e. Format the quote summary as a readable message: itemized list, totals, rental period, pickup/delivery options.
   f. Send quote summary back to customer via the active channel (Signal, voice, or web chat).
   g. Store opportunity ID in conversation state for follow-up.
2. `upsell` skill:
   a. Trigger after quote-gen completes successfully.
   b. Based on event type and selected equipment, suggest complementary items:
      - Weddings: uplighting add-on, haze machine, photo booth lighting
      - Corporate: confidence monitors, presentation clickers, podium lighting
      - Concerts: additional moving heads, fog machines, follow spots
   c. Suggest insurance/damage waiver if not already included.
   d. Suggest delivery/setup service if customer selected pickup.
   e. Use `sigma1_catalog_search` to find upsell products and `sigma1_check_availability` to verify.
   f. If customer accepts upsell items, regenerate quote via quote-gen with updated line items.

## Validation
Verify quote-gen produces a correctly formatted quote with accurate line item calculations by providing known product data and verifying totals. Verify RMS opportunity is created with correct data. Verify upsell suggests relevant items for a wedding event type and can regenerate the quote with accepted additions.