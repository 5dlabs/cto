## Acceptance Criteria

- [ ] 1. `kubectl get ns sigma-1` returns Active status with expected labels. 2. `kubectl get externalsecrets -n sigma-1` shows all 4 ExternalSecret resources with status 'SecretSynced'. 3. Secret validation job completes with exit code 0, and logs confirm non-empty values for linear-token, discord-webhook, and github-token. 4. `kubectl get configmap sigma-1-infra-endpoints -n sigma-1 -o json` contains all 5 expected keys with non-empty values for DISCORD_BRIDGE_URL, LINEAR_BRIDGE_URL, and PM_SERVER_URL. 5. Health check probes to discord-bridge-http, linear-bridge, and cto-pm return 200 from within the sigma-1 namespace.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.