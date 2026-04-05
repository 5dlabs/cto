Implement subtask 7004: Implement MCP Tool Server — Customer Vetting tool (vet_customer)

## Objective
Define and implement the MCP tool definition for sigma1_vet_customer, mapping to the Customer Vetting service POST endpoint.

## Steps
1. Create MCP tool definition for `sigma1_vet_customer`:
   - HTTP: POST /api/v1/vetting/run
   - Input schema: { customer_name: string, customer_email: string, company?: string, event_type?: string, additional_context?: string }
   - Output schema: { vetting_id: string, status: 'approved'|'flagged'|'pending_review', risk_score: number, findings: [{ source, result, detail }], recommendation: string }
   - Description: 'Run a background vetting check on a potential customer before proceeding with a rental quote'
2. Include Authorization header: `Bearer ${MORGAN_SERVICE_JWT}`
3. Implement async handling: vetting may take several seconds. If the vetting service returns 202 Accepted with a job ID, implement polling or accept webhook callback.
4. Map vetting results to conversational language Morgan can relay to the customer (e.g., 'approved' → proceed with quote, 'flagged' → escalate to Mike).
5. Error handling: service unavailable → inform agent to skip vetting and note it for manual review.

## Validation
Invoke sigma1_vet_customer with test customer data and verify correct HTTP POST body. Mock vetting service with 'approved', 'flagged', and 'pending_review' responses and verify each maps correctly. Test timeout/unavailable scenario and verify graceful degradation message.