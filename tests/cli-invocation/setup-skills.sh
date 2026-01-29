#!/usr/bin/env bash
# =============================================================================
# Setup Skills - Mirror Controller's Skill Resolution
# =============================================================================
#
# Populates the flat-skills directory based on cto-config.json for a given
# agent and job type, mirroring how the controller generates skills.
#
# Usage:
#   ./setup-skills.sh [agent] [job_type]
#   ./setup-skills.sh rex coder      # Default
#   ./setup-skills.sh morgan intake
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CTO_ROOT="${SCRIPT_DIR}/../.."
CTO_CONFIG="${CTO_ROOT}/cto-config.json"
SKILLS_SOURCE="${CTO_ROOT}/templates/skills"
FLAT_SKILLS_DIR="${SCRIPT_DIR}/config/flat-skills"

AGENT="${1:-rex}"
JOB_TYPE="${2:-coder}"

echo "Setting up skills for agent '${AGENT}' with job type '${JOB_TYPE}'"
echo "  Config: ${CTO_CONFIG}"
echo "  Source: ${SKILLS_SOURCE}"
echo "  Output: ${FLAT_SKILLS_DIR}"

# Clear and recreate flat-skills directory
rm -rf "${FLAT_SKILLS_DIR}"
mkdir -p "${FLAT_SKILLS_DIR}"

# Extract skills from cto-config.json using jq
# Get default skills + job-type-specific skills
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed"
    exit 1
fi

# Get default skills
DEFAULT_SKILLS=$(jq -r ".agents.${AGENT}.skills.default // [] | .[]" "${CTO_CONFIG}" 2>/dev/null || echo "")

# Get job-type specific skills
JOB_SKILLS=$(jq -r ".agents.${AGENT}.skills.${JOB_TYPE} // [] | .[]" "${CTO_CONFIG}" 2>/dev/null || echo "")

# Combine and dedupe
ALL_SKILLS=$(echo -e "${DEFAULT_SKILLS}\n${JOB_SKILLS}" | sort -u | grep -v '^$' || true)

if [ -z "${ALL_SKILLS}" ]; then
    echo "Warning: No skills found for agent '${AGENT}' in cto-config.json"
    echo "         Falling back to skill-mappings.yaml is not implemented in this script"
    exit 0
fi

echo ""
echo "Skills to install:"
echo "${ALL_SKILLS}" | sed 's/^/  - /'
echo ""

# Find and copy each skill file
COPIED=0
MISSING=0

for skill in ${ALL_SKILLS}; do
    # Search for SKILL.md in templates/skills/
    SKILL_FILE=$(find "${SKILLS_SOURCE}" -type f -name "SKILL.md" -path "*/${skill}/*" 2>/dev/null | head -1)
    
    if [ -n "${SKILL_FILE}" ]; then
        # Claude expects flat files: skill-name.md
        cp "${SKILL_FILE}" "${FLAT_SKILLS_DIR}/${skill}.md"
        echo "  ✓ ${skill}"
        ((COPIED++))
    else
        echo "  ✗ ${skill} (not found in templates/skills/)"
        ((MISSING++))
    fi
done

echo ""
echo "Summary: ${COPIED} skills copied, ${MISSING} not found"
echo ""

# List the flat-skills directory
echo "Contents of ${FLAT_SKILLS_DIR}:"
ls -la "${FLAT_SKILLS_DIR}" | tail -n +2
