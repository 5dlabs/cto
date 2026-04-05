Implement subtask 7006: Implement operations skills: customer vetting, finance, and RMS workflows

## Objective
Implement Morgan's operations-focused skills (customer-vet, finance, rms-*) that wire to sigma1_vet_customer, sigma1_create_invoice, sigma1_finance_report, and RMS-related MCP tools to handle customer verification, invoicing, financial reporting, and rental management workflows.

## Steps
Step 1: Implement the customer-vet skill — Morgan gathers customer identification info, invokes sigma1_vet_customer to run background/credit checks, and communicates the vetting result (approved, conditional, declined) with appropriate messaging. Step 2: Implement the finance skill — Morgan can generate invoices via sigma1_create_invoice (after quote approval), and retrieve financial reports/summaries via sigma1_finance_report for admin users. Step 3: Implement the rms-* skills — Morgan interfaces with the Rental Management System for operations like checking rental status, managing reservations, handling returns, and reporting equipment issues. Step 4: Implement role-based access control within conversation context — finance reports and admin operations are only available to authenticated admin users, not general customers. Step 5: Handle async workflows — vetting may take time, so implement a callback/polling mechanism to notify customers when vetting completes.

## Validation
Simulate a customer vetting workflow end-to-end: customer provides info, Morgan invokes sigma1_vet_customer, result is communicated; simulate invoice creation after quote approval; verify finance reports are only accessible to admin-role conversations; RMS tools return correct rental status data.