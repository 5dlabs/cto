# Integration & Deployment Verification - Acceptance Criteria

## PR Merge Criteria

- [ ] **All open PRs reviewed and processed**
  - Validation: `gh pr list --state open | wc -l` returns 0 or all documented
  
- [ ] **PRs merged in correct dependency order**
  - Validation: Check tasks.json dependencies, verify prerequisites merged first
  
- [ ] **No merge conflicts left unresolved**
  - Validation: All attempted merges either succeeded or documented
  
- [ ] **Merged branches cleaned up**
  - Validation: `git branch -r | grep feature/ | wc -l` shows only active branches

## Testing Criteria

### Backend Integration

- [ ] **All backend integration tests passing**
  - Validation: `cargo test --test "*"` (or equivalent) exits with code 0
  - Minimum: 95% test pass rate
  
- [ ] **Unit test coverage maintained**
  - Validation: Coverage report shows no significant decrease
  
- [ ] **No new test warnings or errors**
  - Validation: Review test output for warnings

### Frontend Integration (if applicable)

- [ ] **E2E tests passing**
  - Validation: `npx playwright test` (or equivalent) exits with code 0
  - Minimum: 90% test pass rate
  
- [ ] **Component tests passing**
  - Validation: `npm run test:unit` exits with code 0
  
- [ ] **No visual regressions**
  - Validation: Visual regression tests pass or changes approved

### API Testing

- [ ] **All API endpoints accessible**
  - Validation: Health check endpoints return 200 OK
  - Validation: Core endpoints return expected responses
  
- [ ] **API contracts verified**
  - Validation: Contract tests pass or schema validation succeeds
  
- [ ] **API documentation accurate**
  - Validation: Swagger/OpenAPI specs match implementation

### Inter-Service Testing

- [ ] **Service-to-service communication working**
  - Validation: Services can communicate with dependencies
  
- [ ] **Message queues functioning** (if applicable)
  - Validation: Messages sent are received and processed
  
- [ ] **External integrations working** (if applicable)
  - Validation: Third-party API calls succeed

## Deployment Criteria

### Configuration

- [ ] **Kubernetes manifests valid**
  - Validation: `kubectl apply --dry-run=client -f k8s/` succeeds
  
- [ ] **Helm charts valid** (if applicable)
  - Validation: `helm lint charts/*` passes
  - Validation: `helm template charts/*` generates valid YAML
  
- [ ] **Environment variables documented**
  - Validation: All required env vars listed in docs
  
- [ ] **Secrets exist in target environment**
  - Validation: `kubectl get secrets -n <namespace>` shows required secrets

### Database

- [ ] **Database migrations ready**
  - Validation: Migration status shows "up to date" or pending migrations listed
  
- [ ] **Migration scripts validated**
  - Validation: Dry-run migrations succeed without errors
  
- [ ] **Rollback plan exists**
  - Validation: Rollback migrations documented and tested

### Service Health

- [ ] **All services running** (if deployed)
  - Validation: `kubectl get pods -n <namespace>` shows all pods Running
  
- [ ] **Health endpoints responding**
  - Validation: `curl -f http://service:port/health` returns 200
  
- [ ] **Readiness probes passing**
  - Validation: `curl -f http://service:port/ready` returns 200
  
- [ ] **No pod restarts or crashes**
  - Validation: Pod restart count is 0 or low and stable

## Smoke Testing Criteria

- [ ] **User authentication working**
  - Validation: Can login/logout successfully
  - Validation: Session management works correctly
  
- [ ] **Core features functional**
  - Validation: Primary user workflows complete successfully
  
- [ ] **Data persistence verified**
  - Validation: CRUD operations work correctly
  - Validation: Data survives service restarts
  
- [ ] **Error handling working**
  - Validation: Errors display user-friendly messages
  - Validation: System recovers gracefully from errors
  
- [ ] **Authorization enforced**
  - Validation: Role-based access control works
  - Validation: Unauthorized actions are blocked

## Performance Criteria

- [ ] **API response times acceptable**
  - Validation: P95 response time < 500ms for critical endpoints
  - Validation: No significant regression from baseline
  
- [ ] **Resource utilization within limits**
  - Validation: CPU usage < 80% under normal load
  - Validation: Memory usage < 80% of allocated
  
- [ ] **Database queries optimized**
  - Validation: No N+1 queries detected
  - Validation: Query times within expected range

## Security Criteria

- [ ] **No critical vulnerabilities**
  - Validation: `cargo audit` (or `npm audit`, etc.) shows no critical issues
  
- [ ] **No secrets in code**
  - Validation: `git secrets --scan` passes
  
- [ ] **Container images scanned**
  - Validation: `trivy image <image:tag>` shows no HIGH/CRITICAL CVEs
  
- [ ] **Dependencies up to date**
  - Validation: No outdated dependencies with known vulnerabilities

## Documentation Criteria

- [ ] **API documentation updated**
  - Validation: API docs reflect all endpoints and changes
  
- [ ] **Deployment guide current**
  - Validation: Deployment instructions are accurate
  
- [ ] **Configuration changes documented**
  - Validation: New config options listed and explained
  
- [ ] **Breaking changes noted**
  - Validation: CHANGELOG.md includes breaking changes
  
- [ ] **README updated** (if needed)
  - Validation: Setup instructions are current

## Reporting Criteria

- [ ] **Integration report created**
  - Location: `docs/integration-report-[DATE].md`
  - Contains: PR summary, test results, deployment status
  
- [ ] **All failures documented**
  - Each failure includes: what, why, impact, next steps
  
- [ ] **Deployment recommendation provided**
  - Clear GO/NO-GO decision with reasoning
  
- [ ] **Follow-up tasks identified**
  - Any remaining work captured in task list
  
- [ ] **Stakeholders notified**
  - Integration status communicated to team

## Final Deployment Checklist

- [ ] All prerequisite tasks completed (dependencies satisfied)
- [ ] All PRs merged or documented why not
- [ ] All tests passing (≥95% success rate)
- [ ] All services healthy (0 error state pods)
- [ ] No critical bugs found during integration
- [ ] No critical security vulnerabilities
- [ ] Performance within acceptable range
- [ ] Documentation up to date
- [ ] Integration report complete
- [ ] Ready for production deployment

## Validation Commands

**Quick Health Check:**
```bash
# Check all services
kubectl get pods -n <namespace> --field-selector=status.phase=Running

# Test health endpoints
curl -f http://service:port/health && echo "✅ Healthy"

# Verify tests
cargo test --test "*" && echo "✅ Tests passing"
```

**Full Validation:**
```bash
# Run integration validation script
./scripts/validate-integration.sh  # if exists

# Or manual validation:
cargo test --all-features &&
npm run test:integration &&
kubectl get pods --field-selector=status.phase=Running &&
curl -f http://service:port/health &&
echo "✅ Integration validation complete"
```

## Exit Criteria

**Success:** All criteria above are met or failures are documented with approved workarounds.

**Failure:** Critical criteria not met and no acceptable workaround exists. Deployment blocked until resolved.

**Partial Success:** Most criteria met, some non-critical failures documented. Deployment proceeds with caveats and follow-up tasks created.

