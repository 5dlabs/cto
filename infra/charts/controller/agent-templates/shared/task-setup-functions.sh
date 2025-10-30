#!/bin/bash
# Shared functions for resilient task setup operations
# Used across all agent container scripts to ensure robust file handling

# Function to safely ensure directory exists without overwriting contents
# Usage: safe_ensure_directory "/path/to/dir" "description"
safe_ensure_directory() {
    local target_dir="$1"
    local description="${2:-directory}"
    
    echo "🔍 Ensuring $description exists at: $target_dir"
    
    if [ -d "$target_dir" ]; then
        local file_count=$(find "$target_dir" -type f 2>/dev/null | wc -l)
        echo "✓ $description already exists with $file_count files"
        return 0
    else
        echo "📁 Creating $description..."
        if mkdir -p "$target_dir"; then
            echo "✓ Created $description at: $target_dir"
            return 0
        else
            echo "❌ Failed to create $description at: $target_dir"
            return 1
        fi
    fi
}

# Function to safely copy files with verification and retry
# Usage: safe_copy_file "/source/file" "/dest/file" "description" [retries]
safe_copy_file() {
    local source_file="$1"
    local dest_file="$2"
    local description="${3:-file}"
    local max_retries="${4:-3}"
    local attempt=1
    
    # Ensure destination directory exists
    local dest_dir=$(dirname "$dest_file")
    if ! safe_ensure_directory "$dest_dir" "destination directory for $description"; then
        echo "❌ Cannot create destination directory for $description"
        return 1
    fi
    
    echo "📋 Copying $description..."
    echo "   Source: $source_file"
    echo "   Dest:   $dest_file"
    
    # Check if source file exists
    if [ ! -f "$source_file" ]; then
        echo "❌ Source file not found: $source_file"
        return 1
    fi
    
    while [ $attempt -le $max_retries ]; do
        echo "🔄 Attempt $attempt/$max_retries: Copying $description..."
        
        # Perform the copy
        if cp "$source_file" "$dest_file" 2>/dev/null; then
            # Verify the copy was successful
            if [ -f "$dest_file" ]; then
                local source_size=$(stat -c%s "$source_file" 2>/dev/null || echo "0")
                local dest_size=$(stat -c%s "$dest_file" 2>/dev/null || echo "0")
                
                if [ "$source_size" = "$dest_size" ] && [ "$source_size" -gt 0 ]; then
                    echo "✅ $description copied successfully (${source_size} bytes)"
                    return 0
                else
                    echo "⚠️ Size mismatch: source=$source_size, dest=$dest_size"
                fi
            else
                echo "⚠️ Destination file not found after copy"
            fi
        else
            echo "⚠️ Copy command failed"
        fi
        
        if [ $attempt -lt $max_retries ]; then
            echo "🔄 Retrying in 1 second..."
            sleep 1
        fi
        
        attempt=$((attempt + 1))
    done
    
    echo "❌ Failed to copy $description after $max_retries attempts"
    return 1
}

# Function to safely copy directory contents with verification
# Usage: safe_copy_directory "/source/dir" "/dest/dir" "description" [retries]
safe_copy_directory() {
    local source_dir="$1"
    local dest_dir="$2"
    local description="${3:-directory}"
    local max_retries="${4:-3}"
    
    echo "📂 Copying $description contents..."
    echo "   Source: $source_dir"
    echo "   Dest:   $dest_dir"
    
    # Check if source directory exists
    if [ ! -d "$source_dir" ]; then
        echo "❌ Source directory not found: $source_dir"
        return 1
    fi
    
    # Ensure destination directory exists
    if ! safe_ensure_directory "$dest_dir" "$description destination"; then
        return 1
    fi
    
    # Copy all files in source directory
    local failed_files=0
    local total_files=0
    
    find "$source_dir" -maxdepth 1 -type f | while read -r source_file; do
        local filename=$(basename "$source_file")
        local dest_file="$dest_dir/$filename"
        total_files=$((total_files + 1))
        
        if ! safe_copy_file "$source_file" "$dest_file" "$filename" "$max_retries"; then
            failed_files=$((failed_files + 1))
        fi
    done
    
    # Get final counts for reporting
    local source_count=$(find "$source_dir" -maxdepth 1 -type f 2>/dev/null | wc -l)
    local dest_count=$(find "$dest_dir" -maxdepth 1 -type f 2>/dev/null | wc -l)
    
    # Success if destination has at least as many files as source (extra files from previous agents are OK)
    if [ "$dest_count" -ge "$source_count" ] && [ "$source_count" -gt 0 ]; then
        if [ "$dest_count" -gt "$source_count" ]; then
            echo "✅ $description copied successfully ($source_count files copied, $((dest_count - source_count)) pre-existing)"
        else
            echo "✅ $description copied successfully ($dest_count files)"
        fi
        return 0
    else
        echo "❌ $description copy incomplete: $dest_count/$source_count files copied"
        return 1
    fi
}

# Function to verify critical task files exist
# Usage: verify_task_files "/path/to/task/dir" "task_id"
verify_task_files() {
    local task_dir="$1"
    local task_id="${2:-unknown}"
    local missing_files=0
    
    echo "🔍 Verifying task $task_id files in: $task_dir"
    
    # Critical files that must exist
    local critical_files=(
        "prompt.md"
        "task.md"
        "acceptance-criteria.md"
    )
    
    # Optional files to check
    local optional_files=(
        "architecture.md"
        "toolman-guide.md"
        "tasks.json"
    )
    
    # Check critical files
    for file in "${critical_files[@]}"; do
        local file_path="$task_dir/$file"
        if [ -f "$file_path" ]; then
            local size=$(stat -c%s "$file_path" 2>/dev/null || echo "0")
            echo "✅ Critical file found: $file (${size} bytes)"
        else
            echo "❌ CRITICAL FILE MISSING: $file"
            missing_files=$((missing_files + 1))
        fi
    done
    
    # Check optional files
    for file in "${optional_files[@]}"; do
        local file_path="$task_dir/$file"
        if [ -f "$file_path" ]; then
            local size=$(stat -c%s "$file_path" 2>/dev/null || echo "0")
            echo "✓ Optional file found: $file (${size} bytes)"
        else
            echo "ℹ️ Optional file missing: $file"
        fi
    done
    
    # Overall status
    if [ $missing_files -eq 0 ]; then
        echo "✅ Task $task_id files verification: PASSED"
        return 0
    else
        echo "❌ Task $task_id files verification: FAILED ($missing_files critical files missing)"
        return 1
    fi
}

# Function to create comprehensive error report
# Usage: create_error_report "/path/to/task/dir" "task_id" "error_description"
create_error_report() {
    local task_dir="$1"
    local task_id="${2:-unknown}"
    local error_desc="${3:-Unknown error during task setup}"
    
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "❌ TASK SETUP ERROR REPORT"
    echo "════════════════════════════════════════════════════════════════"
    echo "Task ID: $task_id"
    echo "Error: $error_desc"
    echo "Timestamp: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    echo ""
    
    echo "📁 DIRECTORY STATUS:"
    if [ -d "$task_dir" ]; then
        echo "✓ Task directory exists: $task_dir"
        echo "  Contents:"
        ls -la "$task_dir" | sed 's/^/    /'
        echo "  File count: $(find "$task_dir" -type f 2>/dev/null | wc -l)"
    else
        echo "❌ Task directory missing: $task_dir"
    fi
    
    echo ""
    echo "🔧 RECOMMENDED ACTIONS:"
    echo "1. Check ConfigMap contents for task-$task_id"
    echo "2. Verify workspace volume mount permissions"
    echo "3. Check container logs for file operation errors"
    echo "4. Validate docs repository structure"
    echo ""
    echo "════════════════════════════════════════════════════════════════"
}

# Function to attempt recovery from task setup failure
# Usage: attempt_task_recovery "/docs/source" "/task/dest" "task_id"
attempt_task_recovery() {
    local docs_source="$1"
    local task_dest="$2" 
    local task_id="${3:-unknown}"
    
    echo "🔄 ATTEMPTING TASK RECOVERY for task $task_id..."
    
    # Try to find task files in alternative locations
    local recovery_locations=(
        "/tmp/docs-repo/.taskmaster/docs/task-$task_id"
        "/task-files"
        "/workspace/.taskmaster/docs/task-$task_id"
    )
    
    for location in "${recovery_locations[@]}"; do
        echo "🔍 Checking recovery location: $location"
        if [ -d "$location" ] && [ -f "$location/prompt.md" ]; then
            echo "✅ Found task files at: $location"
            if safe_copy_directory "$location" "$task_dest" "recovery task files"; then
                echo "✅ Recovery successful!"
                return 0
            fi
        fi
    done
    
    echo "❌ Recovery failed - no valid task files found"
    return 1
}
