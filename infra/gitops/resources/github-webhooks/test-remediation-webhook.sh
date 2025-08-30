#!/bin/bash
# Test script for PR comment remediation webhook

set -e

echo "🧪 Testing PR Comment Remediation Webhook"

# Function to send test webhook
send_webhook() {
  local payload="$1"
  local webhook_url="http://github-eventsource-svc.argo.svc.cluster.local:12000/github/webhook"
  
  echo "📡 Sending webhook payload to: $webhook_url"
  
  kubectl run webhook-test --rm -i --tty \
    --image=curlimages/curl:latest \
    --restart=Never \
    -- curl -X POST \
      -H "Content-Type: application/json" \
      -H "X-GitHub-Event: issue_comment" \
      -H "X-GitHub-Delivery: test-$(date +%s)" \
      --data "$payload" \
      "$webhook_url"
}

# Test payload with PR comment containing feedback marker
FEEDBACK_PAYLOAD='{
  "action": "created",
  "issue": {
    "number": 42,
    "state": "open",
    "pull_request": {
      "url": "https://api.github.com/repos/5dlabs/cto/pulls/42"
    },
    "labels": [
      {
        "name": "task-1",
        "color": "ff0000",
        "description": "Task 1 implementation"
      }
    ]
  },
  "comment": {
    "id": 123456789,
    "body": "## 🔴 Required Changes\n\n**Issue Type**: Bug\n**Severity**: High\n\n### Description\nThe implementation is missing error handling for edge cases.\n\n### Expected Behavior\nAll edge cases should be handled gracefully.\n\n### Actual Behavior\nApplication crashes on invalid input.",
    "user": {
      "login": "5DLabs-Tess"
    }
  },
  "repository": {
    "clone_url": "https://github.com/5dlabs/cto.git",
    "name": "cto"
  },
  "sender": {
    "login": "5DLabs-Tess"
  }
}'

echo "🎯 Test 1: Valid feedback comment on PR with task label"
echo "Payload includes:"
echo "- Action: created"
echo "- Issue: #42 (open)"
echo "- Labels: task-1"
echo "- Comment author: 5DLabs-Tess"
echo "- Comment body: Contains '🔴 Required Changes'"
echo ""

# Send the webhook
send_webhook "$FEEDBACK_PAYLOAD"

echo ""
echo "✅ Webhook sent! Checking for CodeRun creation..."

# Wait a moment for processing
sleep 5

# Check if CodeRun was created
echo "🔍 Looking for CodeRuns in agent-platform namespace..."
kubectl get coderuns -n agent-platform -l trigger-type=comment-feedback --no-headers | tail -5

echo ""
echo "🔍 Checking sensor logs..."
kubectl logs -n argo deployment/pr-comment-remediation-sensor-86mp8 --tail=10 || true

echo ""
echo "🎉 Test completed!"