#!/bin/bash

# Markdown linting and fixing script
# This script helps identify and fix common Markdown linting issues

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if markdownlint is installed
check_markdownlint() {
    if ! command -v markdownlint &> /dev/null; then
        print_error "markdownlint-cli is not installed."
        print_status "Installing markdownlint-cli..."
        npm install -g markdownlint-cli
        print_success "markdownlint-cli installed successfully!"
    else
        print_success "markdownlint-cli is already installed."
    fi
}

# Function to run markdownlint
run_lint() {
    print_status "Running Markdown linting..."
    
    if markdownlint --config .markdownlint.yaml "**/*.md"; then
        print_success "All Markdown files passed linting!"
        return 0
    else
        print_warning "Some Markdown files have linting issues."
        return 1
    fi
}

# Function to fix common issues automatically
fix_common_issues() {
    print_status "Attempting to fix common Markdown issues..."
    
    # Find all markdown files
    local md_files
    md_files=$(find . -name "*.md" -type f)
    
    local fixed_count=0
    
    while IFS= read -r file; do
        if [[ -f "$file" ]]; then
            local original_content
            original_content=$(cat "$file")
            local fixed_content="$original_content"
            
            # Fix trailing whitespace
            fixed_content=$(echo "$fixed_content" | sed 's/[[:space:]]*$//')
            
            # Fix multiple consecutive blank lines (keep max 2)
            fixed_content=$(echo "$fixed_content" | sed '/^$/N;/^\n$/D')
            
            # Fix list indentation (ensure consistent 2-space indentation)
            fixed_content=$(echo "$fixed_content" | sed 's/^  \* /  * /g')
            fixed_content=$(echo "$fixed_content" | sed 's/^  - /  - /g')
            fixed_content=$(echo "$fixed_content" | sed 's/^  \+ /  + /g')
            
            # Fix heading spacing (ensure blank lines around headings)
            fixed_content=$(echo "$fixed_content" | sed 's/^\([^#\n].*\)\n\(#\{1,6\} .*\)/\1\n\n\2/g')
            fixed_content=$(echo "$fixed_content" | sed 's/^\(#\{1,6\} .*\)\n\([^#\n].*\)/\1\n\n\2/g')
            
            # Only write if content changed
            if [[ "$original_content" != "$fixed_content" ]]; then
                echo "$fixed_content" > "$file"
                print_status "Fixed issues in: $file"
                ((fixed_count++))
            fi
        fi
    done <<< "$md_files"
    
    if [[ $fixed_count -gt 0 ]]; then
        print_success "Fixed issues in $fixed_count files."
    else
        print_status "No automatic fixes were needed."
    fi
}

# Function to show linting statistics
show_stats() {
    print_status "Markdown linting statistics:"
    
    local total_files
    total_files=$(find . -name "*.md" -type f | wc -l)
    echo "  Total Markdown files: $total_files"
    
    # Count files by directory
    echo "  Files by directory:"
    find . -name "*.md" -type f | sed 's|/[^/]*$||' | sort | uniq -c | sort -nr | head -10 | while read -r count dir; do
        echo "    $count files in $dir"
    done
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  lint          Run Markdown linting (default)"
    echo "  fix           Run linting and attempt to fix common issues"
    echo "  stats         Show Markdown file statistics"
    echo "  install       Install markdownlint-cli if not present"
    echo "  help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 lint       # Run linting only"
    echo "  $0 fix        # Run linting and fix issues"
    echo "  $0 stats      # Show file statistics"
}

# Main script logic
main() {
    local action="${1:-lint}"
    
    case "$action" in
        "lint")
            check_markdownlint
            run_lint
            ;;
        "fix")
            check_markdownlint
            fix_common_issues
            run_lint
            ;;
        "stats")
            show_stats
            ;;
        "install")
            check_markdownlint
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "Unknown action: $action"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
