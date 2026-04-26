#!/usr/bin/env bash
# linear-agent-prompt.sh - Send prompts to Linear agent sessions via GraphQL API
#
# Usage:
#   ./scripts/2026-01/linear-agent-prompt.sh <session-id> "Your prompt message here"
#   ./scripts/2026-01/linear-agent-prompt.sh --list-sessions CTOPA-841
#
# Examples:
#   # Send prompt to a known session ID (requires OAuth token)
#   ./scripts/2026-01/linear-agent-prompt.sh dbae4c02-041a-4525-ac64-29cb5990dacd "Please continue"
#   
#   # List sessions for an issue (works with API key)
#   ./scripts/2026-01/linear-agent-prompt.sh --list-sessions CTOPA-841
#
# Environment:
#   LINEAR_API_KEY   - For read-only operations (list sessions, query)
#   LINEAR_OAUTH_TOKEN - Required for sending prompts (user identity needed)
#
# Authentication Note:
#   The agentActivityCreatePrompt mutation requires OAuth authentication because
#   prompts must be associated with a human user identity. API keys don't work.
#   
#   To get an OAuth token:
#   1. Go to https://linear.app/settings/api and create a personal API key (this is OAuth)
#   2. Or use the Linear OAuth flow with your app credentials
#
# Getting OAuth Token:
#   Linear's "Personal API keys" in settings are actually OAuth tokens that work
#   for user-authenticated operations. Create one at https://linear.app/settings/api

set -euo pipefail

# Get API key from environment or 1Password (for read operations)
LINEAR_API_KEY="${LINEAR_API_KEY:-$(op item get "Linear API Credentials" --format json 2>/dev/null | jq -r '.fields[] | select(.label == "LINEAR_API_KEY" or .label == "credential") | .value' 2>/dev/null | head -1 || echo "")}"

# OAuth token for mutations (user-authenticated operations)
LINEAR_OAUTH_TOKEN="${LINEAR_OAUTH_TOKEN:-}"

# Use OAuth token if available, otherwise fall back to API key
get_auth_header() {
    local operation="${1:-read}"
    
    if [[ "$operation" == "mutation" ]] && [[ -z "$LINEAR_OAUTH_TOKEN" ]]; then
        echo "Error: LINEAR_OAUTH_TOKEN required for mutations (sending prompts)." >&2
        echo "" >&2
        echo "The agentActivityCreatePrompt API requires user authentication." >&2
        echo "API keys don't carry user identity, so OAuth is required." >&2
        echo "" >&2
        echo "To get an OAuth token:" >&2
        echo "1. Go to https://linear.app/settings/api" >&2
        echo "2. Create a 'Personal API key' (these are OAuth tokens)" >&2
        echo "3. Export it: export LINEAR_OAUTH_TOKEN='lin_oauth_...'" >&2
        exit 1
    fi
    
    if [[ -n "$LINEAR_OAUTH_TOKEN" ]]; then
        echo "$LINEAR_OAUTH_TOKEN"
    elif [[ -n "$LINEAR_API_KEY" ]]; then
        echo "$LINEAR_API_KEY"
    else
        echo "Error: No Linear credentials found. Set LINEAR_API_KEY or LINEAR_OAUTH_TOKEN." >&2
        exit 1
    fi
}

LINEAR_API_URL="https://api.linear.app/graphql"

# Function to make GraphQL requests (for read operations)
graphql_request() {
    local query="$1"
    local variables="${2:-}"
    
    local auth_token
    auth_token=$(get_auth_header "read")
    
    # Create a proper JSON payload using jq
    local payload
    if [[ -z "$variables" ]] || [[ "$variables" == "{}" ]]; then
        payload=$(jq -n --arg q "$query" '{query: $q}')
    else
        payload=$(jq -n --arg q "$query" --argjson v "$variables" '{query: $q, variables: $v}')
    fi
    
    curl -s -X POST "$LINEAR_API_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: $auth_token" \
        -d "$payload"
}

# Function to list agent sessions for an issue
list_sessions() {
    local issue_identifier="$1"
    
    echo "Fetching agent sessions for issue: $issue_identifier"
    
    # First, get all recent agent sessions and filter by issue identifier
    local query='
    query RecentAgentSessions {
      agentSessions(first: 50) {
        nodes {
          id
          status
          createdAt
          updatedAt
          issue {
            id
            identifier
            title
          }
          activities(first: 5) {
            nodes {
              id
              createdAt
              content {
                ... on AgentActivityThoughtContent { type body }
                ... on AgentActivityActionContent { type action parameter result }
                ... on AgentActivityResponseContent { type body }
                ... on AgentActivityPromptContent { type body }
                ... on AgentActivityElicitationContent { type body }
                ... on AgentActivityErrorContent { type body }
              }
            }
          }
        }
      }
    }'
    
    local result
    result=$(graphql_request "$query" "{}")
    
    if echo "$result" | jq -e '.errors' > /dev/null 2>&1; then
        echo "Error:"
        echo "$result" | jq '.errors'
        exit 1
    fi
    
    # Filter for the specific issue
    echo "$result" | jq --arg id "$issue_identifier" '.data.agentSessions.nodes[] | select(.issue.identifier == $id)'
}

# Function to get session by partial ID
get_session_id() {
    local partial_id="$1"
    
    # First try as full UUID
    local query='
    query AgentSession($id: String!) {
      agentSession(id: $id) {
        id
        status
        issue {
          identifier
          title
        }
      }
    }'
    
    local result
    result=$(graphql_request "$query" "{\"id\": \"$partial_id\"}")
    
    if echo "$result" | jq -e '.data.agentSession.id' > /dev/null 2>&1; then
        echo "$result" | jq -r '.data.agentSession.id'
        return 0
    fi
    
    # Search recent sessions for matching partial ID
    local search_query='
    query RecentAgentSessions {
      agentSessions(first: 50) {
        nodes {
          id
          status
          issue { identifier }
        }
      }
    }'
    
    result=$(graphql_request "$search_query" "{}")
    local full_id
    full_id=$(echo "$result" | jq -r --arg partial "$partial_id" '.data.agentSessions.nodes[] | select(.id | startswith($partial)) | .id' | head -1)
    
    if [[ -n "$full_id" ]]; then
        echo "$full_id"
        return 0
    fi
    
    echo ""
}

# Function to send a prompt to an agent session
send_prompt() {
    local session_id="$1"
    local message="$2"
    
    # Get OAuth token (required for mutations)
    local auth_token
    auth_token=$(get_auth_header "mutation")
    
    echo "Sending prompt to session: $session_id"
    echo "Message: $message"
    echo ""
    
    local query='mutation AgentActivityCreatePrompt($input: AgentActivityCreatePromptInput!) { agentActivityCreatePrompt(input: $input) { success agentActivity { id createdAt } } }'
    
    # Build the full payload directly with jq
    local payload
    payload=$(jq -n \
        --arg query "$query" \
        --arg sessionId "$session_id" \
        --arg body "$message" \
        '{
            query: $query,
            variables: {
                input: {
                    agentSessionId: $sessionId,
                    content: {
                        type: "prompt",
                        body: $body
                    }
                }
            }
        }')
    
    local result
    result=$(curl -s -X POST "$LINEAR_API_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: $auth_token" \
        -d "$payload")
    
    if echo "$result" | jq -e '.errors' > /dev/null 2>&1; then
        echo "Error:"
        echo "$result" | jq '.errors'
        exit 1
    fi
    
    if echo "$result" | jq -e '.data.agentActivityCreatePrompt.success == true' > /dev/null 2>&1; then
        echo "✓ Prompt sent successfully!"
        echo "$result" | jq '.data.agentActivityCreatePrompt'
    else
        echo "Failed to send prompt:"
        echo "$result" | jq '.'
        exit 1
    fi
}

# Main
case "${1:-}" in
    --list-sessions)
        if [[ -z "${2:-}" ]]; then
            echo "Usage: $0 --list-sessions <issue-identifier>"
            exit 1
        fi
        list_sessions "$2"
        ;;
    --help|-h)
        head -20 "$0" | tail -18
        ;;
    *)
        if [[ -z "${1:-}" ]] || [[ -z "${2:-}" ]]; then
            echo "Usage: $0 <session-id> \"prompt message\""
            echo "       $0 --list-sessions <issue-identifier>"
            exit 1
        fi
        
        session_id="$1"
        message="$2"
        
        # If session_id looks like a short ID, try to resolve it
        if [[ ${#session_id} -lt 36 ]]; then
            echo "Resolving short session ID: $session_id"
            full_id=$(get_session_id "$session_id")
            if [[ -n "$full_id" ]]; then
                echo "Found full session ID: $full_id"
                session_id="$full_id"
            else
                echo "Warning: Could not resolve short ID, using as-is"
            fi
        fi
        
        send_prompt "$session_id" "$message"
        ;;
esac
