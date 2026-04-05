Implement subtask 10003: Implement automated secret rotation for third-party integration credentials

## Objective
Configure automated rotation schedules for all third-party API keys and credentials (Stripe, LinkedIn, etc.) with zero-downtime pod update mechanisms.

## Steps
1. For each third-party integration, determine the rotation strategy: a) If the provider supports programmatic key rotation (e.g., Stripe API key rolling), create a CronJob or use ESO's `spec.refreshInterval` to automatically fetch the latest key. b) If manual rotation is required, document the process and set up alerting for expiring keys. 2. Configure ExternalSecret resources with appropriate `refreshInterval` (e.g., `1h`) so Kubernetes Secrets stay in sync with the external backend. 3. For credentials that require pod restarts on rotation, implement one of: a) Use Reloader (stakater/Reloader) to watch Secret changes and trigger rolling restarts, or b) Use ESO's `spec.target.template` with annotations that change on rotation. 4. Install Reloader via Helm if chosen, and annotate Deployments with `reloader.stakater.com/auto: 'true'`. 5. Test a simulated rotation: update a credential in the external backend and verify the new value propagates to pods without manual intervention. 6. Set up monitoring alerts for ExternalSecret sync failures.

## Validation
Rotate a test credential in the external secrets backend. Verify the Kubernetes Secret updates within the configured refresh interval. Confirm that affected pods are automatically restarted (if Reloader is used) or pick up the new value. Verify zero downtime by monitoring the service endpoint during rotation. Check that ExternalSecret sync failure alerts fire when the backend is temporarily unreachable.