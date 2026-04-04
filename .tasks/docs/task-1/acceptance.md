## Acceptance Criteria

- [ ] 1. `kubectl get namespace sigma-1-dev` returns Active status. 2. `kubectl get secret sigma-1-secrets -n sigma-1-dev` exists and contains exactly 4 keys (LINEAR_API_KEY, DISCORD_WEBHOOK_URL, NOUS_API_KEY, GITHUB_TOKEN). 3. `kubectl get configmap sigma-1-infra-endpoints -n sigma-1-dev -o json` contains all 4 endpoint keys with non-empty values. 4. `kubectl get serviceaccount sigma-1-pm-server -n sigma-1-dev` exists. 5. A connectivity test pod in `sigma-1-dev` can resolve DNS for `discord-bridge-http`, `linear-bridge`, and `openclaw-nats.openclaw.svc.cluster.local`.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.