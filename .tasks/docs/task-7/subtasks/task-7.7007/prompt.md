Implement subtask 7007: Implement agent skills: sales-qual, customer-vet, and quote-gen

## Objective
Implement the sales qualification, customer vetting, and quote generation skill definitions within the Morgan agent, including conversation flows, tool orchestration logic, and decision trees.

## Steps
1. Implement sales-qual skill:
   - Define qualifying questions (project type, timeline, budget, equipment needs).
   - Implement lead scoring logic based on responses.
   - Configure tool calls to catalog_search for equipment matching.
   - Define escalation criteria (high-value leads, complex requirements).
2. Implement customer-vet skill:
   - Define the vetting conversation flow (collect customer info, explain process).
   - Orchestrate vetting_run_credit_check and vetting_verify_identity tools.
   - Handle async vetting results (polling or webhook-based updates).
   - Communicate vetting outcomes to the customer appropriately.
3. Implement quote-gen skill:
   - Collect rental parameters (equipment, duration, delivery location).
   - Call finance_generate_quote with parameters.
   - Present quote to customer with line items and totals.
   - Handle quote acceptance/modification flow.
4. Wire skills into the agent's routing logic so the LLM selects the appropriate skill based on conversation context.

## Validation
Sales-qual skill correctly identifies lead quality and recommends equipment; customer-vet skill triggers vetting tools and communicates results; quote-gen skill produces accurate quotes with correct pricing; skill routing selects the correct skill based on user intent.