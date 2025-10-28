# Integration & Deployment Verification

## Overview

This task coordinates the final integration of all parallel development work. As the testing/integration specialist agent (Tess), you are responsible for ensuring all features are properly merged, tested, and deployment-ready.

## Responsibilities

### 1. PR Merge Coordination

**Check PR Status:**
```bash
gh pr list --state open --json number,title,mergeable,statusCheckRollup,headRefName
```

**Merge Strategy:**
- Review all open PRs and their dependency chains
- Merge PRs in dependency order (prerequisites first)
- For each PR:
  ```bash
  # Verify checks are passing
  gh pr checks <PR_NUMBER>
  
  # Merge using squash strategy
  gh pr merge <PR_NUMBER> --squash --delete-branch
  ```

**Conflict Resolution:**
- If merge conflicts occur, document them clearly
- Attempt automatic resolution for simple conflicts
- Request manual intervention for complex conflicts
- Never force-merge without resolving conflicts

### 2. Integration Testing

#### Backend Integration Tests
Run the full integration test suite:
```bash
# Rust projects
cargo test --test "*" -- --test-threads=1

# Node.js projects  
npm run test:integration

# Python projects
pytest tests/integration/

# Go projects
go test ./... -tags=integration
```

#### API Contract Testing
Verify all service contracts:
```bash
# Check endpoint availability
curl -f http://service:port/health
curl -f http://service:port/ready

# Run API test suite if available
npm run test:api  # or equivalent
```

#### Frontend Integration (if applicable)
Run end-to-end tests:
```bash
# Playwright
npx playwright test

# Cypress
npm run cypress:run

# Verify UI renders correctly
npm run test:e2e
```

#### Inter-Service Communication
Test service interactions:
- Verify message queues (if used)
- Test API gateway routing
- Confirm service mesh connectivity
- Validate load balancer configuration

### 3. Deployment Verification

#### Configuration Validation
```bash
# Check deployment manifests
kubectl apply --dry-run=client -f k8s/ || echo "Validation needed"

# Verify environment variables
env | grep -E "(API_|DB_|SERVICE_)" | sort

# Validate Helm charts (if used)
helm lint ./charts/*
helm template ./charts/* --debug
```

#### Database Migrations
```bash
# Check migration status
# Rust (sqlx)
sqlx migrate info

# Node.js (Prisma)
npx prisma migrate status

# Python (Alembic)
alembic current

# Verify schema matches expectations
```

#### Service Health Checks
```bash
# Kubernetes deployments
kubectl get pods -n <namespace>
kubectl get services -n <namespace>
kubectl get ingress -n <namespace>

# Check all pods are Running
kubectl get pods --field-selector=status.phase!=Running

# View recent events
kubectl get events --sort-by='.lastTimestamp' | tail -20
```

### 4. Smoke Testing

Execute critical user paths:
- **Authentication Flow**: Test login/logout/session management
- **Core Features**: Verify primary user workflows
- **Data Persistence**: Confirm CRUD operations work
- **Error Handling**: Test graceful failure scenarios
- **Authorization**: Verify role-based access control

### 5. Performance Baseline

Establish performance metrics:
```bash
# API response times
curl -w "@curl-format.txt" -o /dev/null -s http://service:port/endpoint

# Load testing (if tools available)
# k6, Artillery, or similar
k6 run load-test.js

# Resource utilization
kubectl top pods -n <namespace>
kubectl top nodes
```

### 6. Security Validation

Run security checks:
```bash
# Dependency vulnerabilities
cargo audit  # Rust
npm audit    # Node.js
pip-audit    # Python

# Container image scanning (if available)
trivy image <image:tag>

# Secrets validation (no secrets in code)
git secrets --scan
```

### 7. Documentation Review

Verify documentation is complete:
- [ ] API documentation updated
- [ ] Deployment guide current
- [ ] Configuration changes documented
- [ ] Breaking changes noted
- [ ] Changelog updated

## Reporting

### Integration Summary Report

Create a comprehensive report:

```markdown
# Integration Report - [Date]

## PR Merge Summary
- Total PRs merged: [COUNT]
- Merge conflicts: [COUNT] (list them)
- Failed merges: [COUNT] (list them)

## Test Results
### Backend Tests
- ✅ Unit tests: [PASS/TOTAL]
- ✅ Integration tests: [PASS/TOTAL]

### Frontend Tests (if applicable)
- ✅ E2E tests: [PASS/TOTAL]
- ✅ Component tests: [PASS/TOTAL]

### API Tests
- ✅ Contract tests: [PASS/TOTAL]
- ✅ Health checks: [SERVICES_HEALTHY/TOTAL_SERVICES]

## Deployment Readiness
- ✅ Configuration validated
- ✅ Migrations applied
- ✅ Health checks passing
- ✅ No critical vulnerabilities
- ✅ Performance within acceptable range

## Issues Found
[List any issues discovered during integration]

## Recommendations
[Any recommendations for deployment or follow-up work]

## Deployment Checklist
- [ ] All PRs merged successfully
- [ ] All tests passing
- [ ] Health checks green
- [ ] Documentation updated
- [ ] Team notified
- [ ] Ready for production deployment
```

### Commit and PR

After completing integration:
```bash
# Commit the integration report
git add docs/integration-report.md
git commit -m "chore: integration verification complete"

# Create summary PR if needed
gh pr create --title "Integration Verification Complete" \
  --body "All tasks integrated and tested. See integration-report.md for details."
```

## Success Criteria

✅ **All prerequisite tasks completed**: All dependent task PRs merged
✅ **Tests passing**: Integration test suite passes with 100% success rate
✅ **Services healthy**: All service health endpoints returning 200 OK
✅ **No critical issues**: Zero critical bugs or security vulnerabilities
✅ **Documentation current**: All docs updated and accurate
✅ **Deployment validated**: Configurations verified and ready for production
✅ **Performance acceptable**: Response times and resource usage within limits
✅ **Integration report created**: Comprehensive report documenting all validation

## Error Handling

If any checks fail:
1. Document the failure clearly
2. Attempt automated remediation if possible
3. Create detailed error report with reproduction steps
4. Escalate critical issues immediately
5. Do not proceed to deployment until all critical issues resolved

## Notes

- This task may take several hours depending on project size
- Expect some issues during initial integration - this is normal
- Be thorough but pragmatic - not every minor warning needs blocking
- Focus on critical path and user-facing functionality
- When in doubt, err on the side of caution

