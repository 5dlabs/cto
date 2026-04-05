Implement subtask 9013: Create ArgoCD Application CR for sigma1 namespace

## Objective
Define an ArgoCD Application CR for the sigma1 namespace with automated sync, self-heal, and prune enabled, pointing to the infrastructure Git repository.

## Steps
1. Create ArgoCD Application CR:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Application
   metadata:
     name: sigma1
     namespace: argocd
   spec:
     project: default
     source:
       repoURL: <infrastructure-repo-url>
       targetRevision: main
       path: sigma1/
     destination:
       server: https://kubernetes.default.svc
       namespace: sigma1
     syncPolicy:
       automated:
         prune: true
         selfHeal: true
       syncOptions:
         - CreateNamespace=true
         - PrunePropagationPolicy=foreground
       retry:
         limit: 5
         backoff:
           duration: 5s
           factor: 2
           maxDuration: 3m
   ```
2. Ensure the infrastructure repo has all sigma1 manifests organized under the `sigma1/` directory (or appropriate path).
3. Apply the Application CR to the argocd namespace.
4. Verify ArgoCD picks up the application and syncs successfully.
5. Test self-heal: manually delete a resource, verify ArgoCD recreates it within the retry window.

## Validation
Verify ArgoCD Application shows 'Synced' and 'Healthy' status in the ArgoCD UI or via `argocd app get sigma1`. Test self-heal by manually deleting a non-critical ConfigMap — verify ArgoCD recreates it within 1 minute. Test prune by removing a manifest from the repo, pushing, and verifying the resource is pruned from the cluster.