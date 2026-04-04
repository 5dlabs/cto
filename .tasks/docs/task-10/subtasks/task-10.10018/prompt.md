Implement subtask 10018: ArgoCD GitOps: create Application CRs for all sigma1 services with automated sync

## Objective
Create ArgoCD Application custom resources for each sigma1 service with automated sync, self-heal, prune, and rollback on health check failure.

## Steps
Step-by-step:
1. For each service (equipment-catalog, rms, finance, customer-vetting, social-engine, gdpr-orchestrator, cloudflared), create an ArgoCD Application CR:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Application
   metadata:
     name: sigma1-<service-name>
     namespace: argocd
   spec:
     project: sigma1
     source:
       repoURL: <git-repo-url>
       targetRevision: main
       path: k8s/<service-name>
     destination:
       server: https://kubernetes.default.svc
       namespace: sigma1
     syncPolicy:
       automated:
         selfHeal: true
         prune: true
       syncOptions:
         - CreateNamespace=false
       retry:
         limit: 3
         backoff:
           duration: 5s
           maxDuration: 3m
   ```
2. Create an ArgoCD AppProject `sigma1` that restricts source repos and destination namespaces.
3. Configure health checks: ArgoCD uses built-in Deployment health (checks rollout status). For custom health (e.g., HTTP health endpoint), add a `resource.customizations.health` ConfigMap entry if needed.
4. Rollback: ArgoCD's automated sync with `selfHeal` will revert manual changes. For failed deployments, the Deployment's `progressDeadlineSeconds` + `maxUnavailable` settings handle rollback. Set `progressDeadlineSeconds: 300` on all Deployments.
5. Apply all Application CRs.

## Validation
Verify all ArgoCD Applications show 'Synced' and 'Healthy' in the ArgoCD UI or via `argocd app list`. Manually change a Deployment's image tag via `kubectl edit` and verify ArgoCD reverts it within 30 seconds (self-heal). Update a manifest in Git and verify ArgoCD syncs within 3 minutes. Deploy a broken image (one that fails readiness probe), verify the Deployment does not complete rollout and ArgoCD shows 'Degraded' status.