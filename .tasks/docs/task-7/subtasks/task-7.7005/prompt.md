Implement subtask 7005: Implement MCP tool-server plugins for business workflow tools (quote, vetting, lead scoring, invoice, finance)

## Objective
Build MCP tool-server plugins for: generate_quote, vet_customer, score_lead, create_invoice, and finance_report. Each plugin integrates with the respective backend service API.

## Steps
1. Implement the `generate_quote` MCP tool plugin:
   - Input: customer_id, list of equipment_ids, rental dates, optional discount code.
   - Calls the Quote/Pricing backend service.
   - Returns a formatted quote with line items, subtotal, tax, and total.
2. Implement the `vet_customer` MCP tool plugin:
   - Input: customer_id or customer details (name, business, license).
   - Calls the Customer Vetting backend service.
   - Returns vetting status (approved/pending/rejected) and any flags.
3. Implement the `score_lead` MCP tool plugin:
   - Input: lead details (contact info, equipment interest, budget, timeline).
   - Calls the Lead Scoring backend service.
   - Returns a numeric score and qualification tier (hot/warm/cold).
4. Implement the `create_invoice` MCP tool plugin:
   - Input: quote_id or manual line items, customer_id, payment terms.
   - Calls the Finance/Invoice backend service.
   - Returns invoice_id and a link/PDF reference.
5. Implement the `finance_report` MCP tool plugin:
   - Input: report_type (revenue, outstanding, overdue), date_range.
   - Calls the Finance reporting backend service.
   - Returns structured report data.
6. Register all five tools with the MCP tool-server.
7. Add input validation and error handling for each.

## Validation
Invoke each of the five tools via the MCP interface with valid test data and verify correct responses from the respective backend services. Test edge cases: generating a quote with unavailable equipment, vetting a nonexistent customer, scoring a lead with minimal data, creating an invoice for a rejected quote, requesting a finance report for an empty date range. All tools appear in the agent's tool registry.