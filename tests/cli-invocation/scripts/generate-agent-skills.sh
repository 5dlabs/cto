#!/usr/bin/env bash
# =============================================================================
# Generate Skills Directory for an Agent
#
# Reads skill-mappings.yaml and copies only the skills for a specific agent/job
# to mirror how the controller sets up skills for CodeRuns.
#
# Usage:
#   ./generate-agent-skills.sh <agent_name> <job_type> <output_dir>
#   ./generate-agent-skills.sh rex coder ./config/skills-rex
#
# =============================================================================
set -euo pipefail

AGENT="${1:-rex}"
JOB_TYPE="${2:-coder}"
OUTPUT_DIR="${3:-./config/skills-${AGENT}}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")"
SKILLS_SOURCE="${PROJECT_ROOT}/templates/skills"
MAPPINGS_FILE="${SKILLS_SOURCE}/skill-mappings.yaml"

echo "═══════════════════════════════════════════════════════════════"
echo "║          Generate Agent Skills Directory                     ║"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Agent:       ${AGENT}"
echo "Job Type:    ${JOB_TYPE}"
echo "Output:      ${OUTPUT_DIR}"
echo "Source:      ${SKILLS_SOURCE}"
echo "Mappings:    ${MAPPINGS_FILE}"
echo ""

if [ ! -f "$MAPPINGS_FILE" ]; then
    echo "ERROR: Mappings file not found: $MAPPINGS_FILE"
    exit 1
fi

# Parse YAML to get skills for agent
# Extract default skills and job-type skills
get_skills() {
    local agent="$1"
    local job="$2"
    local yaml_file="$3"
    
    # Use yq if available, otherwise fallback to grep/sed
    if command -v yq &> /dev/null; then
        # Get default skills
        default_skills=$(yq -r ".${agent}.default[]?" "$yaml_file" 2>/dev/null | tr '\n' ' ')
        # Get job-specific skills
        job_skills=$(yq -r ".${agent}.${job}[]?" "$yaml_file" 2>/dev/null | tr '\n' ' ')
        echo "$default_skills $job_skills"
    else
        # Fallback: simple extraction (less robust)
        echo "WARNING: yq not found, using fallback parser" >&2
        
        # Extract section for agent and parse skills
        in_agent=0
        in_section=""
        skills=""
        
        while IFS= read -r line; do
            # Check if we're entering the agent section
            if [[ "$line" =~ ^${agent}: ]]; then
                in_agent=1
                continue
            fi
            
            # Check if we're leaving the agent section (another top-level key)
            if [[ $in_agent -eq 1 && "$line" =~ ^[a-z]+: && ! "$line" =~ ^[[:space:]] ]]; then
                break
            fi
            
            if [[ $in_agent -eq 1 ]]; then
                # Check for section start (default:, coder:, etc.)
                if [[ "$line" =~ ^[[:space:]]*default: ]]; then
                    in_section="default"
                elif [[ "$line" =~ ^[[:space:]]*${job}: ]]; then
                    in_section="$job"
                elif [[ "$line" =~ ^[[:space:]]*[a-z]+: && ! "$line" =~ ^[[:space:]]*- ]]; then
                    in_section=""
                fi
                
                # Extract skill names from list items
                if [[ "$in_section" == "default" || "$in_section" == "$job" ]]; then
                    if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*([a-z0-9-]+) ]]; then
                        skill="${BASH_REMATCH[1]}"
                        skills="$skills $skill"
                    fi
                fi
            fi
        done < "$yaml_file"
        
        echo "$skills"
    fi
}

# Get skills for the agent
SKILLS=$(get_skills "$AGENT" "$JOB_TYPE" "$MAPPINGS_FILE")

# Remove duplicates and empty entries
SKILLS=$(echo "$SKILLS" | tr ' ' '\n' | sort -u | grep -v '^$' | tr '\n' ' ')

echo "Skills to load: $SKILLS"
echo ""

# Create output directory
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Copy each skill
LOADED=0
MISSING=0

for skill in $SKILLS; do
    FOUND=0
    
    # Search in all category directories
    for category in stacks auth context design languages llm-docs platforms quality workflow animations tools; do
        if [ -f "$SKILLS_SOURCE/$category/$skill/SKILL.md" ]; then
            mkdir -p "$OUTPUT_DIR/$skill"
            cp "$SKILLS_SOURCE/$category/$skill/SKILL.md" "$OUTPUT_DIR/$skill/"
            echo "  ✓ Copied: $skill (from $category)"
            LOADED=$((LOADED + 1))
            FOUND=1
            break
        fi
    done
    
    if [ "$FOUND" -eq 0 ]; then
        echo "  ⚠️ Not found: $skill"
        MISSING=$((MISSING + 1))
    fi
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Summary: $LOADED skills loaded, $MISSING not found"
echo "Output directory: $OUTPUT_DIR"
echo "═══════════════════════════════════════════════════════════════"

# List what was created
if [ "$LOADED" -gt 0 ]; then
    echo ""
    echo "Skills directory structure:"
    ls -la "$OUTPUT_DIR"
fi
