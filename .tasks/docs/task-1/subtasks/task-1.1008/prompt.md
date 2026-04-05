Implement subtask 1008: Validate end-to-end infrastructure connectivity from a test pod

## Objective
Deploy a temporary test pod that loads connection details from the sigma1-infra-endpoints ConfigMap and validates connectivity to PostgreSQL, Redis, S3/R2, and Signal-CLI.

## Steps
1. Author a Job manifest 'infra-connectivity-test' in the sigma1 namespace.
2. Use a lightweight image (e.g., alpine with curl, psql, redis-cli installed, or a custom test image).
3. Mount the sigma1-infra-endpoints ConfigMap via envFrom and relevant secrets.
4. The job script should:
   a. Connect to PostgreSQL and run 'SELECT 1' and '\dn' to verify schemas.
   b. Connect to Redis and run PING.
   c. Use curl to list S3 buckets or PUT/GET a test object.
   d. curl Signal-CLI /v1/about endpoint.
5. Each check should output PASS/FAIL with details.
6. The Job should exit 0 only if all checks pass.
7. Document the test job so it can be re-run during CI or after infra changes.

## Validation
kubectl get job infra-connectivity-test -n sigma1 shows Completed with exit code 0; kubectl logs job/infra-connectivity-test shows PASS for all four connectivity checks (PostgreSQL, Redis, S3, Signal-CLI).