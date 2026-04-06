Implement subtask 7006: Implement sales flow skills: sales-qual, customer-vet, quote-gen, and upsell

## Objective
Implement the core sales-oriented agent skills that handle the lead qualification → vetting → quote generation → upsell pipeline.

## Steps
1. Implement 'sales-qual' skill: when a new lead contacts Morgan, gather requirements (equipment type, rental dates, location, budget), assess fit, and classify lead quality (hot/warm/cold).
2. Implement 'customer-vet' skill: after qualification, trigger the vetting MCP tool to run background/credit checks, wait for results, and communicate approval/denial to the customer.
3. Implement 'quote-gen' skill: based on qualified requirements and equipment availability (via catalog tools), generate a detailed rental quote using the finance create-quote tool, present it to the customer, and handle negotiation.
4. Implement 'upsell' skill: after quote acceptance, suggest complementary equipment, extended rental periods, or premium services based on the customer's requirements and catalog data.
5. Define skill routing logic: the agent's system prompt and tool-use instructions should naturally chain these skills in sequence.
6. Handle edge cases: customer abandonment mid-flow, re-engagement, quote expiry.

## Validation
Simulate a full sales conversation via Signal or chat; verify the agent asks qualifying questions, triggers vetting, generates a quote with correct pricing, and suggests upsells; verify the flow completes end-to-end with correct MCP tool invocations.