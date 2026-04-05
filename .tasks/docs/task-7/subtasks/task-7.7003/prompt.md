Implement subtask 7003: Implement MCP Tool Server — RMS tools (generate_quote, score_lead)

## Objective
Define and implement MCP tool definitions for the two RMS (Rental Management System) tools: sigma1_generate_quote and sigma1_score_lead, with JSON schemas and HTTP mappings.

## Steps
1. Create MCP tool definition for `sigma1_generate_quote`:
   - HTTP: POST /api/v1/opportunities
   - Input schema: { customer_name: string, customer_email: string, event_type: string, event_date: string, venue?: string, line_items: [{ product_id: string, quantity: number, rental_days: number }], notes?: string }
   - Output schema: { opportunity_id: string, quote_number: string, total_amount: number, line_items: [{ product_name, quantity, daily_rate, subtotal }], status: string, pdf_url?: string }
   - Description: 'Generate a rental quote/opportunity with specified equipment line items for a customer event'
2. Create MCP tool definition for `sigma1_score_lead`:
   - HTTP: GET /api/v1/opportunities/:id
   - Input schema: { opportunity_id: string }
   - Output schema: { opportunity_id: string, lead_score: number (0-100), scoring_factors: [{ factor, weight, value }], recommendation: string }
   - Description: 'Retrieve the lead score for an existing opportunity to assess qualification level'
3. Include Authorization header: `Bearer ${MORGAN_SERVICE_JWT}`
4. Implement request/response mapping from MCP tool call format to HTTP requests against RMS service URL.
5. For generate_quote, validate that line_items array is non-empty before sending request.
6. Handle partial failures: if lead scoring data is not yet computed, return a pending status rather than error.

## Validation
Invoke sigma1_generate_quote with sample line items and verify the HTTP POST body matches expected format. Mock RMS service and verify response is correctly parsed. Test sigma1_score_lead with valid opportunity ID and verify lead_score is extracted. Test with invalid opportunity ID and verify graceful error handling.