## Acceptance Criteria

- [ ] 1. Unit tests for scoring/selection algorithm: mock OpenAI responses, verify top N photos selected based on score threshold. 2. Effect Schema validation tests: verify all endpoints reject malformed requests with appropriate error messages. 3. Integration test for upload pipeline: upload test images, verify R2 upload (mock S3 client), verify photo records created in DB with dimensions. 4. Draft approval → NATS publish test: approve draft, verify message published to `social.publish` subject (use NATS test server). 5. Platform publishing test: mock Instagram Graph API, verify publish flow creates published_posts record with post_url. 6. Retry test: mock platform API returning 500 twice then 200, verify Effect.retry succeeds on third attempt. 7. Dead-letter test: mock platform API returning 500 always, verify draft marked as `failed` after 3 attempts. 8. GDPR test: create upload with photos and drafts, call DELETE, verify DB records removed and R2 delete called. 9. Minimum 80% code coverage via Vitest.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.