Implement subtask 9007: Set resource requests and limits on all sigma-1 pods

## Objective
Configure CPU and memory requests and limits for all pods in the sigma-1 namespace, starting with cto-pm at 256Mi/250m requests and 512Mi/500m limits.

## Steps
Step-by-step:
1. Update the cto-pm Deployment spec with resource requirements:
   ```yaml
   spec:
     containers:
     - name: cto-pm
       resources:
         requests:
           memory: "256Mi"
           cpu: "250m"
         limits:
           memory: "512Mi"
           cpu: "500m"
   ```
2. Identify all other pods in sigma-1 (e.g., external-secrets pods, any sidecar containers) and set appropriate resource requests/limits for each.
3. Apply the updated manifests.
4. Verify: `kubectl describe pod <pod> -n sigma-1` shows non-zero requests and limits for all containers.
5. Optionally, create a LimitRange for the namespace as a safety net:
   ```yaml
   apiVersion: v1
   kind: LimitRange
   metadata:
     name: sigma-1-limits
     namespace: sigma-1
   spec:
     limits:
     - default:
         cpu: "500m"
         memory: "512Mi"
       defaultRequest:
         cpu: "100m"
         memory: "128Mi"
       type: Container
   ```

## Validation
`kubectl describe pod -n sigma-1 | grep -A4 'Requests\|Limits'` shows non-zero values for cpu and memory on every container in every pod. A pod without explicit resources in sigma-1 picks up LimitRange defaults.