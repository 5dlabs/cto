Implement subtask 10009: Configure container image admission policy with Kyverno or Gatekeeper

## Objective
Deploy an admission controller policy that restricts container images in the sigma1 namespace to approved registries only, and enforce image pull policies.

## Steps
1. Choose Kyverno or Gatekeeper (per decision point — default to Kyverno for simplicity if no decision made).
2. If Kyverno:
   - Install Kyverno via Helm if not already present.
   - Create a ClusterPolicy `restrict-image-registries` that:
     - Applies to sigma1 namespace
     - Validates that all container images match allowed patterns (e.g., `ghcr.io/sigma1/*`, `docker.io/library/*` for base images)
     - Action: `enforce` (block non-compliant)
   - Create a ClusterPolicy `require-image-pull-policy` that enforces `imagePullPolicy: Always` for tags, or `IfNotPresent` for SHA-pinned images.
3. If Gatekeeper:
   - Install Gatekeeper via Helm.
   - Create ConstraintTemplate `K8sAllowedRepos` with Rego policy.
   - Create Constraint applying the template to sigma1 namespace.
4. Test by attempting to deploy an image from an unapproved registry.
5. Document the approved registry list in the ops runbook ConfigMap.

## Validation
Attempt to create a pod with an image from an unapproved registry (e.g., `random-registry.io/malicious:latest`) — verify it is rejected. Deploy a pod with an approved registry image — verify it is allowed. Verify the policy is in `enforce` mode. Check policy report/audit for any existing violations.