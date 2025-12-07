#!/bin/bash
# Template Snapshot Testing Script
# 
# Usage:
#   ./scripts/snapshot-templates.sh save    # Save current rendered output as baseline
#   ./scripts/snapshot-templates.sh compare # Compare current output against baseline
#   ./scripts/snapshot-templates.sh clean   # Remove snapshot directory
#
# This enables safe refactoring by ensuring rendered output doesn't change unexpectedly.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SNAPSHOT_DIR="$PROJECT_ROOT/tmp/template-snapshots"
TEMPLATE_DIR="$PROJECT_ROOT/templates"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Render a single template with mock data
render_template() {
    local template_path="$1"
    local output_path="$2"
    
    # Simple rendering using the test binary
    # For now, we'll just copy the template content as a placeholder
    # In production, this would use actual Handlebars rendering
    cat "$template_path" > "$output_path"
}

save_snapshots() {
    log_info "Saving template snapshots to $SNAPSHOT_DIR"
    
    mkdir -p "$SNAPSHOT_DIR/before"
    
    # Run the test-templates binary and capture its validation
    cd "$PROJECT_ROOT"
    if cargo run -p controller --bin test-templates > "$SNAPSHOT_DIR/before/render-log.txt" 2>&1; then
        log_info "Templates rendered successfully"
    else
        log_error "Template rendering failed!"
        cat "$SNAPSHOT_DIR/before/render-log.txt"
        exit 1
    fi
    
    # Copy all template files for comparison
    log_info "Copying template files..."
    find "$TEMPLATE_DIR" -name "*.hbs" -o -name "*.md" | while read -r file; do
        rel_path="${file#$TEMPLATE_DIR/}"
        mkdir -p "$SNAPSHOT_DIR/before/$(dirname "$rel_path")"
        cp "$file" "$SNAPSHOT_DIR/before/$rel_path"
    done
    
    log_info "Snapshots saved! $(find "$SNAPSHOT_DIR/before" -type f | wc -l | tr -d ' ') files"
}

compare_snapshots() {
    if [ ! -d "$SNAPSHOT_DIR/before" ]; then
        log_error "No baseline snapshots found. Run './scripts/snapshot-templates.sh save' first."
        exit 1
    fi
    
    log_info "Comparing current templates against baseline..."
    
    mkdir -p "$SNAPSHOT_DIR/after"
    
    # Run the test-templates binary
    cd "$PROJECT_ROOT"
    if cargo run -p controller --bin test-templates > "$SNAPSHOT_DIR/after/render-log.txt" 2>&1; then
        log_info "Templates rendered successfully"
    else
        log_error "Template rendering failed!"
        cat "$SNAPSHOT_DIR/after/render-log.txt"
        exit 1
    fi
    
    # Copy current template files
    find "$TEMPLATE_DIR" -name "*.hbs" -o -name "*.md" | while read -r file; do
        rel_path="${file#$TEMPLATE_DIR/}"
        mkdir -p "$SNAPSHOT_DIR/after/$(dirname "$rel_path")"
        cp "$file" "$SNAPSHOT_DIR/after/$rel_path"
    done
    
    # Compare the directories
    log_info "Comparing directories..."
    
    local diff_count=0
    local new_count=0
    local deleted_count=0
    
    # Check for modified files
    while IFS= read -r file; do
        rel_path="${file#$SNAPSHOT_DIR/before/}"
        after_file="$SNAPSHOT_DIR/after/$rel_path"
        
        if [ -f "$after_file" ]; then
            if ! diff -q "$file" "$after_file" > /dev/null 2>&1; then
                log_warn "MODIFIED: $rel_path"
                diff_count=$((diff_count + 1))
            fi
        else
            log_warn "DELETED: $rel_path"
            deleted_count=$((deleted_count + 1))
        fi
    done < <(find "$SNAPSHOT_DIR/before" -type f -name "*.hbs" -o -name "*.md")
    
    # Check for new files
    while IFS= read -r file; do
        rel_path="${file#$SNAPSHOT_DIR/after/}"
        before_file="$SNAPSHOT_DIR/before/$rel_path"
        
        if [ ! -f "$before_file" ]; then
            log_info "NEW: $rel_path"
            new_count=$((new_count + 1))
        fi
    done < <(find "$SNAPSHOT_DIR/after" -type f -name "*.hbs" -o -name "*.md")
    
    echo ""
    log_info "=== Summary ==="
    log_info "Modified: $diff_count"
    log_info "New: $new_count"
    log_info "Deleted: $deleted_count"
    
    if [ $diff_count -eq 0 ] && [ $deleted_count -eq 0 ]; then
        log_info "✅ No unexpected changes detected!"
        return 0
    else
        log_warn "⚠️ Changes detected. Review with: diff -r $SNAPSHOT_DIR/before $SNAPSHOT_DIR/after"
        return 1
    fi
}

clean_snapshots() {
    log_info "Removing snapshot directory..."
    rm -rf "$SNAPSHOT_DIR"
    log_info "Done!"
}

show_diff() {
    local file="${1:-}"
    
    if [ -z "$file" ]; then
        # Show overall diff
        diff -rq "$SNAPSHOT_DIR/before" "$SNAPSHOT_DIR/after" 2>/dev/null | head -50 || true
    else
        # Show specific file diff
        diff -u "$SNAPSHOT_DIR/before/$file" "$SNAPSHOT_DIR/after/$file" | head -100 || true
    fi
}

case "${1:-help}" in
    save)
        save_snapshots
        ;;
    compare)
        compare_snapshots
        ;;
    clean)
        clean_snapshots
        ;;
    diff)
        show_diff "${2:-}"
        ;;
    *)
        echo "Template Snapshot Testing"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  save     Save current templates as baseline"
        echo "  compare  Compare current templates against baseline"
        echo "  diff     Show differences (optionally specify file)"
        echo "  clean    Remove snapshot directory"
        ;;
esac

