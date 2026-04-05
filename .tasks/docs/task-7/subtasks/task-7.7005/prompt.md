Implement subtask 7005: Implement MCP Tool Server — Finance tools (create_invoice, finance_report)

## Objective
Define and implement MCP tool definitions for the two Finance service tools: sigma1_create_invoice and sigma1_finance_report.

## Steps
1. Create MCP tool definition for `sigma1_create_invoice`:
   - HTTP: POST /api/v1/invoices
   - Input schema: { opportunity_id: string, customer_email: string, due_date?: string, payment_terms?: string, notes?: string }
   - Output schema: { invoice_id: string, invoice_number: string, amount_due: number, status: string, pdf_url?: string, payment_link?: string }
   - Description: 'Create an invoice from an approved opportunity/quote for the customer'
2. Create MCP tool definition for `sigma1_finance_report`:
   - HTTP: GET /api/v1/finance/reports/{report_type}
   - Input schema: { report_type: 'revenue'|'outstanding'|'monthly-summary', date_from?: string, date_to?: string }
   - Output schema: { report_type: string, period: { from, to }, data: object, generated_at: string }
   - Description: 'Generate financial reports including revenue summaries, outstanding invoices, and monthly breakdowns'
3. Include Authorization header: `Bearer ${MORGAN_SERVICE_JWT}`
4. Implement URL path parameter substitution for report_type in finance_report.
5. Error handling for create_invoice: validate opportunity_id exists before calling, handle 409 Conflict if invoice already exists for opportunity.

## Validation
Invoke sigma1_create_invoice with a test opportunity ID and verify correct HTTP POST body formation. Mock Finance service and verify response parsing. Invoke sigma1_finance_report with each report_type and verify URL path parameter substitution is correct. Test 409 Conflict handling for duplicate invoice creation.