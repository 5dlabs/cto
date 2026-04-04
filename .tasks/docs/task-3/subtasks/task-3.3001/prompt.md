Implement subtask 3001: Implement Hermes endpoint discovery with ConfigMap and service discovery fallback

## Objective
Create a module that resolves the Hermes agent endpoint URL by first checking the HERMES_URL environment variable (sourced from sigma-1-infra-endpoints ConfigMap), then falling back to Kubernetes service discovery at hermes.cto.svc.cluster.local and hermes.cto-tools.svc.cluster.local, and finally returning null if neither is reachable.

## Steps
1. Create `src/research/hermes-discovery.ts`.
2. Export an async function `discoverHermesEndpoint(): Promise<string | null>`.
3. First, read `process.env.HERMES_URL`. If non-empty, return it immediately and log `[research:discovery] Using HERMES_URL from ConfigMap: <url>`.
4. If HERMES_URL is empty/undefined, attempt an HTTP GET to `http://hermes.cto.svc.cluster.local/health` with a 5-second timeout. If it responds 200, return that base URL and log the discovery.
5. If that fails, attempt the same against `http://hermes.cto-tools.svc.cluster.local/health` with a 5-second timeout.
6. If both fail, log a warning `[research:discovery] No Hermes endpoint found via ConfigMap or service discovery` and return null.
7. Use Bun's native `fetch` for HTTP calls. Wrap each attempt in try/catch so DNS resolution failures and timeouts are handled gracefully.
8. Export types: `HermesDiscoveryResult = { url: string | null; source: 'configmap' | 'service-discovery-cto' | 'service-discovery-cto-tools' | 'none' }`.

## Validation
Unit test with mocked fetch: (1) When HERMES_URL env var is set, function returns that URL without making HTTP calls. (2) When HERMES_URL is unset but hermes.cto.svc.cluster.local responds 200, that URL is returned. (3) When first service fails but hermes.cto-tools.svc.cluster.local responds 200, second URL is returned. (4) When all fail, null is returned and a warning is logged. Verify log output contains expected source labels.