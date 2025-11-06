#!/bin/bash
# Fix existing GitHub Project to add agent-based Status options
# This adds agent names to the Status field for existing projects

set -euo pipefail

PROJECT_NUMBER="${1:-58}"
ORG="${2:-5dlabs}"

echo "🔧 Fixing project #$PROJECT_NUMBER in $ORG..."

# Get project ID
PROJECT_ID=$(gh api graphql -f query="
  query {
    organization(login: \"$ORG\") {
      projectV2(number: $PROJECT_NUMBER) {
        id
      }
    }
  }
" --jq '.data.organization.projectV2.id')

echo "✅ Project ID: $PROJECT_ID"

# Get Status field ID
STATUS_FIELD=$(gh api graphql -f query="
  query {
    node(id: \"$PROJECT_ID\") {
      ... on ProjectV2 {
        fields(first: 20) {
          nodes {
            ... on ProjectV2SingleSelectField {
              id
              name
              options {
                id
                name
              }
            }
          }
        }
      }
    }
  }
" --jq '.data.node.fields.nodes[] | select(.name == "Status")')

if [[ -z "$STATUS_FIELD" ]]; then
  echo "❌ Status field not found!"
  exit 1
fi

FIELD_ID=$(echo "$STATUS_FIELD" | jq -r '.id')
EXISTING_OPTIONS=$(echo "$STATUS_FIELD" | jq -r '.options[] | .name')

echo "✅ Found Status field: $FIELD_ID"
echo "📋 Existing options:"
echo "$EXISTING_OPTIONS"
echo ""

# Define agent names to add
AGENT_OPTIONS=(
  "Rex (Implementation)"
  "Blaze (Frontend)"
  "Cleo (Quality)"
  "Cipher (Security)"
  "Tess (QA)"
  "Atlas (Integration)"
  "Bolt (Deployment)"
)

COLORS=("RED" "ORANGE" "YELLOW" "GREEN" "BLUE" "PURPLE" "PINK")

echo "Adding agent-based status options..."
for i in "${!AGENT_OPTIONS[@]}"; do
  OPTION_NAME="${AGENT_OPTIONS[$i]}"
  COLOR="${COLORS[$i]}"
  
  # Check if option already exists
  if echo "$EXISTING_OPTIONS" | grep -q "^${OPTION_NAME}$"; then
    echo "  ✅ '$OPTION_NAME' already exists"
    continue
  fi
  
  echo "  ➕ Adding '$OPTION_NAME' ($COLOR)..."
  
  # Get all current options to preserve them
  CURRENT_OPTIONS=$(echo "$STATUS_FIELD" | jq -c '[.options[] | {name: .name, color: "GRAY", description: .name}]')
  
  # Add new option
  NEW_OPTIONS=$(echo "$CURRENT_OPTIONS" | jq --arg name "$OPTION_NAME" --arg color "$COLOR" \
    '. += [{name: $name, color: $color, description: $name}]')
  
  # Update field with all options
  RESULT=$(gh api graphql -f query="
    mutation {
      updateProjectV2Field(input: {
        fieldId: \"$FIELD_ID\"
        singleSelectOptions: $NEW_OPTIONS
      }) {
        projectV2Field {
          ... on ProjectV2SingleSelectField {
            id
            name
          }
        }
      }
    }
  " 2>&1)
  
  if echo "$RESULT" | jq -e '.errors' >/dev/null 2>&1; then
    echo "  ❌ Failed to add '$OPTION_NAME':"
    echo "$RESULT" | jq '.errors'
  else
    echo "  ✅ Added '$OPTION_NAME'"
    # Update STATUS_FIELD for next iteration
    STATUS_FIELD=$(gh api graphql -f query="
      query {
        node(id: \"$FIELD_ID\") {
          ... on ProjectV2SingleSelectField {
            id
            name
            options {
              id
              name
            }
          }
        }
      }
    " --jq '.data.node')
  fi
done

echo ""
echo "✅ All agent options added!"
echo ""
echo "📋 Final Status field options:"
gh api graphql -f query="
  query {
    node(id: \"$FIELD_ID\") {
      ... on ProjectV2SingleSelectField {
        options {
          name
        }
      }
    }
  }
" --jq '.data.node.options[] | "   - \(.name)"'

echo ""
echo "🎯 Next: Open the project board and columns should auto-appear!"
echo "   URL: https://github.com/orgs/$ORG/projects/$PROJECT_NUMBER"


