#!/usr/bin/env bash
# OpenMemory Integration Functions for Agents

# Configuration
MEMORY_API="${MEMORY_API:-http://openmemory:3000}"
MEMORY_TIMEOUT="${MEMORY_TIMEOUT:-5}"
AGENT_NAME="${AGENT_NAME:-unknown}"
TASK_ID="${TASK_ID:-unknown}"
SERVICE_NAME="${SERVICE_NAME:-unknown}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Core Memory Functions
# ============================================================================

# Query memories based on context
query_memory() {
  local query="$1"
  local limit="${2:-10}"
  local include_waypoints="${3:-true}"
  
  echo -e "${BLUE}🧠 Querying memory: ${query:0:50}...${NC}" >&2
  
  local response
  response=$(curl -s -X POST "${MEMORY_API}/memory/query" \
    --max-time "${MEMORY_TIMEOUT}" \
    -H "Content-Type: application/json" \
    -d "{
      \"query\": \"${query}\",
      \"agent\": \"${AGENT_NAME}\",
      \"task_id\": \"${TASK_ID}\",
      \"service\": \"${SERVICE_NAME}\",
      \"k\": ${limit},
      \"include_waypoints\": ${include_waypoints}
    }" 2>/dev/null)
  
  if [ $? -eq 0 ] && [ -n "$response" ]; then
    echo "$response"
    return 0
  else
    echo -e "${YELLOW}⚠️  Memory query failed or timed out${NC}" >&2
    echo "[]"
    return 1
  fi
}

# Store a new memory
add_memory() {
  local content="$1"
  local pattern_type="${2:-general}"
  local success="${3:-true}"
  local metadata="${4:-{}}"
  
  echo -e "${BLUE}💾 Storing memory: ${content:0:50}...${NC}" >&2
  
  # Build metadata JSON
  local full_metadata
  full_metadata=$(jq -n \
    --arg agent "${AGENT_NAME}" \
    --arg task "${TASK_ID}" \
    --arg service "${SERVICE_NAME}" \
    --arg pattern "${pattern_type}" \
    --argjson success "${success}" \
    --argjson meta "${metadata}" \
    '$meta + {
      agent: $agent,
      task_id: $task,
      service: $service,
      pattern_type: $pattern,
      success: $success,
      timestamp: (now | todate)
    }')
  
  local response
  response=$(curl -s -X POST "${MEMORY_API}/memory/add" \
    --max-time "${MEMORY_TIMEOUT}" \
    -H "Content-Type: application/json" \
    -d "{
      \"content\": \"${content}\",
      \"metadata\": ${full_metadata}
    }" 2>/dev/null)
  
  if [ $? -eq 0 ] && [ -n "$response" ]; then
    local memory_id
    memory_id=$(echo "$response" | jq -r '.id // "unknown"')
    echo -e "${GREEN}✅ Memory stored: ${memory_id}${NC}" >&2
    echo "$response"
    return 0
  else
    echo -e "${YELLOW}⚠️  Failed to store memory${NC}" >&2
    return 1
  fi
}

# Reinforce an existing memory
reinforce_memory() {
  local memory_id="$1"
  local amount="${2:-1}"
  
  echo -e "${BLUE}💪 Reinforcing memory: ${memory_id}${NC}" >&2
  
  local response
  response=$(curl -s -X POST "${MEMORY_API}/memory/reinforce" \
    --max-time "${MEMORY_TIMEOUT}" \
    -H "Content-Type: application/json" \
    -d "{
      \"id\": \"${memory_id}\",
      \"amount\": ${amount}
    }" 2>/dev/null)
  
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Memory reinforced${NC}" >&2
    return 0
  else
    echo -e "${YELLOW}⚠️  Failed to reinforce memory${NC}" >&2
    return 1
  fi
}

# ============================================================================
# Pattern-Specific Functions
# ============================================================================

# Check for similar errors and solutions
check_error_memory() {
  local error_message="$1"
  
  echo -e "${BLUE}🔍 Checking memory for similar errors...${NC}" >&2
  
  local memories
  memories=$(query_memory "error: ${error_message}" 20 true)
  
  if [ -n "$memories" ] && [ "$memories" != "[]" ]; then
    echo -e "${GREEN}📚 Found ${YELLOW}$(echo "$memories" | jq 'length')${GREEN} related memories${NC}" >&2
    
    # Extract solutions from memories
    echo "$memories" | jq -r '.[] | 
      select(.metadata.pattern_type == "error" and .metadata.success == true) |
      "• [\(.score | . * 100 | floor)%] \(.content | split("Solution:")[1] // .content)"' 2>/dev/null
    
    return 0
  else
    echo -e "${YELLOW}ℹ️  No similar errors found in memory${NC}" >&2
    return 1
  fi
}

# Load project context at task start
load_project_context() {
  echo -e "${BLUE}📖 Loading project context from memory...${NC}" >&2
  
  # Query multiple aspects of the project
  local tech_stack deps conventions errors
  
  tech_stack=$(query_memory "technology stack libraries frameworks ${SERVICE_NAME}" 10 false)
  deps=$(query_memory "dependencies packages ${SERVICE_NAME}" 10 false)
  conventions=$(query_memory "naming conventions patterns ${SERVICE_NAME}" 10 false)
  errors=$(query_memory "common errors failures ${SERVICE_NAME}" 10 false)
  
  # Build context summary
  local context=""
  
  if [ -n "$tech_stack" ] && [ "$tech_stack" != "[]" ]; then
    context="${context}## Technology Stack\n"
    context="${context}$(echo "$tech_stack" | jq -r '.[].content' | head -5)\n\n"
  fi
  
  if [ -n "$deps" ] && [ "$deps" != "[]" ]; then
    context="${context}## Dependencies\n"
    context="${context}$(echo "$deps" | jq -r '.[].content' | head -5)\n\n"
  fi
  
  if [ -n "$conventions" ] && [ "$conventions" != "[]" ]; then
    context="${context}## Conventions\n"
    context="${context}$(echo "$conventions" | jq -r '.[].content' | head -5)\n\n"
  fi
  
  if [ -n "$errors" ] && [ "$errors" != "[]" ]; then
    context="${context}## Known Issues\n"
    context="${context}$(echo "$errors" | jq -r '.[].content' | head -5)\n\n"
  fi
  
  if [ -n "$context" ]; then
    echo -e "${GREEN}✅ Loaded project context with $(echo -e "$context" | wc -l) insights${NC}" >&2
    echo -e "$context"
    return 0
  else
    echo -e "${YELLOW}ℹ️  No project context found in memory${NC}" >&2
    return 1
  fi
}

# Store successful pattern for reuse
store_success_pattern() {
  local pattern_name="$1"
  local solution="$2"
  local details="${3:-}"
  
  local content="Pattern: ${pattern_name}\nSolution: ${solution}"
  if [ -n "$details" ]; then
    content="${content}\nDetails: ${details}"
  fi
  
  add_memory "$content" "implementation" true "{\"reusable\": true}"
}

# Store error pattern to avoid
store_error_pattern() {
  local error="$1"
  local context="$2"
  local solution="${3:-}"
  
  local content="Error: ${error}\nContext: ${context}"
  if [ -n "$solution" ]; then
    content="${content}\nSolution: ${solution}"
    add_memory "$content" "error" true "{\"solved\": true}"
  else
    add_memory "$content" "error" false "{\"solved\": false}"
  fi
}

# ============================================================================
# Circuit Breaker Enhancement
# ============================================================================

# Enhanced circuit breaker with memory lookup
check_loop_with_memory() {
  local failed_command="$1"
  local fail_count="$2"
  
  if [ "$fail_count" -ge 3 ]; then
    echo -e "${RED}🔴 Loop detected! Checking memory for solutions...${NC}" >&2
    
    # Query memory for this specific error
    local solutions
    solutions=$(check_error_memory "$failed_command")
    
    if [ $? -eq 0 ] && [ -n "$solutions" ]; then
      echo -e "${GREEN}💡 Found potential solutions from memory:${NC}" >&2
      echo "$solutions"
      return 0
    else
      # Store this as an unsolved error
      store_error_pattern "$failed_command" "Repeated failure in loop" ""
      echo -e "${YELLOW}⚠️  No solutions in memory. Storing for future reference.${NC}" >&2
      return 1
    fi
  fi
  
  return 1
}

# ============================================================================
# Metrics Collection
# ============================================================================

# Track memory usage metrics
track_memory_metric() {
  local metric_type="$1"  # query_hit, query_miss, store_success, etc.
  local value="${2:-1}"
  
  # Send metric to monitoring system (if available)
  if [ -n "${METRICS_ENDPOINT:-}" ]; then
    curl -s -X POST "${METRICS_ENDPOINT}/metrics" \
      --max-time 1 \
      -H "Content-Type: application/json" \
      -d "{
        \"metric\": \"openmemory.${AGENT_NAME}.${metric_type}\",
        \"value\": ${value},
        \"timestamp\": $(date +%s),
        \"tags\": {
          \"agent\": \"${AGENT_NAME}\",
          \"task\": \"${TASK_ID}\",
          \"service\": \"${SERVICE_NAME}\"
        }
      }" 2>/dev/null || true
  fi
}

# ============================================================================
# Initialization
# ============================================================================

# Initialize memory connection at startup
init_memory() {
  echo -e "${BLUE}🔌 Initializing OpenMemory connection...${NC}" >&2
  
  # Test connection
  local health
  health=$(curl -s --max-time 2 "${MEMORY_API}/health" 2>/dev/null)
  
  if [ $? -eq 0 ] && [ -n "$health" ]; then
    echo -e "${GREEN}✅ OpenMemory connected at ${MEMORY_API}${NC}" >&2
    
    # Load project context
    local context
    context=$(load_project_context)
    if [ -n "$context" ]; then
      echo -e "${BLUE}📚 Project Context:${NC}" >&2
      echo -e "$context" >&2
    fi
    
    return 0
  else
    echo -e "${YELLOW}⚠️  OpenMemory not available. Running without memory augmentation.${NC}" >&2
    return 1
  fi
}

# Export functions for use in agent scripts
export -f query_memory
export -f add_memory
export -f reinforce_memory
export -f check_error_memory
export -f load_project_context
export -f store_success_pattern
export -f store_error_pattern
export -f check_loop_with_memory
export -f track_memory_metric
export -f init_memory
