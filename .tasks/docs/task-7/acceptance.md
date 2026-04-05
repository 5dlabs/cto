## Acceptance Criteria

- [ ] 1. MCP tool connectivity test: for each of the 10 tools, invoke with valid parameters and verify successful response from the corresponding backend service (requires all 5 backend services running). 2. Signal round-trip test: send a test message via Signal-CLI REST API, verify Morgan receives it, processes it, and sends a response back within 10 seconds. 3. Skill test — sales-qual: simulate a multi-turn conversation ("I need lighting for a wedding on June 15"), verify Morgan calls catalog_search, check_availability, and offers to generate a quote. 4. Skill test — customer-vet: trigger vetting for a test org, verify Morgan calls sigma1_vet_customer and correctly interprets GREEN/YELLOW/RED result. 5. Voice integration test: make a test call via Twilio, verify ElevenLabs processes speech and Morgan responds with relevant content. 6. Web chat WebSocket test: connect via WebSocket, send message, verify JSON response with correct structure within 10 seconds. 7. Performance test: 10 concurrent simple queries (catalog search), verify all respond within 10 seconds. 8. Signal-CLI health monitoring: verify liveness probe detects unhealthy Signal-CLI sidecar and pod restarts.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.