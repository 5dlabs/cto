Implement subtask 8009: Implement Test Case 6: Discord Notification Assertions

## Objective
Write the E2E test that queries the Discord webhook collector and asserts the correct number of messages with expected content (pipeline start and complete notifications).

## Steps
1. Test Case 6 (`it('sends >= 2 Discord notifications with correct content')`):
   a. Call `GET ${DISCORD_COLLECTOR_URL}/messages` to retrieve all collected webhook payloads.
   b. Assert: `messages.length >= 2`.
   c. Find the 'start' message: search for a message whose `content` or `embeds[0].description` contains the `runId`.
   d. Assert: start message exists and contains the run ID.
   e. Find the 'complete' message: search for a message that contains a task count (e.g., a number matching the task count from Test Case 2).
   f. Assert: complete message exists and contains the task count.
   g. Log all received messages for debugging.
2. If no messages are found, provide diagnostic info: is the collector running? Was the webhook URL configured correctly?
3. In `afterAll`, stop the Discord collector process and clean up.

## Validation
Test passes when the webhook collector has received >= 2 messages, one containing the pipeline run ID (start notification) and one containing the task count (completion notification). Collector is cleanly shut down after tests.