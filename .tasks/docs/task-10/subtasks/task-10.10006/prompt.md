Implement subtask 10006: Configure secret rotation via external-secrets with refreshInterval and rolling restart triggers

## Objective
Update all ExternalSecret CRDs to include `refreshInterval: 1h` for automatic rotation. Configure pod rolling restart triggers so rotated secrets are picked up without manual intervention.

## Steps
1. Identify all ExternalSecret resources in sigma-1-dev: `kubectl get externalsecrets -n sigma-1-dev`.
2. Update each ExternalSecret CR to include `spec.refreshInterval: 1h`:
   ```yaml
   apiVersion: external-secrets.io/v1beta1
   kind: ExternalSecret
   metadata:
     name: sigma-1-secrets
     namespace: sigma-1-dev
   spec:
     refreshInterval: 1h
     secretStoreRef:
       name: <store-name>
       kind: SecretStore
     target:
       name: sigma-1-secrets
       creationPolicy: Owner
     data:
     - secretKey: <key>
       remoteRef:
         key: <remote-path>
   ```
3. For pods that mount secrets as environment variables (not volume mounts), configure a rolling restart trigger. Options:
   a. Use Reloader (stakater/reloader) to watch for Secret changes and trigger rolling restarts.
   b. Or add a hash annotation in the Deployment template that references the Secret's resourceVersion (requires a controller or CI step).
4. Install Reloader if chosen: `helm install reloader stakater/reloader -n sigma-1-dev --set reloader.watchGlobally=false`.
5. Annotate Deployments: `reloader.stakater.com/auto: "true"`.
6. Apply all changes and verify the external-secrets operator is reconciling on schedule.

## Validation
Record the `resourceVersion` of a target Kubernetes Secret: `kubectl get secret sigma-1-secrets -n sigma-1-dev -o jsonpath='{.metadata.resourceVersion}'`. Modify the corresponding value in the backing secret store. Wait up to 70 minutes (slightly beyond refreshInterval). Re-check `resourceVersion` and assert it has changed. If Reloader is installed, verify the associated Deployment's pods were restarted (check pod age is less than refreshInterval).