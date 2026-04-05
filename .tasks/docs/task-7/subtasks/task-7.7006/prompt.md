Implement subtask 7006: Configure MCP tool-server with Finance, Vetting, and Social Media backend tools

## Objective
Register MCP tools for the Finance service (invoicing, payments), Customer Vetting service (credit checks, verification), and Social Media Engine (content publishing, portfolio).

## Steps
1. Define and register Finance tools:
   - finance_create_invoice: Generate invoice for a quote/rental.
   - finance_get_invoice_status: Check payment status of an invoice.
   - finance_process_payment: Record a payment against an invoice.
   - finance_generate_quote: Create a price quote for equipment rental.
2. Define and register Vetting tools:
   - vetting_run_credit_check: Initiate credit check for a customer.
   - vetting_get_vetting_status: Get result of a vetting process.
   - vetting_verify_identity: Verify customer identity documents.
3. Define and register Social Media tools:
   - social_publish_content: Publish content to social platforms.
   - social_get_portfolio: Retrieve published portfolio items.
   - social_schedule_post: Schedule a future social media post.
4. Configure each tool's endpoint URL from sigma1-infra-endpoints ConfigMap.
5. Define input/output schemas for each tool.
6. Implement error handling for each tool category.

## Validation
Agent can invoke each finance tool and receive valid structured responses; vetting tools return appropriate check results; social media tools successfully interact with the Social Media Engine; all tool schemas validate correctly; error cases are handled gracefully.