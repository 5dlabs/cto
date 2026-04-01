Implement subtask 1006: Validate infrastructure with a test pod

## Objective
Deploy a temporary curl/test pod in sigma1-dev to validate that all secrets are mountable, the ConfigMap is readable, DNS resolution works, and network policies allow expected egress while blocking unexpected traffic.

## Steps
1. Author a test pod manifest `test-pod-validation.yaml` using a `curlimages/curl` or `busybox` image.
2. Mount all four secrets as environment variables or volume mounts.
3. Set `envFrom` to reference `sigma1-infra-endpoints` ConfigMap.
4. Apply the pod: `kubectl apply -f test-pod-validation.yaml -n sigma1-dev`.
5. Exec into the pod and run validation checks:
   a. `env | grep PM_SERVER_URL` — confirm ConfigMap keys are present.
   b. `env | grep LINEAR_API_BASE` — confirm value is `https://api.linear.app/graphql`.
   c. `cat /secrets/linear-api-token/token` (or env var) — confirm secret is mounted (non-empty).
   d. Repeat for all four secrets.
   e. `curl -s -o /dev/null -w '%{http_code}' https://api.linear.app/graphql` — expect 200 or 401 (auth required).
   f. `curl -s -o /dev/null -w '%{http_code}' https://api.github.com` — expect 200.
   g. Test that egress to a disallowed host (e.g., `curl https://example.com`) is blocked/times out.
6. Delete the test pod after validation: `kubectl delete -f test-pod-validation.yaml -n sigma1-dev`.

## Validation
All five ConfigMap keys are present as environment variables in the test pod. All four secrets are readable/non-empty. curl to LINEAR_API_BASE returns HTTP 200 or 401. curl to GITHUB_API_BASE returns HTTP 200. curl to a disallowed external host times out or is refused. Test pod cleans up successfully.