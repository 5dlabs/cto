Implement subtask 7009: Write end-to-end integration tests for complete workflows

## Objective
Create integration tests covering the full lead → vet → quote → invoice flow and other major multi-skill workflows across all communication channels.

## Steps
1. Write an integration test for the complete sales flow: lead contacts Morgan via chat → sales qualification → vetting → quote generation → quote acceptance → invoice generation. Verify all MCP tools are called in the correct sequence with correct parameters.
2. Write an integration test for the rental management flow: customer requests rental → availability check → rental creation → delivery scheduling → pickup scheduling.
3. Write an integration test for the social media flow: admin drafts post → submission for approval → approval → publishing.
4. Write a multi-channel test: initiate a conversation via Signal, verify it can also be continued via web chat (if session linking is supported).
5. Write SLA tests: measure and assert that simple query responses complete within 10 seconds.
6. Write error/edge case tests: backend service unavailable, invalid customer input, vetting denial mid-flow.
7. Use test fixtures and mock backends where necessary, but prefer real service integration for CI.

## Validation
All integration tests pass in CI; the lead-to-invoice flow completes end-to-end; SLA assertion for 10-second response time passes; error scenarios are handled gracefully without agent crashes.