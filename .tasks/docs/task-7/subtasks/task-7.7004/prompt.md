Implement subtask 7004: Implement customer-vet skill (vetting integration with GREEN/YELLOW/RED interpretation)

## Objective
Build the customer-vet skill that triggers automatically for new customers, calls the vetting service, interprets the GREEN/YELLOW/RED result, and adjusts Morgan's behavior accordingly (deposit requirements, escalation to Mike).

## Steps
1. Define the `customer-vet` skill in OpenClaw.
2. Trigger logic: detect when a conversation involves a customer not previously seen (new phone number on Signal, new contact on web chat). Can also be explicitly triggered by other skills (e.g., sales-qual before generating a quote).
3. Call `sigma1_vet_customer` with available customer information (name, organization, phone, email).
4. Interpret results:
   - GREEN: Proceed normally. No special handling.
   - YELLOW: Proceed with caution. Mention to customer that a deposit may be required. Flag the opportunity in RMS.
   - RED: Require full prepayment/deposit before any equipment reservation. Send alert to Mike via Signal with vetting details. Do NOT generate quotes without Mike's approval.
5. Store vetting result in conversation state so other skills can reference it without re-vetting.
6. Handle vetting service timeout/errors gracefully — default to YELLOW behavior and notify Mike.
7. Compose a natural-language summary of vetting results for Mike when escalating (not raw JSON).

## Validation
Test with three mock customers that return GREEN, YELLOW, and RED ratings respectively. Verify GREEN proceeds without intervention, YELLOW mentions deposit requirement in conversation, RED triggers escalation message to Mike and blocks quote generation. Verify timeout fallback defaults to YELLOW.