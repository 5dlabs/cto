Implement subtask 10008: Apply Pod Security Standards to sigma1 namespace

## Objective
Enforce the 'restricted' PodSecurity level on the sigma1 namespace and update all Deployment/StatefulSet SecurityContexts to comply: runAsNonRoot, readOnlyRootFilesystem, drop all capabilities, disallow privilege escalation.

## Steps
1. Label the sigma1 namespace with Pod Security Admission labels:
   - `pod-security.kubernetes.io/enforce: restricted`
   - `pod-security.kubernetes.io/audit: restricted`
   - `pod-security.kubernetes.io/warn: restricted`
2. Update ALL Deployment and StatefulSet manifests in the sigma1 namespace to include SecurityContext at both pod and container level:
   - Pod level: `securityContext: { runAsNonRoot: true, seccompProfile: { type: RuntimeDefault } }`
   - Container level for each container:
     - `securityContext: { allowPrivilegeEscalation: false, readOnlyRootFilesystem: true, capabilities: { drop: ['ALL'] } }`
   - Add `emptyDir: {}` volumes mounted at `/tmp` (and any other writable paths) for containers that need temp file access.
3. For any container that needs to bind to low ports (unlikely in Kubernetes), add `capabilities: { add: ['NET_BIND_SERVICE'] }` — but only if strictly necessary.
4. Verify init containers and Job containers also comply.
5. Update Helm values or kustomize overlays to enforce these settings as defaults.

## Validation
Apply the namespace labels. Attempt to deploy a test pod with `privileged: true` — verify it is rejected by the admission controller. Attempt to deploy a pod without `runAsNonRoot` — verify it is rejected. Deploy all existing services and verify they start successfully under the restricted policy. Verify no pods are in CrashLoopBackOff due to filesystem permission issues.