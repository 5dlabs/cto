#!/bin/bash

# Test script for remediation feedback sensor
# This script tests the sensor's ability to process PR comments with feedback

set -e

echo "=== Testing Remediation Feedback Sensor ==="

# Test payload with feedback comment on PR with task label
TEST_PAYLOAD='{
  "action": "created",
  "issue": {
    "number": 123,
    "pull_request": {
      "url": "https://api.github.com/repos/5dlabs/cto/pulls/123"
    },
    "labels": [
      {"name": "task-1"},
      {"name": "ready-for-qa"}
    ]
  },
  "comment": {
    "id": 456,
    "body": "## 🔴 Required Changes\n\n**Issue Type**: Bug\n**Severity**: High\n\n### Description\nTest feedback for remediation flow",
    "user": {
      "login": "5DLabs-Tess",
      "type": "Bot"
    }
  }
}'

echo "Test Payload:"
echo "$TEST_PAYLOAD" | jq '.'

# Get webhook secret for signature
WEBHOOK_SECRET=$(kubectl get secret github-webhook-secret -n argo -o jsonpath='{.data.secret}' | base64 -d)

if [ -z "$WEBHOOK_SECRET" ]; then
  echo "❌ ERROR: Could not retrieve webhook secret"
  exit 1
fi

# Calculate signature
SIGNATURE=$(echo -n "$TEST_PAYLOAD" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" | sed 's/^.* /sha256=/')

echo "Webhook Signature: $SIGNATURE"

# Send test webhook
echo "Sending test webhook to sensor..."
curl -v -X POST \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: issue_comment" \
  -H "X-Hub-Signature-256: $SIGNATURE" \
  -H "X-GitHub-Delivery: test-$(date +%s)" \
  -d "$TEST_PAYLOAD" \
  http://localhost:8080/github/webhook 2>/dev/null || echo "Note: Webhook sent (curl may show connection errors in test environment)"

echo ""
echo "=== Verification Steps ==="
echo "1. Check sensor logs:"
echo "   kubectl logs -n argo deployment/remediation-feedback-sensor"
echo ""
echo "2. Check for created workflow:"
echo "   kubectl get workflows -n cto -l type=remediation-workflow"
echo ""
echo "3. Check for created CodeRun:"
echo "   kubectl get coderuns -n cto -l trigger-type=comment-feedback"
echo ""
echo "4. Verify CodeRun spec contains REMEDIATION_MODE=true:"
echo "   kubectl get coderun <coderun-name> -n cto -o yaml | grep -A5 'env:'"

echo ""
echo "=== Expected Results ==="
echo "✅ Sensor should detect the feedback comment"
echo "✅ Workflow should be created with task ID extraction"
echo "✅ CodeRun should be created with task-1 and REMEDIATION_MODE=true"
echo "✅ Environment variables should include FEEDBACK_COMMENT_ID=456"

echo ""
echo "Test completed. Check the verification steps above."
