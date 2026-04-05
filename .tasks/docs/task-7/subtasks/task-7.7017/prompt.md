Implement subtask 7017: Implement blue-green deployment strategy with Valkey-persisted conversation state

## Objective
Configure the Morgan agent deployment for blue-green rollouts where conversation state in Valkey survives pod restarts and version transitions, ensuring zero conversation loss during deployments.

## Steps
1. Configure Deployment strategy:
   - Set strategy type to RollingUpdate with maxUnavailable: 0, maxSurge: 1
   - This ensures a new pod is fully ready before the old one terminates
   - Alternative: use Argo Rollouts blue-green strategy if available in cluster
2. Conversation state resilience:
   - Verify all conversation state is stored in Valkey (external to pod), NOT in-memory
   - On pod startup: reconnect to Valkey, resume any active sessions
   - On pod shutdown (SIGTERM): graceful shutdown handler
     - Close active WebSocket connections with 'server restarting' message
     - Flush any buffered conversation state to Valkey
     - Allow 30-second grace period for in-flight requests
3. Configure terminationGracePeriodSeconds: 45 in pod spec
4. Client reconnection support:
   - WebSocket clients should receive a close frame with code 1012 (Service Restart)
   - Clients can reconnect with session_token to resume seamlessly
5. Pre-stop hook:
   - Signal-CLI sidecar: no special handling needed (stateless relay, data on PVC)
   - Morgan container: flush state, close connections
6. Deployment verification:
   - After new pod becomes ready, run a synthetic health check before draining old pod
   - Verify new pod can connect to Valkey and load existing sessions
7. Rollback procedure:
   - `kubectl rollout undo deployment/morgan-agent`
   - Conversation state in Valkey is version-agnostic, so rollback is seamless

## Validation
Start a WebSocket chat session, send several messages, then trigger a rolling update. Verify the new pod starts and becomes ready. Reconnect with session_token and verify full conversation history is preserved. Verify the old pod sends close frame 1012 before terminating. Test rollback: deploy a broken version, rollback, verify conversations are intact.