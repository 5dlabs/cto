Implement subtask 9010: Validate full production hardening deployment end-to-end

## Objective
Deploy all production hardening manifests to the sigma1-prod namespace and run comprehensive validation tests.

## Steps
1. Run `helm upgrade --install sigma1 . -f values-sigma1-prod.yaml -n sigma1-prod`.
2. Verify HPA: `kubectl get hpa -n sigma1-prod` shows PM server HPA with min=3, max=10, target CPU=70%.
3. Verify Ingresses: `kubectl get ingress -n sigma1-prod` shows both ingresses with TLS and ADDRESS populated.
4. Test TLS on API: `curl -I https://api.sigma1.5dlabs.io/health` returns 200 with valid cert.
5. Test TLS and CDN on frontend: `curl -I https://sigma1.5dlabs.io` returns 200; static assets return `Cache-Control: public, max-age=3600`.
6. Verify PDBs: `kubectl get pdb -n sigma1-prod` shows correct minAvailable.
7. Zero-downtime test: delete one PM server pod with `kubectl delete pod <pod>` while running continuous `curl` loop against `/health`; confirm no 5xx errors.
8. Anti-affinity test: `kubectl get pods -o wide -n sigma1-prod -l app=pm-server` confirms pods on multiple nodes/zones.
9. Verify probes: `kubectl describe pod` for both services shows configured readiness and liveness probes.

## Validation
All 8 validation checks above pass successfully. Document results for each check in a validation report. Zero-downtime test must show 0 failed requests during pod deletion.