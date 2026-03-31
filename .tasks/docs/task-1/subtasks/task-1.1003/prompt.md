Implement subtask 1003: Create CiliumNetworkPolicies for Namespace Isolation

## Objective
Deploy CiliumNetworkPolicy resources implementing default-deny ingress, intra-namespace allow, MinIO egress allow (port 9000 to GitLab or dedicated instance), DNS egress allow, and explicit cross-namespace isolation between hermes-staging and hermes-production.

## Steps
Step-by-step:
1. Create `templates/cilium-default-deny.yaml`: CiliumNetworkPolicy with `spec.endpointSelector: {}` and empty `ingress: []` to deny all ingress by default in `{{ .Values.namespace }}`.
2. Create `templates/cilium-allow-intra-namespace.yaml`: Allow ingress from pods within the same namespace using `fromEndpoints` with `matchLabels: {"k8s:io.kubernetes.pod.namespace": "{{ .Values.namespace }}"}`.
3. Create `templates/cilium-allow-minio-egress.yaml`: Allow egress to MinIO on port 9000. Use conditional logic:
   - If `{{ .Values.minio.dedicated }}` is false: target `gitlab-minio-svc.gitlab.svc` via `toServices` or `toEndpoints` with namespace selector for `gitlab`.
   - If `{{ .Values.minio.dedicated }}` is true: target the dedicated MinIO service in `hermes-minio` namespace.
4. Create `templates/cilium-allow-dns-egress.yaml`: Allow egress to kube-dns on port 53 (TCP and UDP) in the `kube-system` namespace.
5. All policies namespaced to `{{ .Values.namespace }}` with standard labels.
6. Verify rendered YAML with `helm template --debug` for both environments and both minio.dedicated=true/false.

## Validation
Deploy a test pod (busybox/curl) in hermes-staging: (1) `curl http://<staging-service>:port` within namespace succeeds; (2) `curl http://<production-service>.hermes-production.svc:port` times out or is refused; (3) `curl http://gitlab-minio-svc.gitlab.svc:9000` succeeds (or dedicated endpoint if applicable); (4) `nslookup kubernetes.default` succeeds (DNS works). Remove test pod after validation.