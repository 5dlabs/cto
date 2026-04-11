<identity>
You are bolt working on subtask 10013 of task 10.
</identity>

<context>
<scope>
Create an ArgoCD Application CR for each of the 6 services (equipment-catalog, rms, finance, customer-vetting, social-engine, morgan) pointing to helm/sigma1/ in the Git repo. Enable automated sync with selfHeal: true and prune: true. Add a Loki alert rule that fires when any Application has 3+ consecutive sync failures.
</scope>
</context>

<implementation_plan>
Create gitops/applications/equipment-catalog-app.yaml: `apiVersion: argoproj.io/v1alpha1, kind: Application, spec.source: { repoURL: '<git-repo-url>', targetRevision: main, path: helm/sigma1, helm: { valueFiles: ['values-equipment-catalog.yaml'] } }, spec.destination: { server: https://kubernetes.default.svc, namespace: sigma1 }, spec.syncPolicy: { automated: { selfHeal: true, prune: true }, syncOptions: ['CreateNamespace=true'] }`. Repeat for all 6 services with their respective value files. Create a Loki alert rule in monitoring/loki-rules/argocd-alerts.yaml: alert fires when `count_over_time({app='argocd'} |= 'ComparisonError' [10m]) > 3` per application. Route alert to a Loki-compatible notification channel.
</implementation_plan>

<validation>
ArgoCD UI shows 6 Application objects in Synced + Healthy state. Manually introduce a bad Helm value to one app — confirm ArgoCD shows OutOfSync, then selfHeals after the value is reverted. Loki alert: inject 4 sync failure log lines for one app, confirm alert fires in Loki ruler logs.
</validation>