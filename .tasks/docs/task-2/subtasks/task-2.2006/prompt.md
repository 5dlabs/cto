Implement subtask 2006: Write unit and integration tests for agent delegation flow

## Objective
Create comprehensive test coverage for resolve_agent_delegates, the updated issueCreate integration, cache behavior, and end-to-end pipeline delegation.

## Steps
1. **Unit tests for resolve_agent_delegates:**
   a. Mock Linear users API returning 3 agents (bolt, nova, blaze). Assert correct map returned.
   b. Include an unknown hint. Assert it's excluded from map and a warning is logged.
   c. Simulate Linear API failure. Assert graceful degradation (empty map, error logged).
2. **Unit tests for cache:**
   a. Verify second call doesn't hit API. Verify clearDelegateCache resets.
3. **Integration test for issueCreate with assigneeId:**
   a. Mock Linear issueCreate mutation. Submit a task with hint 'nova'. Assert assigneeId is present in mutation variables.
   b. Submit a task with unresolvable hint. Assert assigneeId is absent.
4. **Regression test for legacy label removal:**
   a. Run issue creation flow. Assert no 'agent:pending' label is attached to any created issue.
5. **End-to-end test:**
   a. Run a full pipeline with 5+ tasks. Query created issues and verify non-null assigneeId on all resolvable hints.
6. Use Bun's built-in test runner (`bun test`).

## Validation
All tests pass with `bun test`. Coverage report shows >90% line coverage for resolve-agent-delegates.ts and the modified issue creation module. CI pipeline gate on test pass.