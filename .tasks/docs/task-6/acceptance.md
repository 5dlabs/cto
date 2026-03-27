## Acceptance Criteria

- [ ] 1. Verify the Morgan agent pod is running and accessible.
- [ ] 2. Send a test message via Signal to Morgan and confirm a response is received.
- [ ] 3. Initiate a voice call via Twilio/ElevenLabs and verify Morgan can respond verbally.
- [ ] 4. Test `sales-qual` skill: Send a natural language query like 'Can you qualify a new lead for me?' and verify Morgan triggers `sigma1_vet_customer` and `sigma1_score_lead` tools, returning a lead score.
- [ ] 5. Test `quote-gen` skill: Ask Morgan to 'Generate a quote for 5 projectors for next week' and verify it uses `sigma1_catalog_search`, `sigma1_check_availability`, and `sigma1_generate_quote` tools, providing a quote ID.
- [ ] 6. Confirm Cloudflare Tunnel is correctly routing traffic to the Morgan agent.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.