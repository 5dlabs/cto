#!/usr/bin/env bash
# scaffold-agents.sh - Create test folders and configs for all agents

set -e

CTO_CONFIG="../../cto-config.json"
SKILLS_SOURCE="../../templates/skills"

# All agents from cto-config.json
AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

echo "🚀 Scaffolding test infrastructure for ${#AGENTS[@]} agents..."
echo ""

# Create directories
for agent in "${AGENTS[@]}"; do
    echo "📁 Creating folders for: $agent"
    
    # Task folder with subtasks
    mkdir -p "config/task-${agent}/subtasks"
    
    # Workspace folder
    mkdir -p "workspaces/${agent}"
    
    # Skills folder (will be populated by setup.sh)
    mkdir -p "config/skills-${agent}"
done

echo ""
echo "📝 Generating client-config.json for each agent..."

for agent in "${AGENTS[@]}"; do
    # Extract tools for this agent from cto-config.json
    TOOLS=$(jq -r ".agents.${agent}.tools.remote // [] | @json" "$CTO_CONFIG")
    
    # Create client-config
    cat > "config/client-config-${agent}.json" << EOF
{
  "remoteTools": ${TOOLS}
}
EOF
    
    TOOL_COUNT=$(echo "$TOOLS" | jq 'length')
    echo "   ✓ ${agent}: ${TOOL_COUNT} tools"
done

echo ""
echo "📚 Generating sample prompts for each agent..."

get_agent_role() {
    case "$1" in
        atlas) echo "Architecture design and system planning" ;;
        blaze) echo "Frontend development and UI/UX" ;;
        bolt) echo "Infrastructure orchestration and Kubernetes" ;;
        cipher) echo "Security analysis and encryption" ;;
        cleo) echo "Documentation and technical writing" ;;
        grizz) echo "Database operations and optimization" ;;
        morgan) echo "Project management and coordination" ;;
        nova) echo "Data science and machine learning" ;;
        rex) echo "Backend development and APIs" ;;
        spark) echo "Performance optimization and profiling" ;;
        stitch) echo "Integration and glue code" ;;
        tap) echo "API integrations and webhooks" ;;
        tess) echo "Testing and QA" ;;
        vex) echo "Debugging and troubleshooting" ;;
        *) echo "General purpose agent" ;;
    esac
}

for agent in "${AGENTS[@]}"; do
    ROLE=$(get_agent_role "$agent")
    UPPER_AGENT=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    
    cat > "config/task-${agent}/prompt.md" << EOF
# ${UPPER_AGENT} Test Task

<task>
<agent>${agent}</agent>
<title>${ROLE}</title>
<objective>
Test ${agent} agent capabilities for ${ROLE}
</objective>

<requirements>
- Demonstrate core capabilities
- Use appropriate MCP tools
- Follow best practices
</requirements>

<deliverables>
- Task completion report
- Any generated artifacts
</deliverables>
</task>

## Instructions

This is a test prompt for the ${agent} agent. Replace with actual task content.
EOF
    
    echo "   ✓ ${agent}: config/task-${agent}/prompt.md"
done

echo ""
echo "✅ Scaffolding complete!"
echo ""
echo "Next steps:"
echo "  1. Run './setup.sh {agent} coder' to populate skills"
echo "  2. Update .env with LINEAR_ISSUE_IDENTIFIER"
echo "  3. Run 'docker compose up {agent} {agent}-sidecar'"
echo ""
echo "Agents ready: ${AGENTS[*]}"
