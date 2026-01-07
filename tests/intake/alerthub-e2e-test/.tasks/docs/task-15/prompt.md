# Task 15: End-to-End Integration Testing

**Agent**: tess | **Language**: text

## Role

You are a Senior QA Engineer with expertise in test automation and quality assurance implementing Task 15.

## Goal

Implement comprehensive E2E tests covering the full notification flow across all services and clients

## Requirements

1. Setup E2E test framework:
   - Create e2e-tests/ directory
   - Install Playwright for web testing
   - Install Detox for mobile testing
   - Install Spectron for desktop testing

2. Create test data fixtures:
   - Create test tenant, users, integrations, rules
   - Seed database with test data
   - Create mock external services (Slack, Discord)

3. Implement E2E test scenarios:

   Scenario 1: Submit notification via Web Console
   - Login to web console
   - Navigate to notifications page
   - Submit new notification with channel = Slack
   - Verify notification appears in feed
   - Verify notification is delivered to Slack (mock)
   - Verify delivery status updates in real-time (WebSocket)

   Scenario 2: Configure integration and rule
   - Login to web console
   - Navigate to integrations page
   - Create new Slack integration
   - Navigate to rules page
   - Create rule: if severity = critical, route to Slack
   - Submit critical notification
   - Verify notification is routed to Slack integration

   Scenario 3: Receive push notification on mobile
   - Submit notification via API
   - Verify push notification is received on mobile device (mock FCM)
   - Tap notification to open app
   - Verify app navigates to notification detail screen

   Scenario 4: Desktop notification flow
   - Submit notification via API
   - Verify desktop notification is shown (native)
   - Verify system tray badge count updates
   - Click notification to open main window
   - Verify notification is highlighted

   Scenario 5: Batch notification submission
   - Submit batch of 100 notifications via API
   - Verify all notifications are processed
   - Verify delivery status for each notification
   - Check metrics endpoint for throughput

   Scenario 6: Failed delivery retry
   - Create integration with invalid webhook URL
   - Submit notification
   - Verify delivery fails
   - Verify retry attempts (3 times with exponential backoff)
   - Verify final status = failed
   - Verify dead letter queue contains notification

4. Implement performance tests:
   - Load test: 10,000 notifications/minute sustained for 5 minutes
   - Stress test: Increase load until failure
   - Spike test: Sudden burst of 50,000 notifications
   - Verify response times stay within SLA (< 100ms p95)

5. Implement security tests:
   - Test JWT authentication (invalid token, expired token)
   - Test RBAC (viewer cannot create notifications)
   - Test rate limiting (exceed limit, verify 429 response)
   - Test input validation (XSS, SQL injection attempts)

6. Create CI/CD pipeline for E2E tests:
   - Run E2E tests on every PR
   - Run performance tests nightly
   - Generate test report with coverage
   - Fail build if tests fail

7. Document test results:
   - Create test report with pass/fail status
   - Include screenshots/videos of test runs
   - Document known issues and workarounds

## Acceptance Criteria

1. Run all E2E test scenarios and verify pass
2. Verify performance tests meet SLA targets
3. Verify security tests catch vulnerabilities
4. Run tests in CI/CD pipeline
5. Verify test coverage > 80% for critical paths
6. Manual smoke test after deployment
7. Verify monitoring dashboards show expected metrics during tests
8. Test rollback procedure (deploy previous version, verify functionality)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-15): End-to-End Integration Testing`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 11, 12, 13
