Implement subtask 7009: End-to-end integration testing across all channels and workflows

## Objective
Run comprehensive end-to-end tests covering all three channels (Signal, voice, web chat) and all major business workflows (lead qualification → quote → vetting → invoice) to ensure the complete Morgan agent system works together.

## Steps
1. Test the full lead qualification flow via Signal: customer sends an inquiry → Morgan qualifies the lead (score_lead) → searches catalog → checks availability → generates a quote → sends quote back via Signal.
2. Test the full vetting and invoice flow via web chat: customer requests a rental → Morgan vets the customer → generates a quote → creates an invoice → returns invoice reference.
3. Test the voice channel flow: customer calls → asks about equipment availability → Morgan uses equipment_lookup and check_availability → responds with spoken availability info.
4. Test social media workflow: admin requests content curation via web chat → Morgan curates content → admin approves → Morgan publishes.
5. Test RMS workflow: customer checks reservation status via Signal → Morgan retrieves reservation details.
6. Test cross-skill transitions: a single conversation that starts with equipment inquiry, transitions to quote generation, then to reservation creation.
7. Measure response latency for each channel and verify < 10 seconds for simple queries.
8. Test error scenarios: backend service unavailable, invalid customer data, equipment not found.
9. Document any integration issues and their resolutions.

## Validation
All six test scenarios above complete successfully end-to-end. Response latency is under 10 seconds for simple queries across all channels. Error scenarios return helpful error messages rather than crashes. No orphaned conversations or leaked sessions. All MCP tool invocations during E2E tests return correct data from their respective backend services.