Implement subtask 1008: Validate end-to-end infrastructure connectivity

## Objective
Run a comprehensive connectivity test from a test pod to verify all provisioned infrastructure services are reachable and functional, and that the ConfigMap provides correct endpoints.

## Steps
1. Deploy a test pod in the sigma1 namespace with envFrom: sigma1-infra-endpoints ConfigMap and all external Secrets mounted.
2. From the test pod:
   a. Connect to PostgreSQL using POSTGRES_URL, run SELECT 1, list schemas.
   b. Connect to Redis using REDIS_URL, run PING.
   c. Use AWS CLI to list S3 buckets at S3_ENDPOINT.
   d. Curl SIGNAL_CLI_URL health endpoint.
   e. Curl ElevenLabs API with the API key (or verify key format).
   f. Verify Twilio credentials format (optional: make a test API call to Twilio's verify endpoint).
3. Collect results and report any failures.
4. Clean up the test pod after validation.

## Validation
All connectivity checks pass: PostgreSQL returns query result, Redis returns PONG, S3 bucket list succeeds, Signal-CLI health returns 200. Test pod logs contain no errors. All checks documented in a validation report.