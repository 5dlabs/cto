#!/usr/bin/env bash
set -euo pipefail

# Twitter/X Authentication Setup for Research Agent
# This script helps you extract Twitter session cookies and store them in Kubernetes

echo "=== Twitter/X Authentication Setup for Research Agent ==="
echo
echo "You need to extract two cookies from your browser:"
echo "1. auth_token - Long-lived authentication token (~40-50 characters)"
echo "2. ct0 - CSRF token (~32 characters)"
echo
echo "Steps to extract cookies:"
echo "1. Open your browser and go to https://x.com (make sure you're logged in)"
echo "2. Open Developer Tools (F12 or Cmd+Option+I)"
echo "3. Go to Application tab → Cookies → https://x.com"
echo "4. Find 'auth_token' and copy its value"
echo "5. Find 'ct0' and copy its value"
echo
echo "Press Enter when you're ready to continue..."
read -r

# Prompt for auth_token
echo
echo -n "Paste your auth_token: "
read -r AUTH_TOKEN

# Validate auth_token
if [ -z "$AUTH_TOKEN" ] || [ "$AUTH_TOKEN" = "placeholder" ]; then
    echo "ERROR: auth_token is required and cannot be 'placeholder'"
    exit 1
fi

if [ ${#AUTH_TOKEN} -lt 30 ]; then
    echo "WARNING: auth_token seems too short (${#AUTH_TOKEN} chars). Twitter tokens are usually 40-50 characters."
    echo -n "Continue anyway? (y/N): "
    read -r CONTINUE
    if [ "$CONTINUE" != "y" ] && [ "$CONTINUE" != "Y" ]; then
        exit 1
    fi
fi

# Prompt for ct0
echo -n "Paste your ct0: "
read -r CT0

# Validate ct0
if [ -z "$CT0" ] || [ "$CT0" = "placeholder" ]; then
    echo "ERROR: ct0 is required and cannot be 'placeholder'"
    exit 1
fi

if [ ${#CT0} -lt 20 ]; then
    echo "WARNING: ct0 seems too short (${#CT0} chars). Twitter CSRF tokens are usually 32 characters."
    echo -n "Continue anyway? (y/N): "
    read -r CONTINUE
    if [ "$CONTINUE" != "y" ] && [ "$CONTINUE" != "Y" ]; then
        exit 1
    fi
fi

echo
echo "Credentials collected:"
echo "  auth_token: ${AUTH_TOKEN:0:10}...${AUTH_TOKEN: -10} (${#AUTH_TOKEN} chars)"
echo "  ct0: ${CT0:0:10}...${CT0: -10} (${#CT0} chars)"
echo

# Option 1: Update Kubernetes secret directly
echo "=== Updating Kubernetes Secret ==="
echo "This will update the research-twitter-secrets secret in the cto namespace"
echo

# Create the secret
kubectl create secret generic research-twitter-secrets \
    --from-literal=TWITTER_AUTH_TOKEN="$AUTH_TOKEN" \
    --from-literal=TWITTER_CT0="$CT0" \
    --namespace=cto \
    --dry-run=client -o yaml | kubectl apply -f -

echo "✓ Kubernetes secret updated successfully"
echo

# Verify
echo "=== Verifying Secret ==="
AUTH_TOKEN_LEN=$(kubectl get secret research-twitter-secrets -n cto -o jsonpath='{.data.TWITTER_AUTH_TOKEN}' | base64 -d | wc -c | tr -d ' ')
CT0_LEN=$(kubectl get secret research-twitter-secrets -n cto -o jsonpath='{.data.TWITTER_CT0}' | base64 -d | wc -c | tr -d ' ')

echo "  auth_token length: $AUTH_TOKEN_LEN chars"
echo "  ct0 length: $CT0_LEN chars"

if [ "$AUTH_TOKEN_LEN" -gt 30 ] && [ "$CT0_LEN" -gt 20 ]; then
    echo "✓ Credentials look valid"
else
    echo "⚠ Credentials may be too short, but they've been stored"
fi

echo
echo "=== Next Steps ==="
echo "1. Test the research agent:"
echo "   kubectl create job --from=cronjob/research-poller research-test -n cto"
echo "   kubectl logs -n cto -l job-name=research-test --follow"
echo
echo "2. (Optional) Store credentials in 1Password for backup:"
echo "   op item create --vault Personal --category Login \\"
echo "     --title 'Twitter/X Research Auth' \\"
echo "     'auth_token[password]=$AUTH_TOKEN' \\"
echo "     'ct0[password]=$CT0'"
echo
echo "3. The research-poller CronJob will automatically use these credentials"
echo "   on the next scheduled run (every 5 minutes)"
echo
echo "Done! 🎉"

