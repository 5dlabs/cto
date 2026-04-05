## Acceptance Criteria

- [ ] All services remain available during failover; public endpoints serve via Cloudflare with valid TLS; Morgan and website accessible via Cloudflare Tunnel; network policies block unauthorized access; simulate pod/node failure and verify recovery.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.