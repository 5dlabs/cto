Implement subtask 7013: Implement lead qualification workflow (sales-qual skill)

## Objective
Build the sales-qual skill workflow: detect inquiry intent, guide the customer through qualifying questions (event type, date, budget, venue), check equipment availability, trigger customer vetting, generate a quote, and send approval summary to Mike via Signal.

## Steps
1. Implement sales-qual skill handler:
   - Activated when intent detection identifies equipment rental inquiry
   - Maintains a qualification state machine: GATHERING_INFO → CHECKING_AVAILABILITY → VETTING → GENERATING_QUOTE → AWAITING_APPROVAL
2. Qualification question flow:
   - Ask for event type (wedding, corporate, concert, etc.) if not provided
   - Ask for event date(s) if not provided
   - Ask for venue/location if not provided
   - Ask for budget range if not volunteered
   - Ask for specific equipment needs or let Morgan suggest based on event type
3. Equipment availability check:
   - Once equipment needs and dates are known, call sigma1_check_availability for each item
   - If unavailable, suggest alternatives using sigma1_catalog_search
   - Present availability summary to customer
4. Customer vetting:
   - Once customer provides name/email, call sigma1_vet_customer
   - If 'approved' → proceed to quote
   - If 'flagged' → tell customer 'Let me connect you with our team for this request' and notify Mike
   - If 'pending_review' → tell customer 'We're reviewing your request, I'll follow up shortly'
5. Quote generation:
   - Call sigma1_generate_quote with gathered info and selected equipment
   - Present quote summary to customer (total, line items, dates)
6. Mike notification:
   - After quote is generated, send Signal message to Mike's number (configured via env)
   - Message format: 'New quote #{quote_number} for {customer_name}: {event_type} on {date} at {venue}. Total: ${amount}. Equipment: {items}. Vetting: {status}. Approve/modify?'
   - Parse Mike's response and update quote status accordingly
7. Store qualification progress in conversation state so it survives disconnects.

## Validation
Simulate a full lead qualification conversation: user says 'I need lighting for a wedding on June 15' → verify Morgan asks for venue, budget, specific equipment → verify availability is checked via MCP tool → verify vetting is triggered → verify quote is generated → verify Signal message is sent to Mike's number. Test resume: disconnect mid-qualification, reconnect, and verify Morgan picks up where it left off.