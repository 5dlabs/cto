Implement subtask 1001: Create sigma1 and sigma1-db namespaces with labels

## Objective
Create the `sigma1` application namespace and the `sigma1-db` database namespace (or reuse `databases` per cluster convention). Apply standard labels including `app.kubernetes.io/part-of: sigma1` for observability selector matching.

## Steps
1. Create namespace manifest for `sigma1` with labels: `app.kubernetes.io/part-of: sigma1`, `purpose: application`.
2. Create namespace manifest for `sigma1-db` with labels: `app.kubernetes.io/part-of: sigma1`, `purpose: database`.
3. Apply both namespace YAMLs via `kubectl apply`.
4. Verify namespaces exist and labels are correctly applied.

## Validation
`kubectl get ns sigma1 -o jsonpath='{.metadata.labels}'` returns expected labels. Same for `sigma1-db`. Both namespaces are in Active phase.