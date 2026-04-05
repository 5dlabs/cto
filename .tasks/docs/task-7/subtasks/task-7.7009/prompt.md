Implement subtask 7009: End-to-end integration tests for complete multi-channel flows

## Objective
Write and execute comprehensive end-to-end tests covering the full lead-to-invoice lifecycle across Signal, voice, and web chat channels, validating all skills and MCP tool interactions.

## Steps
1. Test: Lead Qualification → Quote Flow (Web Chat)
   - Simulate a new customer inquiry via web chat.
   - Verify sales-qual skill activates, collects info, scores lead.
   - Verify quote-gen skill creates and presents a quote.
   - Assert all tool calls (catalog_search, finance_generate_quote) execute correctly.
2. Test: Customer Vetting → Rental Flow (Signal)
   - Simulate a customer proceeding after quote acceptance via Signal.
   - Verify customer-vet skill triggers credit check and identity verification.
   - Verify rental creation via RMS tools after vetting passes.
   - Assert end-to-end data consistency.
3. Test: Invoice → Payment Flow (Voice)
   - Simulate invoice inquiry and payment confirmation via voice channel.
   - Verify STT/TTS pipeline doesn't corrupt data.
   - Assert finance tools are called correctly.
4. Test: Upsell Flow
   - Verify upsell suggestions appear during quote/rental context.
5. Test: Social Media Publishing
   - Verify content publishing flow via social tools.
6. Test: Cross-channel session continuity (if applicable).
7. Test: Error handling (backend service down, invalid input, timeout).
8. Measure response latency across all channels (target: <10s).

## Validation
All end-to-end flows complete successfully with correct data at each step; response times are under 10 seconds for all channels; error scenarios are handled gracefully without crashes; at least one test per skill and per channel combination passes.