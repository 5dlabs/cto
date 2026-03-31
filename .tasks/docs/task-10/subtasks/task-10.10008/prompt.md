Implement subtask 10008: Apply Pod Security contexts to all Hermes pods

## Objective
Configure SecurityContext on all Hermes pod specs with runAsNonRoot, readOnlyRootFilesystem (with writable tmpdir for headless browser), allowPrivilegeEscalation=false, and drop all capabilities.

## Steps
1. Update backend Deployment pod spec with SecurityContext:
   ```yaml
   securityContext:
     runAsNonRoot: true
     runAsUser: 1000
     fsGroup: 1000
   containers:
   - securityContext:
       allowPrivilegeEscalation: false
       readOnlyRootFilesystem: true
       capabilities:
         drop: ["ALL"]
   ```
2. Add an emptyDir volume mounted at `/tmp` for the headless browser's temporary files (since readOnlyRootFilesystem prevents writing to the default tmpdir).
3. Update frontend Deployment pod spec with the same SecurityContext (without the tmpdir mount unless needed by Next.js).
4. Apply `PodSecurityStandard: restricted` label on the `hermes-production` namespace: `pod-security.kubernetes.io/enforce: restricted`.
5. Verify all containers in operator-managed pods (CNPG, Redis, NATS, MinIO) also meet the restricted standard or document exceptions.
6. Test that the backend headless browser (Puppeteer/Playwright) still functions correctly with the restricted security context.

## Validation
Verify `kubectl get pod -n hermes-production -o jsonpath='{.items[*].spec.containers[*].securityContext}'` shows `runAsNonRoot: true`, `allowPrivilegeEscalation: false`, `readOnlyRootFilesystem: true` for all Hermes application pods. Verify headless browser screenshot capture still works by triggering a deliberation that requires a screenshot. Verify namespace has the restricted PSS label.