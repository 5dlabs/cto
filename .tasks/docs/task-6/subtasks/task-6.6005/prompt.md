Implement subtask 6005: Implement Signal-based approval workflow for Morgan

## Objective
Build the approval notification system that sends curated drafts to Morgan via Signal and processes approval/rejection responses.

## Steps
1. Create a `services/signal` module. 2. Integrate with the chosen Signal interface (signal-cli REST API or subprocess). Configure the Signal phone number and Morgan's contact number from Kubernetes secrets. 3. When new drafts are created by the AI curation pipeline, send a Signal message to Morgan containing: thumbnail image(s), AI-generated caption, AI score, and instructions (reply 'approve <id>' or 'reject <id> <reason>'). 4. Implement a polling mechanism or webhook listener to receive incoming Signal messages from Morgan. 5. Parse incoming messages: match 'approve <id>' pattern → call approve endpoint logic, match 'reject <id> <reason>' → call reject endpoint logic. 6. Send confirmation messages back to Morgan after processing (e.g., 'Draft #123 approved and queued for publishing'). 7. Handle error cases: unknown draft ID, draft already processed, malformed message (reply with help text). 8. Wrap all Signal operations in Effect for error handling and retries.

## Validation
Unit test message formatting and parsing with mocked Signal client. Verify approve/reject commands are correctly parsed and executed. Test error handling: unknown ID, already-processed draft, malformed message. Integration test: send a test message and verify delivery (requires Signal test setup).