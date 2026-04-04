Implement subtask 10012: Pod security: apply PodSecurity Standards restricted profile and image scanning policy

## Objective
Enforce PodSecurity Standards restricted profile on the sigma1 namespace (no root, read-only rootfs, drop all capabilities) and configure image scanning to block critical CVEs.

## Steps
Step-by-step:
1. Label the sigma1 namespace with PodSecurity admission labels:
   ```yaml
   labels:
     pod-security.kubernetes.io/enforce: restricted
     pod-security.kubernetes.io/audit: restricted
     pod-security.kubernetes.io/warn: restricted
   ```
2. Verify all service Deployments comply with restricted profile:
   - `securityContext.runAsNonRoot: true`
   - `securityContext.readOnlyRootFilesystem: true` (add emptyDir for `/tmp` if needed)
   - `securityContext.allowPrivilegeEscalation: false`
   - `securityContext.capabilities.drop: [ALL]`
   - `securityContext.seccompProfile.type: RuntimeDefault`
3. Fix any services that fail validation (e.g., add writable emptyDir volumes for temp files).
4. For image scanning: if a policy engine (Kyverno/OPA Gatekeeper) is available, create a policy that blocks images with critical CVEs. If not, document the recommended approach and add a CI-level Trivy scan as a gate.

## Validation
Attempt to deploy a pod in sigma1 with `runAsUser: 0` (root) — verify it is rejected by admission controller with a PodSecurity violation message. Deploy all legitimate services and verify they start successfully under the restricted profile. Run `kubectl get events -n sigma1` and confirm no PodSecurity warnings for production pods.