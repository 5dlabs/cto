#!/bin/bash

# PR Comment Management Script
# Tool for tracking and managing GitHub PR review comments
# Provides manual management until event-driven system is ready
# Usage: ./pr-comments.sh <command> [pr-number] [options]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMMENTS_DIR="${SCRIPT_DIR}/.pr-comments"
mkdir -p "$COMMENTS_DIR"

# Version and metadata
SCRIPT_VERSION="2.0.0"
SCRIPT_NAME="$(basename "$0")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default PR number (can be overridden)
DEFAULT_PR=""

usage() {
    cat << EOF
${BLUE}PR Comment Management Script v${SCRIPT_VERSION}${NC}
${BLUE}================================${NC}

${YELLOW}Usage:${NC} $SCRIPT_NAME <command> [pr-number] [options]

${YELLOW}Commands:${NC}
  ${GREEN}pull${NC} [pr]           Pull latest review comments from GitHub PR
  ${GREEN}list${NC} [pr]           List all comments with current status
  ${GREEN}status${NC} [pr]         Show comment status summary and statistics
  ${GREEN}resolve${NC} <id> [pr]   Mark a review comment as resolved
  ${GREEN}unresolve${NC} <id> [pr] Mark a review comment as unresolved
  ${GREEN}report${NC} [pr]         Generate comprehensive markdown status report
  ${GREEN}clean${NC} [pr]          Clean cached comment data
  ${GREEN}version${NC}             Show script version and information
  ${GREEN}help${NC}                Show this help message

${YELLOW}Examples:${NC}
  $SCRIPT_NAME pull 249                    # Pull comments from PR #249
  $SCRIPT_NAME list                        # List comments (uses last PR)
  $SCRIPT_NAME resolve 123                 # Mark comment 123 as resolved
  $SCRIPT_NAME status 249                  # Show status for PR #249
  $SCRIPT_NAME report 249 > status.md      # Generate markdown report

${YELLOW}Notes:${NC}
  • PR numbers are remembered between commands
  • Comments are cached locally for performance
  • Use 'pull' to refresh comment data
  • Review comments are code-specific line comments, not issue comments
EOF
}

# Get PR number (from arg, or last used, or prompt)
get_pr_number() {
    local pr_arg="$1"
    
    if [[ -n "$pr_arg" ]]; then
        echo "$pr_arg"
        return
    fi
    
    # Check for last used PR
    if [[ -f "$COMMENTS_DIR/last_pr" ]]; then
        cat "$COMMENTS_DIR/last_pr"
        return
    fi
    
    # No PR specified and no last PR
    echo "Error: No PR number specified and no previous PR found." >&2
    echo "Usage: $0 <command> <pr-number>" >&2
    exit 1
}

# Save last used PR number
save_last_pr() {
    echo "$1" > "$COMMENTS_DIR/last_pr"
}

# Pull comments from GitHub
pull_comments() {
    local pr_number="$1"
    local comments_file="$COMMENTS_DIR/pr_${pr_number}_comments.json"
    local status_file="$COMMENTS_DIR/pr_${pr_number}_status.json"
    
    echo "Pulling comments from PR #$pr_number..."
    
    # Get only review comments (actionable line-specific comments)
    if ! gh api "repos/$(gh repo view --json owner,name --jq '.owner.login + "/" + .name')/pulls/$pr_number/comments" > "$comments_file" 2>/dev/null; then
        echo "Error: Failed to fetch PR #$pr_number. Make sure it exists and you have access." >&2
        exit 1
    fi
    
    # Normalize review comment fields to match expected format
    jq 'map(. + {author: {login: .user.login}, createdAt: .created_at})' "$comments_file" > "${comments_file}.tmp"
    mv "${comments_file}.tmp" "$comments_file"
    
    # Initialize status file if it doesn't exist
    if [[ ! -f "$status_file" ]]; then
        echo "{}" > "$status_file"
    fi
    
    # Count comments
    local comment_count=$(jq 'length' "$comments_file")
    echo "✅ Fetched $comment_count review comments from PR #$pr_number"
    
    save_last_pr "$pr_number"
}

# Get comment status
get_comment_status() {
    local pr_number="$1"
    local comment_id="$2"
    local status_file="$COMMENTS_DIR/pr_${pr_number}_status.json"
    
    if [[ -f "$status_file" ]]; then
        jq -r ".[\"$comment_id\"].status // \"new\"" "$status_file"
    else
        echo "new"
    fi
}

# Set comment status
set_comment_status() {
    local pr_number="$1"
    local comment_id="$2"
    local status="$3"
    local status_file="$COMMENTS_DIR/pr_${pr_number}_status.json"
    
    # Initialize if doesn't exist
    if [[ ! -f "$status_file" ]]; then
        echo "{}" > "$status_file"
    fi
    
    # Update status with timestamp
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    jq ".[\"$comment_id\"] = {\"status\": \"$status\", \"timestamp\": \"$timestamp\"}" "$status_file" > "${status_file}.tmp"
    mv "${status_file}.tmp" "$status_file"
}

# List all comments
list_comments() {
    local pr_number="$1"
    local comments_file="$COMMENTS_DIR/pr_${pr_number}_comments.json"

    if [[ ! -f "$comments_file" ]]; then
        echo -e "${RED}❌ No comments found for PR #$pr_number${NC}"
        echo -e "${YELLOW}💡 Run: $SCRIPT_NAME pull $pr_number${NC}"
        return 1
    fi

    local comment_count=$(jq 'length' "$comments_file" 2>/dev/null || echo "0")

    echo -e "${BLUE}📋 PR #$pr_number Review Comments ($comment_count total)${NC}"
    echo -e "${BLUE}==================================================${NC}"
    echo ""

    if [[ "$comment_count" -eq 0 ]]; then
        echo -e "${YELLOW}📝 No review comments found for this PR${NC}"
        echo -e "${YELLOW}💡 Review comments are code-specific line comments, not issue comments${NC}"
        return 0
    fi

    # Process each review comment
    local comment_num=1
    jq -c '.[] | {id: .id, author: .author.login, created: .createdAt, body: .body, path: .path, line: .line}' "$comments_file" | while read -r comment; do
        local id=$(echo "$comment" | jq -r '.id')
        local author=$(echo "$comment" | jq -r '.author')
        local created=$(echo "$comment" | jq -r '.created')
        local body=$(echo "$comment" | jq -r '.body' | head -n 3 | tr '\n' ' ' | cut -c1-120)
        local path=$(echo "$comment" | jq -r '.path')
        local line=$(echo "$comment" | jq -r '.line')
        local status=$(get_comment_status "$pr_number" "$id")

        # Enhanced status indicator with emoji
        local status_icon="🔴"
        local status_color="$RED"
        local status_text="Open"
        case "$status" in
            "resolved")
                status_icon="✅"
                status_color="$GREEN"
                status_text="Resolved"
                ;;
            "new")
                status_icon="🔴"
                status_color="$RED"
                status_text="Open"
                ;;
        esac

        echo -e "${status_color}${status_icon} Comment ${comment_num}/${comment_count} - #$id by @$author${NC}"
        echo -e "   📁 ${YELLOW}$path:$line${NC}"
        echo -e "   📅 ${BLUE}$(date -d "$created" +"%Y-%m-%d %H:%M" 2>/dev/null || echo "$created")${NC}"
        echo -e "   💬 ${body}..."
        echo -e "   📊 ${status_color}Status: $status_text${NC}"
        echo ""
        ((comment_num++))
    done
}

# Show status summary
show_status() {
    local pr_number="$1"
    local comments_file="$COMMENTS_DIR/pr_${pr_number}_comments.json"
    
    if [[ ! -f "$comments_file" ]]; then
        echo "No comments found for PR #$pr_number. Run: $0 pull $pr_number"
        exit 1
    fi
    
    echo -e "${BLUE}=== PR #$pr_number Review Comments Status ===${NC}"
    echo ""
    
    local total=0
    local new=0
    local resolved=0
    
    jq -r '.[].id' "$comments_file" | while read -r id; do
        local status=$(get_comment_status "$pr_number" "$id")
        case "$status" in
            "new") ((new++)) ;;
            "resolved") ((resolved++)) ;;
        esac
        ((total++))
    done
    
    # Count comments by status
    total=$(jq 'length' "$comments_file")
    resolved=0
    
    # Count resolved comments
    for id in $(jq -r '.[].id' "$comments_file"); do
        local status=$(get_comment_status "$pr_number" "$id")
        if [[ "$status" == "resolved" ]]; then
            ((resolved++))
        fi
    done
    
    new=$((total - resolved))
    
    echo -e "📊 Total Review Comments: $total"
    echo -e "${RED}🔴 Open: $new${NC}"
    echo -e "${GREEN}✅ Resolved: $resolved${NC}"
    
    if [[ $new -eq 0 ]]; then
        echo ""
        echo -e "${GREEN}🎉 All review comments resolved!${NC}"
    fi
}

# Generate markdown report
generate_report() {
    local pr_number="$1"
    local comments_file="$COMMENTS_DIR/pr_${pr_number}_comments.json"
    
    if [[ ! -f "$comments_file" ]]; then
        echo "No comments found for PR #$pr_number. Run: $0 pull $pr_number"
        exit 1
    fi
    
    echo "# PR #$pr_number Review Comments Report"
    echo ""
    echo "Generated: $(date)"
    echo ""
    
    # Count comments by status
    local total=$(jq 'length' "$comments_file")
    local resolved=0
    
    # Count resolved comments
    for id in $(jq -r '.[].id' "$comments_file"); do
        local status=$(get_comment_status "$pr_number" "$id")
        if [[ "$status" == "resolved" ]]; then
            ((resolved++))
        fi
    done
    
    local new=$((total - resolved))
    
    echo "## Summary"
    echo "- 📊 **Total Review Comments**: $total"
    echo "- 🔴 **Open**: $new"
    echo "- ✅ **Resolved**: $resolved"
    echo ""
    
    if [[ $new -gt 0 ]]; then
        echo "## 🔴 Open Review Comments"
        echo ""
        jq -c '.[] | {id: .id, author: .author.login, created: .createdAt, body: .body, path: .path, line: .line}' "$comments_file" | while read -r comment; do
            local id=$(echo "$comment" | jq -r '.id')
            local status=$(get_comment_status "$pr_number" "$id")
            
            if [[ "$status" == "new" ]]; then
                local author=$(echo "$comment" | jq -r '.author')
                local created=$(echo "$comment" | jq -r '.created')
                local body=$(echo "$comment" | jq -r '.body')
                local path=$(echo "$comment" | jq -r '.path')
                local line=$(echo "$comment" | jq -r '.line')
                
                echo "### Review Comment #$id - @$author"
                echo "📁 **File:** \`$path:$line\`  "
                echo "*Created: $created*"
                echo ""
                echo "$body"
                echo ""
                echo "**Status**: 🔴 New"
                echo ""
                echo "---"
                echo ""
            fi
        done
    fi
    
    if [[ $resolved -gt 0 ]]; then
        echo "## ✅ Resolved Review Comments"
        echo ""
        jq -c '.[] | {id: .id, author: .author.login, created: .createdAt, body: .body, path: .path, line: .line}' "$comments_file" | while read -r comment; do
            local id=$(echo "$comment" | jq -r '.id')
            local status=$(get_comment_status "$pr_number" "$id")
            
            if [[ "$status" == "resolved" ]]; then
                local author=$(echo "$comment" | jq -r '.author')
                local body=$(echo "$comment" | jq -r '.body' | head -n 2 | tr '\n' ' ')
                local path=$(echo "$comment" | jq -r '.path')
                local line=$(echo "$comment" | jq -r '.line')
                
                echo "- **Review Comment #$id** - @$author (`$path:$line`): ${body}..."
            fi
        done
        echo ""
    fi
}

# Mark comment as resolved
resolve_comment() {
    local comment_id="$1"
    local pr_number="$2"
    
    set_comment_status "$pr_number" "$comment_id" "resolved"
    echo -e "${GREEN}✅ Marked comment #$comment_id as resolved${NC}"
}

# Mark comment as unresolved  
unresolve_comment() {
    local comment_id="$1"
    local pr_number="$2"
    
    set_comment_status "$pr_number" "$comment_id" "new"
    echo -e "${RED}🔴 Marked comment #$comment_id as new${NC}"
}

# Clean cached data
clean_data() {
    local pr_number="$1"
    rm -f "$COMMENTS_DIR/pr_${pr_number}_comments.json"
    rm -f "$COMMENTS_DIR/pr_${pr_number}_status.json"
    echo "✅ Cleaned cached data for PR #$pr_number"
}

# Show version information
show_version() {
    echo -e "${BLUE}$SCRIPT_NAME v$SCRIPT_VERSION${NC}"
    echo -e "${YELLOW}GitHub PR Review Comment Management${NC}"
    echo ""
    echo -e "${GREEN}Dependencies:${NC}"
    command -v gh >/dev/null 2>&1 && echo "  ✅ GitHub CLI" || echo "  ❌ GitHub CLI (required)"
    command -v jq >/dev/null 2>&1 && echo "  ✅ jq" || echo "  ❌ jq (required)"
    echo ""
    echo -e "${BLUE}Cache location:${NC} $COMMENTS_DIR"
}

# Main command processing
main() {
    local command="$1"

    case "$command" in
        "version"|"--version"|"-v")
            show_version
            ;;
        "help"|"--help"|"-h"|"")
            usage
            ;;
        "pull")
            local pr_number=$(get_pr_number "$2")
            pull_comments "$pr_number"
            ;;
        "list")
            local pr_number=$(get_pr_number "$2")
            list_comments "$pr_number" || exit 1
            ;;
        "status")
            local pr_number=$(get_pr_number "$2")
            show_status "$pr_number"
            ;;
        "resolve")
            local comment_id="$2"
            local pr_number=$(get_pr_number "$3")
            if [[ -z "$comment_id" ]]; then
                echo -e "${RED}❌ Error: Comment ID required for resolve command${NC}"
                echo -e "${YELLOW}💡 Usage: $SCRIPT_NAME resolve <comment-id> [pr-number]${NC}"
                exit 1
            fi
            resolve_comment "$comment_id" "$pr_number"
            ;;
        "unresolve")
            local comment_id="$2"
            local pr_number=$(get_pr_number "$3")
            if [[ -z "$comment_id" ]]; then
                echo -e "${RED}❌ Error: Comment ID required for unresolve command${NC}"
                echo -e "${YELLOW}💡 Usage: $SCRIPT_NAME unresolve <comment-id> [pr-number]${NC}"
                exit 1
            fi
            unresolve_comment "$comment_id" "$pr_number"
            ;;
        "report")
            local pr_number=$(get_pr_number "$2")
            generate_report "$pr_number"
            ;;
        "clean")
            local pr_number=$(get_pr_number "$2")
            clean_data "$pr_number"
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $command${NC}"
            echo ""
            usage
            exit 1
            ;;
    esac
}

# Check dependencies
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is required but not installed."
    echo "Install it with: brew install gh"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Install it with: brew install jq"
    exit 1
fi

# Run main function
main "$@"
