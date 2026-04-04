Implement subtask 1009: Document backing store paths and manual prerequisites in README

## Objective
Write a README documenting all backing store secret paths that must be pre-configured, the ExternalSecret-to-SecretStore mapping, the ConfigMap contract for downstream consumers, and any manual steps required before running the infrastructure provisioning.

## Steps
1. Create `infra/README.md` (or appropriate path) with sections:
   - **Prerequisites**: List the cluster requirements (External Secrets Operator installed, ClusterSecretStore configured, target namespaces `bots` and `cto` with running services).
   - **Backing Store Paths**: Table listing each ExternalSecret name, the remote key path it references, whether it is required or optional, and the resulting Secret name/key.
   - **ConfigMap Contract**: Document `sigma-1-infra-endpoints` keys, their values, and how downstream workloads should reference them via `envFrom`.
   - **Manual Pre-configuration**: If any backing store paths do not yet exist (per Open Question #1), document exactly what needs to be created and where.
   - **Validation**: How to re-run the secret-validation-job and health-check-job to verify the setup.
   - **Cleanup**: How to tear down all sigma-1 infra resources using the `sigma-1-pipeline: infra` label selector.
2. Keep the document concise and actionable — target audience is a platform engineer onboarding to the sigma-1 project.

## Validation
README file exists at the expected path. It contains all 6 sections listed above. Each ExternalSecret and ConfigMap key is documented. The cleanup command using label selector is included and syntactically correct.