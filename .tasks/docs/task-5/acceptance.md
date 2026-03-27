## Acceptance Criteria

- [ ] 1. Deploy the service to Kubernetes and verify it starts successfully.
- [ ] 2. Call `POST /api/v1/vetting/run` with sample organization data and verify a `VettingResult` is stored in PostgreSQL.
- [ ] 3. Retrieve vetting results using `GET /api/v1/vetting/:org_id` and confirm the `final_score` is correctly computed based on mock inputs.
- [ ] 4. Verify `GET /api/v1/vetting/credit/:org_id` returns expected credit signals.
- [ ] 5. Test error handling for failed external API calls (e.g., by simulating an API timeout or error).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.