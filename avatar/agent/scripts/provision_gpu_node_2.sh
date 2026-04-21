#!/bin/bash
# OVH GPU Node Provisioning Script - 2nd Node
# Provisions a t2-45 (V100S 32GB) in GRA9 for Hunyuan avatar rendering

set -euo pipefail

PROJECT_ID="6093a51de65b458e8b20a7c570a4f2c1"
REGION="gra9"
FLAVOR="t2-45"
IMAGE="Ubuntu 22.04"
INSTANCE_NAME="hunyuan-gpu-1"

# Load OVH credentials from 1Password
echo "Loading OVH credentials..."
export OVH_AK=$(op read "op://Automation/OVH CA API/application_key")
export OVH_AS=$(op read "op://Automation/OVH CA API/application_secret")
export OVH_CK=$(op read "op://Automation/OVH CA API/consumer_key")

# OVH API signer function
ovh_call() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local ts
  ts=$(curl -s https://ca.api.ovh.com/1.0/auth/time)
  local sig
  sig="\$1\$$(printf '%s+%s+%s+https://ca.api.ovh.com/1.0%s+%s+%s' \
    "$OVH_AS" "$OVH_CK" "$method" "$path" "$body" "$ts" | sha1sum | awk '{print $1}')"
  curl -sS -X "$method" "https://ca.api.ovh.com/1.0$path" \
    -H "X-Ovh-Application: $OVH_AK" \
    -H "X-Ovh-Consumer: $OVH_CK" \
    -H "X-Ovh-Timestamp: $ts" \
    -H "X-Ovh-Signature: $sig" \
    -H "Content-Type: application/json" \
    ${body:+--data "$body"}
}

# Get SSH key from 1Password
echo "Loading SSH key..."
SSH_KEY=$(op read "op://Automation/OVH GRA9 GPU SSH/public_key" 2>/dev/null || echo "")

if [ -z "$SSH_KEY" ]; then
  echo "ERROR: SSH key not found in 1Password"
  exit 1
fi

echo "Provisioning GPU node: $INSTANCE_NAME"
echo "Region: $REGION, Flavor: $FLAVOR, Image: $IMAGE"

# Check for existing instance
echo "Checking for existing instance..."
EXISTING=$(ovh_call GET "/cloud/project/$PROJECT_ID/instance" 2>/dev/null | jq -r ".[] | select(.name == \"$INSTANCE_NAME\") | .id" 2>/dev/null || echo "")

if [ -n "$EXISTING" ]; then
  echo "Instance $INSTANCE_NAME already exists (ID: $EXISTING)"
  echo "Checking status..."
  STATUS=$(ovh_call GET "/cloud/project/$PROJECT_ID/instance/$EXISTING" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "UNKNOWN")
  echo "Current status: $STATUS"
  
  if [ "$STATUS" = "ACTIVE" ]; then
    echo "Instance is already active. Getting IP..."
    IP=$(ovh_call GET "/cloud/project/$PROJECT_ID/instance/$EXISTING" 2>/dev/null | jq -r '.ipAddresses[] | select(.type == "public") | .ip' 2>/dev/null || echo "")
    echo "Instance IP: $IP"
    echo "INSTANCE_ID=$EXISTING"
    echo "INSTANCE_IP=$IP"
    exit 0
  elif [ "$STATUS" = "ERROR" ]; then
    echo "Instance in ERROR state. Deleting and recreating..."
    ovh_call DELETE "/cloud/project/$PROJECT_ID/instance/$EXISTING" 2>/dev/null || true
    sleep 10
  else
    echo "Waiting for instance to become ACTIVE..."
    for i in {1..30}; do
      sleep 10
      STATUS=$(ovh_call GET "/cloud/project/$PROJECT_ID/instance/$EXISTING" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "UNKNOWN")
      echo "Status: $STATUS"
      if [ "$STATUS" = "ACTIVE" ]; then
        IP=$(ovh_call GET "/cloud/project/$PROJECT_ID/instance/$EXISTING" 2>/dev/null | jq -r '.ipAddresses[] | select(.type == "public") | .ip' 2>/dev/null || echo "")
        echo "Instance is ACTIVE. IP: $IP"
        echo "INSTANCE_ID=$EXISTING"
        echo "INSTANCE_IP=$IP"
        exit 0
      fi
    done
    echo "Timeout waiting for instance"
    exit 1
  fi
fi

# Get image ID
echo "Finding image ID for $IMAGE..."
IMAGE_RESP=$(ovh_call GET "/cloud/project/$PROJECT_ID/image" 2>/dev/null || echo "[]")
IMAGE_ID=$(echo "$IMAGE_RESP" | jq -r ".[] | select(.name == \"$IMAGE\") | .id" 2>/dev/null | head -1 || echo "")

if [ -z "$IMAGE_ID" ]; then
  echo "ERROR: Image '$IMAGE' not found"
  exit 1
fi
echo "Image ID: $IMAGE_ID"

# Get flavor ID
echo "Finding flavor ID for $FLAVOR..."
FLAVOR_RESP=$(ovh_call GET "/cloud/project/$PROJECT_ID/flavor?region=$REGION" 2>/dev/null || echo "[]")
FLAVOR_ID=$(echo "$FLAVOR_RESP" | jq -r ".[] | select(.name == \"$FLAVOR\") | .id" 2>/dev/null | head -1 || echo "")

if [ -z "$FLAVOR_ID" ]; then
  echo "ERROR: Flavor '$FLAVOR' not found in region $REGION"
  exit 1
fi
echo "Flavor ID: $FLAVOR_ID"

# Get SSH key ID
echo "Finding SSH key..."
KEY_RESP=$(ovh_call GET "/cloud/project/$PROJECT_ID/sshkey" 2>/dev/null || echo "[]")
KEY_ID=$(echo "$KEY_RESP" | jq -r ".[] | select(.publicKey == \"$SSH_KEY\") | .id" 2>/dev/null || echo "")

if [ -z "$KEY_ID" ]; then
  echo "SSH key not found, creating new key..."
  KEY_NAME="coder-gpu-key-$(date +%s)"
  KEY_CREATE_RESP=$(ovh_call POST "/cloud/project/$PROJECT_ID/sshkey" "{\"name\": \"$KEY_NAME\", \"publicKey\": \"$SSH_KEY\"}" 2>/dev/null || echo "{}")
  KEY_ID=$(echo "$KEY_CREATE_RESP" | jq -r '.id' 2>/dev/null || echo "")
  if [ -z "$KEY_ID" ] || [ "$KEY_ID" = "null" ]; then
    echo "ERROR: Failed to create SSH key"
    echo "Response: $KEY_CREATE_RESP"
    exit 1
  fi
  echo "Created SSH key: $KEY_ID"
else
  echo "Using existing SSH key: $KEY_ID"
fi

# Create instance
echo "Creating instance..."
BODY=$(cat <<EOF
{
  "name": "$INSTANCE_NAME",
  "region": "$REGION",
  "flavorId": "$FLAVOR_ID",
  "imageId": "$IMAGE_ID",
  "sshKeyId": "$KEY_ID",
  "monthlyBilling": false
}
EOF
)

echo "Request body: $BODY"

CREATE_RESP=$(ovh_call POST "/cloud/project/$PROJECT_ID/instance" "$BODY" 2>/dev/null || echo "{}")
INSTANCE_ID=$(echo "$CREATE_RESP" | jq -r '.id' 2>/dev/null || echo "")

if [ -z "$INSTANCE_ID" ] || [ "$INSTANCE_ID" = "null" ]; then
  echo "ERROR: Failed to create instance"
  echo "Response: $CREATE_RESP"
  exit 1
fi

echo "Instance created: $INSTANCE_ID"
echo "Waiting for ACTIVE status..."

# Wait for instance to be ready
for i in {1..60}; do
  sleep 10
  STATUS_RESP=$(ovh_call GET "/cloud/project/$PROJECT_ID/instance/$INSTANCE_ID" 2>/dev/null || echo "{}")
  STATUS=$(echo "$STATUS_RESP" | jq -r '.status' 2>/dev/null || echo "UNKNOWN")
  echo "Status: $STATUS"
  
  if [ "$STATUS" = "ACTIVE" ]; then
    IP=$(echo "$STATUS_RESP" | jq -r '.ipAddresses[] | select(.type == "public") | .ip' 2>/dev/null || echo "")
    echo "Instance is ACTIVE!"
    echo "Instance ID: $INSTANCE_ID"
    echo "Public IP: $IP"
    echo ""
    echo "INSTANCE_ID=$INSTANCE_ID"
    echo "INSTANCE_IP=$IP"
    exit 0
  elif [ "$STATUS" = "ERROR" ]; then
    echo "ERROR: Instance failed to start"
    exit 1
  fi
done

echo "ERROR: Timeout waiting for instance to become ACTIVE"
exit 1
