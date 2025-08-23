#!/bin/bash

# Markdown Linting Fix Script
# Fixes common markdown linting issues automatically

set -e

show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  lint     - Show linting issues"
    echo "  fix      - Fix common linting issues"
    echo "  stats    - Show linting statistics"
    echo "  help     - Show this help"
    echo ""
}

show_stats() {
    echo "Markdown linting statistics:"
    echo "Total Markdown files: $(find . -name "*.md" -type f -not -path "./.claude/*" -not -path "./docs/.claude/*" -not -path "./docs/.taskmaster/*" | wc -l)"
    echo ""
    echo "Current issues by type:"
    markdownlint --config .markdownlint.yaml --ignore ".claude/**" --ignore "docs/.claude/**" --ignore "docs/.taskmaster/**" "**/*.md" 2>&1 | grep -o "MD[0-9]*" | sort | uniq -c | sort -nr
}

run_lint() {
    echo "Running markdown linting..."
    markdownlint --config .markdownlint.yaml --ignore ".claude/**" --ignore "docs/.claude/**" --ignore "docs/.taskmaster/**" "**/*.md"
}

fix_issues() {
    echo "Fixing common markdown linting issues..."
    
    # Find all markdown files (excluding .claude and .taskmaster directories)
    find . -name "*.md" -type f -not -path "./.claude/*" -not -path "./docs/.claude/*" -not -path "./docs/.taskmaster/*" | while read -r file; do
        echo "Processing: $file"
        
        # Create a temporary file
        temp_file=$(mktemp)
        
        # Fix common issues
        cat "$file" | sed -E '
            # Fix MD012: Remove multiple blank lines (keep only one) - do this first
            s/\n\n\n+/\n\n/g
            
            # Fix MD009: Remove trailing spaces
            s/[[:space:]]+$//g
            
            # Fix MD031: Add blank lines around code blocks (but be careful)
            s/^```/\n```/g
            s/```$/\n```/g
            
            # Fix MD012 again: Remove any new multiple blank lines created
            s/\n\n\n+/\n\n/g
        ' > "$temp_file"
        
        # Replace original file
        mv "$temp_file" "$file"
    done
    
    echo "Fixed common issues. Run 'lint' to check remaining issues."
}

case "${1:-help}" in
    lint)
        run_lint
        ;;
    fix)
        fix_issues
        ;;
    stats)
        show_stats
        ;;
    help|*)
        show_help
        ;;
esac
