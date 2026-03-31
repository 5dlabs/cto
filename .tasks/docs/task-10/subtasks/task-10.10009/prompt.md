Implement subtask 10009: Integrate container image vulnerability scanning with Trivy in CI

## Objective
Add Trivy (or chosen scanner) to the CI pipeline to scan all Hermes container images for vulnerabilities before deployment, failing the build on critical/high severity findings.

## Steps
1. Add a Trivy scan step to the CI pipeline (GitHub Actions, GitLab CI, etc.) that runs after image build and before image push/deploy.
2. Configure Trivy to scan the backend and frontend container images:
   ```yaml
   - name: Scan backend image
     run: trivy image --exit-code 1 --severity CRITICAL,HIGH hermes-backend:$TAG
   - name: Scan frontend image
     run: trivy image --exit-code 1 --severity CRITICAL,HIGH hermes-frontend:$TAG
   ```
3. Set `--exit-code 1` to fail the pipeline on CRITICAL or HIGH vulnerabilities.
4. Generate a scan report in SARIF or JSON format for audit trail.
5. Add a `.trivyignore` file for any accepted vulnerabilities with documented justification.
6. Document the scanning process and how to handle false positives.

## Validation
Run the CI pipeline and verify the Trivy scan step executes for both images. Intentionally introduce a known vulnerable base image and verify the pipeline fails. Verify scan reports are generated and stored as CI artifacts.