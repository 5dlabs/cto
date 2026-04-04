Implement subtask 7003: Implement sales-qual skill (multi-turn lead qualification conversation flow)

## Objective
Build the sales-qual skill that drives multi-turn conversations to qualify leads by identifying event type, date, venue, budget, and equipment needs, using catalog_search and check_availability tools, ending with quote generation or escalation.

## Steps
1. Define the `sales-qual` skill configuration in OpenClaw.
2. Implement conversation flow stages:
   a. Greeting and intent detection — identify that the customer wants to rent equipment.
   b. Event details gathering — ask for event type (wedding, corporate, concert, etc.), date/date range, venue name/location, expected attendance.
   c. Equipment needs — based on event type, ask targeted questions (e.g., for weddings: 'Do you need uplighting, a DJ booth, or a full lighting rig?'). Use `sigma1_catalog_search` to find matching products.
   d. Availability check — for each identified product, call `sigma1_check_availability` with the event dates. Report conflicts or alternatives.
   e. Budget discussion — ask about budget range, adjust recommendations accordingly.
   f. Decision point: if customer is ready, transition to quote-gen skill. If customer needs time, offer to follow up. If requirements exceed Morgan's knowledge, escalate to Mike.
3. Configure tool call parallelization: when checking availability for multiple items, issue concurrent `check_availability` calls.
4. Include conversation state tracking: store current stage, gathered info, and tool call results in workspace.
5. Handle edge cases: customer changes dates mid-conversation, asks about products not in catalog, requests custom builds (escalate).

## Validation
Simulate a complete multi-turn conversation: 'I need lighting for a wedding on June 15 at The Grand Ballroom, budget around $5000.' Verify Morgan calls catalog_search with relevant terms, calls check_availability for suggested products, and offers to generate a quote. Verify escalation triggers when customer requests custom fabrication.