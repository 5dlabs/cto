#!/usr/bin/env bash
# =============================================================================
# Scaffold Skills - Generate Per-Agent Skills Directories
# =============================================================================
#
# Creates config/skills-{agent}/ directories with SKILL.md files for each
# agent, including both default and all role-specific skills.
#
# Sources (in order of precedence):
#   1. cto-config.json (primary)
#   2. skill-mappings.yaml (fallback)
#
# Usage:
#   ./scaffold-skills.sh           # All agents
#   ./scaffold-skills.sh bolt      # Single agent
#
# This generates:
#   config/skills-{agent}/
#     ├── context-fundamentals/SKILL.md
#     ├── github-mcp/SKILL.md
#     └── ... (all skills from default + all job types)
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CTO_ROOT="${SCRIPT_DIR}/../.."
CTO_CONFIG="${CTO_ROOT}/cto-config.json"
SKILL_MAPPINGS="${CTO_ROOT}/templates/skills/skill-mappings.yaml"
SKILLS_SOURCE="${CTO_ROOT}/templates/skills"

# All agents from cto-config.json
ALL_AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

# If specific agent provided, use only that
if [ $# -ge 1 ] && [ -n "$1" ]; then
    AGENTS=("$1")
else
    AGENTS=("${ALL_AGENTS[@]}")
fi

echo "🎯 Scaffolding skills for ${#AGENTS[@]} agent(s)..."
echo ""

if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed"
    exit 1
fi

# Function to extract skills from skill-mappings.yaml using Python
get_skills_from_yaml() {
    local agent="$1"
    python3 << EOF
import yaml
import sys

try:
    with open("${SKILL_MAPPINGS}", 'r') as f:
        data = yaml.safe_load(f)
    
    if "${agent}" not in data:
        sys.exit(0)
    
    agent_data = data["${agent}"]
    skills = set()
    
    # Iterate through all keys except 'description' and 'optional'
    for key, value in agent_data.items():
        if key in ('description', 'optional'):
            continue
        if isinstance(value, list):
            skills.update(value)
    
    for skill in sorted(skills):
        print(skill)
except Exception as e:
    sys.exit(0)
EOF
}

# Stats
TOTAL_COPIED=0
TOTAL_MISSING=0

for agent in "${AGENTS[@]}"; do
    SKILLS_DIR="${SCRIPT_DIR}/config/skills-${agent}"
    
    echo "📦 Agent: ${agent}"
    
    # Clear and recreate skills directory
    rm -rf "${SKILLS_DIR}"
    mkdir -p "${SKILLS_DIR}"
    
    # Get all skills for this agent (default + all job types)
    # Extract the skills object and get all arrays merged
    SKILLS_JSON=$(jq -r ".agents.${agent}.skills // {}" "${CTO_CONFIG}" 2>/dev/null || echo "{}")
    
    SOURCE="cto-config.json"
    
    if [ "${SKILLS_JSON}" = "{}" ] || [ "${SKILLS_JSON}" = "null" ]; then
        # Fallback to skill-mappings.yaml
        ALL_SKILLS=$(get_skills_from_yaml "${agent}")
        SOURCE="skill-mappings.yaml"
        
        if [ -z "${ALL_SKILLS}" ]; then
            echo "   ⚠️  No skills found for '${agent}' in cto-config.json or skill-mappings.yaml"
            echo ""
            continue
        fi
    else
        # Get all skill names from all job types (default, coder, healer, intake, etc.)
        ALL_SKILLS=$(echo "${SKILLS_JSON}" | jq -r 'to_entries | .[].value | .[]' 2>/dev/null | sort -u | grep -v '^$' || true)
    fi
    
    if [ -z "${ALL_SKILLS}" ]; then
        echo "   ⚠️  Skills object empty for '${agent}'"
        echo ""
        continue
    fi
    
    COPIED=0
    MISSING=0
    MISSING_LIST=""
    
    for skill in ${ALL_SKILLS}; do
        # Search for SKILL.md in templates/skills/
        SKILL_FILE=$(find "${SKILLS_SOURCE}" -type f -name "SKILL.md" -path "*/${skill}/*" 2>/dev/null | head -1)
        
        if [ -n "${SKILL_FILE}" ]; then
            # Controller expects: skills/<skill-name>/SKILL.md
            mkdir -p "${SKILLS_DIR}/${skill}"
            cp "${SKILL_FILE}" "${SKILLS_DIR}/${skill}/SKILL.md"
            ((COPIED++))
        else
            MISSING_LIST="${MISSING_LIST}${skill}, "
            ((MISSING++))
        fi
    done
    
    echo "   ✓ ${COPIED} skills copied (from ${SOURCE})"
    if [ ${MISSING} -gt 0 ]; then
        echo "   ✗ ${MISSING} not found: ${MISSING_LIST%, }"
    fi
    echo "   → ${SKILLS_DIR}/"
    echo ""
    
    TOTAL_COPIED=$((TOTAL_COPIED + COPIED))
    TOTAL_MISSING=$((TOTAL_MISSING + MISSING))
done

echo "=============================================="
echo "📊 Summary"
echo "   Total skills copied:  ${TOTAL_COPIED}"
echo "   Total skills missing: ${TOTAL_MISSING}"
echo "=============================================="
echo ""

# Show what was created
echo "📁 Created directories:"
for agent in "${AGENTS[@]}"; do
    SKILLS_DIR="${SCRIPT_DIR}/config/skills-${agent}"
    if [ -d "${SKILLS_DIR}" ]; then
        COUNT=$(ls -1 "${SKILLS_DIR}" 2>/dev/null | wc -l | tr -d ' ')
        echo "   config/skills-${agent}/ (${COUNT} skills)"
    fi
done
