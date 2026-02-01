#!/usr/bin/env bash
# Gather Linear OAuth credentials from Linear UI

set -eo pipefail

# App ID to Agent mapping (extracted from Linear UI)
declare -a APPS=(
    "vex:763246ef-6fd6-40e5-93df-043664310270"
    "atlas:6c61e4ab-a8a1-4f08-8701-3859e9163690"
    "blaze:bee74dcc-4136-466e-a4b1-66fa04045f08"
    "bolt:7eb037ea-519f-45fe-ae9e-c4d7e04cd60c"
    "cipher:6f7ae61f-40c3-4ecb-835c-cfe6b9df7524"
    "cleo:066673cd-00c3-45c9-96c7-6cbad23cd86b"
    "grizz:0a043c52-6749-4787-a4ae-3523718c5c51"
    "morgan:b4fc9b98-dd73-48b4-a9a5-4dfd804a1361"
    "nova:80cf4b11-f092-4695-961d-bc8fb58b99a0"
    "rex:e6a8e612-89fb-4c91-a745-3f5359e0bf27"
    "spark:e7d739a1-6aa5-4ebc-82d8-0ffbe33665d5"
    "tap:cf1749d7-4782-45dd-81b6-0b9d614326f4"
    "tess:0b459c83-023b-4e3d-b8ab-126338d4caba"
)

echo "Gathering Linear OAuth credentials for ${#APPS[@]} agents..."
echo ""

for entry in "${APPS[@]}"; do
    agent="${entry%%:*}"
    app_id="${entry##*:}"
    
    echo "=== $agent ==="
    echo "App ID: $app_id"
    
    # Navigate to the app page
    clawdbot browser --browser-profile linear navigate "https://linear.app/jonathonfritz/settings/api/applications/${app_id}" 2>/dev/null
    sleep 2
    
    # Get snapshot and extract client_id
    SNAPSHOT=$(clawdbot browser --browser-profile linear snapshot 2>/dev/null)
    CLIENT_ID=$(echo "$SNAPSHOT" | grep -A1 "Client ID" | grep -oE "[a-f0-9]{32}" | head -1)
    
    if [ -n "$CLIENT_ID" ]; then
        echo "Client ID: $CLIENT_ID"
        
        # Check if 1Password item exists
        ITEM_NAME="Linear ${agent^} OAuth"
        if op item get "$ITEM_NAME" --vault "Automation" &>/dev/null 2>&1; then
            echo "1Password item exists: $ITEM_NAME"
        else
            echo "Need to create: $ITEM_NAME"
        fi
    else
        echo "Could not extract client_id"
    fi
    echo ""
done
