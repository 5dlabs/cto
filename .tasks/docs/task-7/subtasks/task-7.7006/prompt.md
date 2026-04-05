Implement subtask 7006: Implement finance skill: invoicing and finance reporting

## Objective
Develop the finance skill for Morgan to create invoices and generate financial reports using the sigma1_create_invoice and sigma1_finance_report MCP tools.

## Steps
1. Implement the invoice creation skill: after a quote is accepted, invoke sigma1_create_invoice with quote data and customer details. Handle invoice confirmation and present the invoice ID/link to the customer.
2. Implement the finance reporting skill: handle admin requests for financial summaries by invoking sigma1_finance_report with date ranges and report types. Format report data for conversational presentation.
3. Define role-based access: invoice creation is available in customer flows, finance reporting is restricted to admin/internal users.
4. Handle error cases: failed invoice creation, incomplete data, report generation timeouts.
5. Add logging for all finance operations (invoice ID, amount, report type, generation time).

## Validation
Invoice creation skill invokes sigma1_create_invoice with correct parameters and returns an invoice ID; finance report skill invokes sigma1_finance_report and returns formatted data; role restrictions prevent customer-facing conversations from accessing finance reports; error handling triggers on simulated failures.