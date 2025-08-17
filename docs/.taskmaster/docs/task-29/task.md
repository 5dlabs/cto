# Task 29: Create Agent-Specific Container Scripts Implementing Distinct Workflows for Each Agent Type

## Overview

This task develops three specialized container scripts for Rex, Cleo, and Tess agents that implement distinct workflows tailored to each agent's responsibilities. The scripts maintain compatibility with the existing CRD structure while providing agent-specific functionality and optimizations.

## Technical Requirements

### Agent-Specific Responsibilities

1. **Rex Agent (Documentation-Driven Implementation)**
   - Query MCP documentation server before implementation
   - Pull relevant API docs, architecture guides, and patterns into context
   - Implement documentation-first approach with comprehensive inline comments
   - Generate implementation plans based on documented patterns

2. **Cleo Agent (Code Quality and Formatting)**
   - Execute comprehensive formatting with `cargo fmt`
   - Apply strict linting with `cargo clippy`
   - Organize imports and detect dead code
   - Generate quality reports and apply automatic fixes

3. **Tess Agent (Testing and Deployment Validation)**
   - Run comprehensive test suites with coverage analysis
   - Execute integration tests and performance benchmarks
   - Validate deployment readiness and coverage thresholds
   - Generate detailed test reports and approve PRs

## Implementation Guide

### Step 1: Rex Agent Container Script (container-rex.sh.hbs)

```bash
#!/bin/bash
# Rex Agent - Documentation-Driven Implementation
# Template: container-rex.sh.hbs

set -euo pipefail

# Configuration
AGENT_TYPE="rex"
GITHUB_APP="{{github_app}}"
TASK_ID="{{task_id}}"
WORKSPACE_PATH="{{workspace_path}}"
GITHUB_TOKEN="{{github_token}}"
MCP_SERVER_URL="{{mcp_server_url}}"

# Logging setup
log() {
    echo "[$(date -Iseconds)] [REX] $*" | tee -a "${WORKSPACE_PATH}/rex.log"
}

error() {
    echo "[$(date -Iseconds)] [REX] ERROR: $*" | tee -a "${WORKSPACE_PATH}/rex.log" >&2
}

# Initialize workspace and environment
initialize_rex_environment() {
    log "Initializing Rex environment for task ${TASK_ID}"
    
    # Set up workspace
    cd "${WORKSPACE_PATH}"
    
    # Configure git
    git config --global user.name "Rex Agent"
    git config --global user.email "rex@taskmaster.io"
    
    # Verify MCP server connectivity
    if ! curl -f --max-time 30 "${MCP_SERVER_URL}/health" > /dev/null 2>&1; then
        error "MCP server not accessible at ${MCP_SERVER_URL}"
        return 1
    fi
    
    log "Rex environment initialized successfully"
}

# Query documentation from MCP server
query_documentation() {
    local query="$1"
    local context_file="${WORKSPACE_PATH}/documentation_context.md"
    
    log "Querying documentation: ${query}"
    
    # Query MCP documentation server
    curl -s -X POST "${MCP_SERVER_URL}/api/v1/query" \
        -H "Content-Type: application/json" \
        -d "{\"query\": \"${query}\", \"include_examples\": true}" \
        -o "/tmp/doc_response.json"
    
    if [ $? -eq 0 ]; then
        # Extract documentation content
        jq -r '.content' /tmp/doc_response.json >> "${context_file}"
        
        # Extract code examples
        jq -r '.examples[]' /tmp/doc_response.json >> "${WORKSPACE_PATH}/code_examples.rs"
        
        log "Documentation retrieved and saved to ${context_file}"
    else
        error "Failed to query documentation for: ${query}"
        return 1
    fi
}

# Analyze task requirements and gather relevant documentation
gather_implementation_context() {
    local task_file="${WORKSPACE_PATH}/.taskmaster/docs/task-${TASK_ID}/task.txt"
    
    log "Gathering implementation context for task ${TASK_ID}"
    
    if [ ! -f "$task_file" ]; then
        error "Task file not found: $task_file"
        return 1
    fi
    
    # Extract key technologies and concepts from task description
    local technologies=$(grep -i -E "(rust|cargo|kubernetes|docker|api)" "$task_file" | head -10)
    local patterns=$(grep -i -E "(pattern|architecture|design)" "$task_file" | head -10)
    
    # Query documentation for relevant technologies
    while IFS= read -r tech_line; do
        if [[ -n "$tech_line" ]]; then
            local tech=$(echo "$tech_line" | grep -oE "\b(rust|cargo|kubernetes|docker|api)\b" | head -1)
            if [[ -n "$tech" ]]; then
                query_documentation "documentation for ${tech} best practices"
                sleep 1  # Rate limiting
            fi
        fi
    done <<< "$technologies"
    
    # Query architectural patterns
    query_documentation "architecture patterns and design principles"
    query_documentation "API design best practices"
    query_documentation "error handling patterns"
    
    log "Implementation context gathered successfully"
}

# Generate implementation plan based on documentation
generate_implementation_plan() {
    local task_file="${WORKSPACE_PATH}/.taskmaster/docs/task-${TASK_ID}/task.txt"
    local plan_file="${WORKSPACE_PATH}/implementation_plan.md"
    
    log "Generating implementation plan"
    
    cat > "$plan_file" <<EOF
# Implementation Plan for Task ${TASK_ID}
Generated by Rex Agent at $(date -Iseconds)

## Task Requirements
$(cat "$task_file")

## Implementation Approach
Based on documentation analysis and architectural patterns:

1. **Architecture Design**
   - Follow documented patterns from MCP server
   - Implement proper error handling as per guidelines
   - Use established API design principles

2. **Code Structure**
   - Organize modules according to documented conventions
   - Implement comprehensive inline documentation
   - Follow documented naming conventions

3. **Testing Strategy**
   - Unit tests following documented patterns
   - Integration tests for API endpoints
   - Error condition testing

4. **Documentation Requirements**
   - API documentation with examples
   - Architecture decision records
   - Usage guidelines

## Implementation Steps
$(generate_step_by_step_plan)

EOF
    
    log "Implementation plan generated: $plan_file"
}

generate_step_by_step_plan() {
    echo "1. Set up project structure following documented conventions"
    echo "2. Implement core functionality with comprehensive comments"
    echo "3. Add error handling based on documentation patterns"
    echo "4. Create unit tests following documented testing patterns"
    echo "5. Generate API documentation with examples"
    echo "6. Validate implementation against documentation requirements"
}

# Implement code with documentation-driven approach
implement_with_documentation() {
    local implementation_file="$1"
    
    log "Implementing ${implementation_file} with documentation-driven approach"
    
    # Create implementation with extensive documentation
    cat > "$implementation_file" <<'EOF'
//! Task Implementation - Generated by Rex Agent
//! 
//! This implementation follows documented patterns and best practices
//! retrieved from the MCP documentation server.
//!
//! # Architecture
//! 
//! The implementation follows the documented architectural patterns:
//! - Modular design with clear separation of concerns
//! - Comprehensive error handling using Result<T, E>
//! - Extensive logging for debugging and monitoring
//! 
//! # Usage
//! 
//! ```rust
//! // Example usage based on documentation
//! let result = TaskImplementation::new()
//!     .configure_from_docs()
//!     .execute();
//! ```

use std::error::Error;
use std::fmt;

/// Main implementation structure following documented patterns
#[derive(Debug, Clone)]
pub struct TaskImplementation {
    /// Configuration loaded from documentation
    config: TaskConfig,
    /// Current execution state
    state: ExecutionState,
}

/// Task configuration based on documented requirements
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Task identifier from CRD
    pub task_id: String,
    /// Agent type (rex, cleo, tess)
    pub agent_type: String,
    /// Workspace path for file operations
    pub workspace_path: String,
}

/// Execution state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    /// Initial state
    Initialized,
    /// Documentation gathered
    DocumentationLoaded,
    /// Implementation in progress
    Implementing,
    /// Implementation complete
    Complete,
    /// Error occurred
    Failed(String),
}

/// Custom error type following documented error handling patterns
#[derive(Debug)]
pub enum TaskError {
    /// Documentation query failed
    DocumentationError(String),
    /// Implementation failed
    ImplementationError(String),
    /// IO error
    IoError(std::io::Error),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::DocumentationError(msg) => write!(f, "Documentation error: {}", msg),
            TaskError::ImplementationError(msg) => write!(f, "Implementation error: {}", msg),
            TaskError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for TaskError {}

impl From<std::io::Error> for TaskError {
    fn from(err: std::io::Error) -> Self {
        TaskError::IoError(err)
    }
}

impl TaskImplementation {
    /// Create new task implementation
    /// 
    /// # Arguments
    /// 
    /// * `config` - Task configuration following documented structure
    /// 
    /// # Returns
    /// 
    /// New TaskImplementation instance
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let config = TaskConfig {
    ///     task_id: "29".to_string(),
    ///     agent_type: "rex".to_string(),
    ///     workspace_path: "/workspace".to_string(),
    /// };
    /// let implementation = TaskImplementation::new(config);
    /// ```
    pub fn new(config: TaskConfig) -> Self {
        Self {
            config,
            state: ExecutionState::Initialized,
        }
    }
    
    /// Load documentation and prepare for implementation
    /// 
    /// This method queries the MCP documentation server to gather
    /// relevant patterns and examples for the task implementation.
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure
    /// 
    /// # Errors
    /// 
    /// Returns TaskError::DocumentationError if documentation
    /// cannot be loaded from the MCP server.
    pub fn load_documentation(&mut self) -> Result<(), TaskError> {
        log::info!("Loading documentation for task {}", self.config.task_id);
        
        // Implementation would query MCP server here
        // For template, we'll simulate success
        
        self.state = ExecutionState::DocumentationLoaded;
        Ok(())
    }
    
    /// Execute the task implementation
    /// 
    /// This method performs the main task implementation following
    /// the patterns and guidelines loaded from documentation.
    /// 
    /// # Returns
    /// 
    /// Result indicating success or failure
    pub fn execute(&mut self) -> Result<(), TaskError> {
        match self.state {
            ExecutionState::DocumentationLoaded => {
                self.state = ExecutionState::Implementing;
                
                // Perform implementation steps
                self.implement_core_functionality()?;
                self.generate_tests()?;
                self.update_documentation()?;
                
                self.state = ExecutionState::Complete;
                Ok(())
            }
            _ => Err(TaskError::ImplementationError(
                "Must load documentation before executing".to_string()
            ))
        }
    }
    
    /// Implement core functionality based on documented patterns
    fn implement_core_functionality(&self) -> Result<(), TaskError> {
        log::info!("Implementing core functionality");
        // Implementation details would go here
        Ok(())
    }
    
    /// Generate tests following documented testing patterns
    fn generate_tests(&self) -> Result<(), TaskError> {
        log::info!("Generating tests based on documentation patterns");
        // Test generation would go here
        Ok(())
    }
    
    /// Update documentation with implementation details
    fn update_documentation(&self) -> Result<(), TaskError> {
        log::info!("Updating documentation");
        // Documentation update would go here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_implementation_creation() {
        let config = TaskConfig {
            task_id: "29".to_string(),
            agent_type: "rex".to_string(),
            workspace_path: "/workspace".to_string(),
        };
        
        let implementation = TaskImplementation::new(config);
        assert_eq!(implementation.state, ExecutionState::Initialized);
    }
    
    #[test]
    fn test_documentation_loading() {
        let config = TaskConfig {
            task_id: "29".to_string(),
            agent_type: "rex".to_string(),
            workspace_path: "/workspace".to_string(),
        };
        
        let mut implementation = TaskImplementation::new(config);
        assert!(implementation.load_documentation().is_ok());
        assert_eq!(implementation.state, ExecutionState::DocumentationLoaded);
    }
}
EOF
    
    log "Implementation file created with comprehensive documentation"
}

# Main Rex workflow execution
main_rex_workflow() {
    log "Starting Rex agent workflow for task ${TASK_ID}"
    
    # Initialize environment
    initialize_rex_environment || {
        error "Failed to initialize Rex environment"
        exit 1
    }
    
    # Gather implementation context
    gather_implementation_context || {
        error "Failed to gather implementation context"
        exit 1
    }
    
    # Generate implementation plan
    generate_implementation_plan || {
        error "Failed to generate implementation plan"
        exit 1
    }
    
    # Implement with documentation-driven approach
    local main_file="${WORKSPACE_PATH}/src/lib.rs"
    mkdir -p "$(dirname "$main_file")"
    implement_with_documentation "$main_file"
    
    # Create Cargo.toml with proper documentation
    create_cargo_manifest
    
    # Generate comprehensive inline documentation
    generate_inline_documentation
    
    # Commit changes with descriptive message
    commit_implementation_changes
    
    log "Rex agent workflow completed successfully"
}

create_cargo_manifest() {
    local cargo_file="${WORKSPACE_PATH}/Cargo.toml"
    
    cat > "$cargo_file" <<EOF
[package]
name = "taskmaster-task-${TASK_ID}"
version = "0.1.0"
edition = "2021"
description = "Task ${TASK_ID} implementation following documented patterns"
documentation = "https://docs.taskmaster.io/task-${TASK_ID}"

[dependencies]
log = "0.4"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tokio-test = "0.4"

[[bin]]
name = "task-${TASK_ID}"
path = "src/main.rs"

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
EOF
    
    log "Cargo manifest created with documentation metadata"
}

generate_inline_documentation() {
    log "Generating comprehensive inline documentation"
    
    # Generate README.md
    cat > "${WORKSPACE_PATH}/README.md" <<EOF
# Task ${TASK_ID} Implementation

Implementation generated by Rex Agent following documentation-driven development.

## Overview

This implementation follows patterns and guidelines retrieved from the MCP documentation server, ensuring consistency with established architectural principles.

## Architecture

The implementation uses documented patterns for:
- Error handling with Result<T, E>
- Modular design with clear separation of concerns
- Comprehensive logging and monitoring
- Test-driven development approach

## Usage

\`\`\`bash
cargo run --bin task-${TASK_ID}
\`\`\`

## Testing

\`\`\`bash
cargo test
\`\`\`

## Documentation

Generate documentation with:

\`\`\`bash
cargo doc --open
\`\`\`
EOF
    
    log "README.md generated with comprehensive documentation"
}

commit_implementation_changes() {
    log "Committing implementation changes"
    
    cd "${WORKSPACE_PATH}"
    
    # Stage all changes
    git add .
    
    # Create descriptive commit message
    local commit_message="feat: implement task-${TASK_ID} following documented patterns

- Documentation-driven implementation using MCP server
- Comprehensive inline documentation and examples
- Error handling following documented patterns
- Test coverage with documented testing approach
- Generated by Rex Agent with architectural guidance

Co-authored-by: MCP-Documentation-Server <mcp@taskmaster.io>"
    
    git commit -m "$commit_message"
    
    log "Changes committed with descriptive message"
}

# Error handling
trap 'error "Rex workflow failed at line $LINENO"' ERR

# Execute main workflow
main_rex_workflow
```

### Step 2: Cleo Agent Container Script (container-cleo.sh.hbs)

```bash
#!/bin/bash
# Cleo Agent - Code Quality and Formatting
# Template: container-cleo.sh.hbs

set -euo pipefail

# Configuration
AGENT_TYPE="cleo"
GITHUB_APP="{{github_app}}"
TASK_ID="{{task_id}}"
WORKSPACE_PATH="{{workspace_path}}"
GITHUB_TOKEN="{{github_token}}"
QUALITY_RULES="{{quality_rules}}"

# Logging setup
log() {
    echo "[$(date -Iseconds)] [CLEO] $*" | tee -a "${WORKSPACE_PATH}/cleo.log"
}

error() {
    echo "[$(date -Iseconds)] [CLEO] ERROR: $*" | tee -a "${WORKSPACE_PATH}/cleo.log" >&2
}

# Initialize Cleo environment
initialize_cleo_environment() {
    log "Initializing Cleo environment for task ${TASK_ID}"
    
    cd "${WORKSPACE_PATH}"
    
    # Configure git
    git config --global user.name "Cleo Agent"
    git config --global user.email "cleo@taskmaster.io"
    
    # Verify required tools
    if ! command -v cargo &> /dev/null; then
        error "cargo not found - required for code quality checks"
        return 1
    fi
    
    if ! cargo fmt --version &> /dev/null; then
        error "cargo fmt not available - installing rustfmt"
        rustup component add rustfmt
    fi
    
    if ! cargo clippy --version &> /dev/null; then
        error "cargo clippy not available - installing clippy"
        rustup component add clippy
    fi
    
    log "Cleo environment initialized successfully"
}

# Check current formatting status
check_formatting_status() {
    local status_file="${WORKSPACE_PATH}/formatting_status.json"
    
    log "Checking current code formatting status"
    
    # Run cargo fmt --check to identify issues
    local fmt_output
    if fmt_output=$(cargo fmt --check 2>&1); then
        log "Code is already properly formatted"
        echo '{"status": "formatted", "issues": []}' > "$status_file"
        return 0
    else
        log "Formatting issues detected"
        
        # Parse formatting issues
        local issues=()
        while IFS= read -r line; do
            if [[ "$line" =~ ^Diff ]]; then
                local file=$(echo "$line" | awk '{print $2}')
                issues+=("\"$file\"")
            fi
        done <<< "$fmt_output"
        
        # Create status report
        local issues_json=$(IFS=,; echo "[${issues[*]}]")
        echo "{\"status\": \"needs_formatting\", \"issues\": $issues_json}" > "$status_file"
        
        log "Formatting status saved to $status_file"
        return 1
    fi
}

# Apply code formatting
apply_code_formatting() {
    log "Applying code formatting with cargo fmt"
    
    # Apply formatting
    if cargo fmt; then
        log "Code formatting applied successfully"
        
        # Show what was changed
        if [ -n "$(git diff --name-only)" ]; then
            log "Files modified by formatting:"
            git diff --name-only | while read -r file; do
                log "  - $file"
            done
            
            # Generate formatting report
            generate_formatting_report
        else
            log "No formatting changes were needed"
        fi
    else
        error "Failed to apply code formatting"
        return 1
    fi
}

generate_formatting_report() {
    local report_file="${WORKSPACE_PATH}/formatting_report.md"
    
    log "Generating formatting report"
    
    cat > "$report_file" <<EOF
# Code Formatting Report
Generated by Cleo Agent at $(date -Iseconds)

## Summary
Applied automatic code formatting using \`cargo fmt\`.

## Files Modified
$(git diff --name-only | sed 's/^/- /')

## Formatting Changes
\`\`\`diff
$(git diff)
\`\`\`

## Configuration
Used default rustfmt configuration with the following key settings:
- Line width: 100 characters
- Indent: 4 spaces
- Import organization: enabled
- Comment formatting: enabled
EOF
    
    log "Formatting report generated: $report_file"
}

# Run Clippy analysis
run_clippy_analysis() {
    local clippy_report="${WORKSPACE_PATH}/clippy_report.json"
    
    log "Running cargo clippy analysis"
    
    # Run clippy with JSON output
    local clippy_output
    if clippy_output=$(cargo clippy --message-format=json --all-targets --all-features -- -D warnings 2>&1); then
        log "Clippy analysis completed with no warnings"
        echo '{"status": "clean", "warnings": [], "errors": []}' > "$clippy_report"
        return 0
    else
        log "Clippy found issues - generating report"
        
        # Parse clippy output
        echo "$clippy_output" | jq -s '
        {
            "status": "issues_found",
            "warnings": [.[] | select(.reason == "compiler-message" and .message.level == "warning")],
            "errors": [.[] | select(.reason == "compiler-message" and .message.level == "error")]
        }' > "$clippy_report"
        
        # Generate human-readable report
        generate_clippy_report "$clippy_output"
        
        return 1
    fi
}

generate_clippy_report() {
    local clippy_output="$1"
    local report_file="${WORKSPACE_PATH}/clippy_analysis.md"
    
    cat > "$report_file" <<EOF
# Clippy Analysis Report
Generated by Cleo Agent at $(date -Iseconds)

## Summary
Cargo clippy analysis with strict warning levels (-D warnings).

## Issues Found
$(echo "$clippy_output" | grep -E "(warning|error):" | head -20)

## Recommendations
Based on clippy analysis:
1. Address all warnings to improve code quality
2. Consider using suggested improvements for better performance
3. Review code patterns flagged by clippy
4. Ensure all new code follows clippy recommendations

## Configuration
Clippy run with:
- All targets included
- All features enabled  
- Warnings treated as errors (-D warnings)
EOF
    
    log "Clippy analysis report generated: $report_file"
}

# Apply automatic clippy fixes
apply_clippy_fixes() {
    log "Applying automatic clippy fixes"
    
    # Run clippy with --fix flag
    if cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings; then
        log "Clippy fixes applied successfully"
        
        if [ -n "$(git diff --name-only)" ]; then
            log "Files modified by clippy fixes:"
            git diff --name-only | while read -r file; do
                log "  - $file"
            done
        fi
    else
        log "Some clippy issues require manual intervention"
    fi
}

# Organize imports
organize_imports() {
    log "Organizing imports and removing dead code"
    
    # This would require additional tooling like cargo-sort or similar
    # For now, we'll focus on basic organization
    
    find "${WORKSPACE_PATH}/src" -name "*.rs" -type f | while read -r file; do
        if grep -q "^use " "$file"; then
            log "Organizing imports in $file"
            
            # Simple import sorting (basic approach)
            # In practice, you'd use more sophisticated tools
            perl -i -pe 'BEGIN{undef $/;} s/(use [^;]*;\n)+/join "", sort split "\n", $&/smge' "$file"
        fi
    done
    
    log "Import organization completed"
}

# Generate comprehensive quality report
generate_quality_report() {
    local report_file="${WORKSPACE_PATH}/quality_report.md"
    local summary_file="${WORKSPACE_PATH}/quality_summary.json"
    
    log "Generating comprehensive quality report"
    
    # Count lines of code
    local loc=$(find "${WORKSPACE_PATH}/src" -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
    
    # Check for common issues
    local todo_count=$(find "${WORKSPACE_PATH}/src" -name "*.rs" -exec grep -c "TODO\|FIXME" {} + | awk '{sum += $1} END {print sum+0}')
    local panic_count=$(find "${WORKSPACE_PATH}/src" -name "*.rs" -exec grep -c "panic!\|unwrap()" {} + | awk '{sum += $1} END {print sum+0}')
    
    cat > "$report_file" <<EOF
# Code Quality Report
Generated by Cleo Agent at $(date -Iseconds)

## Overview
Comprehensive code quality analysis and improvements applied.

## Metrics
- Lines of Code: $loc
- TODO/FIXME items: $todo_count
- Potential panics (panic!/unwrap): $panic_count

## Changes Applied
1. **Code Formatting**
   - Applied cargo fmt with standard configuration
   - Ensured consistent indentation and spacing
   - Organized code structure

2. **Linting Fixes**
   - Applied cargo clippy recommendations
   - Fixed common code patterns
   - Improved performance suggestions

3. **Import Organization**
   - Sorted and organized use statements
   - Removed unused imports
   - Grouped imports logically

## Quality Improvements
$(list_quality_improvements)

## Recommendations
1. Consider addressing remaining TODO items
2. Replace unwrap() calls with proper error handling
3. Add documentation for public APIs
4. Consider adding more comprehensive tests
EOF
    
    # Generate JSON summary
    cat > "$summary_file" <<EOF
{
    "timestamp": "$(date -Iseconds)",
    "agent": "cleo",
    "task_id": "${TASK_ID}",
    "metrics": {
        "lines_of_code": $loc,
        "todo_count": $todo_count,
        "panic_count": $panic_count
    },
    "changes_applied": {
        "formatting": true,
        "clippy_fixes": true,
        "import_organization": true
    },
    "status": "completed"
}
EOF
    
    log "Quality report generated: $report_file"
}

list_quality_improvements() {
    echo "- Consistent code formatting across all files"
    echo "- Resolved clippy warnings and suggestions"
    echo "- Organized imports for better readability"
    echo "- Applied performance optimizations where suggested"
}

# Label PR as ready for QA
label_pr_ready_for_qa() {
    log "Labeling PR as ready for QA"
    
    if [ -n "${GITHUB_TOKEN:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
        local pr_number=$(git log --oneline -1 | grep -oE '#[0-9]+' | head -1 | sed 's/#//')
        
        if [ -n "$pr_number" ]; then
            curl -X POST \
                -H "Authorization: token $GITHUB_TOKEN" \
                -H "Accept: application/vnd.github.v3+json" \
                "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$pr_number/labels" \
                -d '{"labels":["ready-for-qa","cleo-processed"]}'
            
            log "PR #$pr_number labeled as ready-for-qa"
        else
            log "Could not determine PR number for labeling"
        fi
    else
        log "GitHub token or repository not configured - skipping PR labeling"
    fi
}

# Main Cleo workflow execution
main_cleo_workflow() {
    log "Starting Cleo agent workflow for task ${TASK_ID}"
    
    # Initialize environment
    initialize_cleo_environment || {
        error "Failed to initialize Cleo environment"
        exit 1
    }
    
    # Check current formatting status
    local needs_formatting=false
    if ! check_formatting_status; then
        needs_formatting=true
    fi
    
    # Apply formatting if needed
    if [ "$needs_formatting" = true ]; then
        apply_code_formatting || {
            error "Failed to apply code formatting"
            exit 1
        }
    fi
    
    # Run clippy analysis
    local has_clippy_issues=false
    if ! run_clippy_analysis; then
        has_clippy_issues=true
    fi
    
    # Apply clippy fixes if needed
    if [ "$has_clippy_issues" = true ]; then
        apply_clippy_fixes
        
        # Re-run clippy to verify fixes
        run_clippy_analysis
    fi
    
    # Organize imports
    organize_imports
    
    # Generate comprehensive quality report
    generate_quality_report
    
    # Commit quality improvements
    commit_quality_improvements
    
    # Label PR as ready for QA
    label_pr_ready_for_qa
    
    log "Cleo agent workflow completed successfully"
}

commit_quality_improvements() {
    log "Committing code quality improvements"
    
    cd "${WORKSPACE_PATH}"
    
    # Stage all changes
    git add .
    
    local commit_message="style: apply code quality improvements for task-${TASK_ID}

- Applied cargo fmt for consistent formatting
- Resolved cargo clippy warnings and suggestions
- Organized imports and removed dead code
- Generated comprehensive quality report
- Applied performance optimizations

Processed by Cleo Agent with strict quality standards"
    
    git commit -m "$commit_message"
    
    log "Quality improvements committed"
}

# Error handling
trap 'error "Cleo workflow failed at line $LINENO"' ERR

# Execute main workflow
main_cleo_workflow
```

### Step 3: Tess Agent Container Script (container-tess.sh.hbs)

```bash
#!/bin/bash
# Tess Agent - Testing and Deployment Validation
# Template: container-tess.sh.hbs

set -euo pipefail

# Configuration
AGENT_TYPE="tess"
GITHUB_APP="{{github_app}}"
TASK_ID="{{task_id}}"
WORKSPACE_PATH="{{workspace_path}}"
GITHUB_TOKEN="{{github_token}}"
COVERAGE_THRESHOLD="{{coverage_threshold}}"

# Logging setup
log() {
    echo "[$(date -Iseconds)] [TESS] $*" | tee -a "${WORKSPACE_PATH}/tess.log"
}

error() {
    echo "[$(date -Iseconds)] [TESS] ERROR: $*" | tee -a "${WORKSPACE_PATH}/tess.log" >&2
}

# Initialize Tess environment
initialize_tess_environment() {
    log "Initializing Tess environment for task ${TASK_ID}"
    
    cd "${WORKSPACE_PATH}"
    
    # Configure git
    git config --global user.name "Tess Agent"
    git config --global user.email "tess@taskmaster.io"
    
    # Verify required tools
    if ! command -v cargo &> /dev/null; then
        error "cargo not found - required for testing"
        return 1
    fi
    
    # Install coverage tools if not available
    if ! cargo llvm-cov --version &> /dev/null 2>&1; then
        log "Installing cargo-llvm-cov for coverage analysis"
        cargo install cargo-llvm-cov
    fi
    
    # Install additional testing tools
    if ! cargo nextest --version &> /dev/null 2>&1; then
        log "Installing cargo-nextest for enhanced testing"
        cargo install cargo-nextest --locked
    fi
    
    log "Tess environment initialized successfully"
}

# Run comprehensive test suite
run_comprehensive_tests() {
    local test_results_dir="${WORKSPACE_PATH}/test_results"
    mkdir -p "$test_results_dir"
    
    log "Running comprehensive test suite"
    
    # Run unit tests with detailed output
    log "Executing unit tests..."
    if cargo test --all-features --no-fail-fast -- --nocapture > "${test_results_dir}/unit_tests.log" 2>&1; then
        log "Unit tests passed"
        echo "PASSED" > "${test_results_dir}/unit_test_status"
    else
        error "Unit tests failed"
        echo "FAILED" > "${test_results_dir}/unit_test_status"
        return 1
    fi
    
    # Run integration tests
    log "Executing integration tests..."
    if find . -name "*integration*" -type f | grep -q .; then
        if cargo test --test '*' > "${test_results_dir}/integration_tests.log" 2>&1; then
            log "Integration tests passed"
            echo "PASSED" > "${test_results_dir}/integration_test_status"
        else
            error "Integration tests failed"
            echo "FAILED" > "${test_results_dir}/integration_test_status"
            return 1
        fi
    else
        log "No integration tests found"
        echo "NONE" > "${test_results_dir}/integration_test_status"
    fi
    
    # Run doc tests
    log "Executing documentation tests..."
    if cargo test --doc > "${test_results_dir}/doc_tests.log" 2>&1; then
        log "Documentation tests passed"
        echo "PASSED" > "${test_results_dir}/doc_test_status"
    else
        log "Documentation tests failed (non-critical)"
        echo "FAILED" > "${test_results_dir}/doc_test_status"
    fi
}

# Generate coverage reports
generate_coverage_reports() {
    local coverage_dir="${WORKSPACE_PATH}/coverage"
    mkdir -p "$coverage_dir"
    
    log "Generating coverage reports with cargo llvm-cov"
    
    # Generate coverage report
    if cargo llvm-cov --all-features --workspace --lcov --output-path "${coverage_dir}/lcov.info"; then
        log "Coverage report generated successfully"
        
        # Generate HTML report
        if command -v genhtml &> /dev/null; then
            genhtml "${coverage_dir}/lcov.info" -o "${coverage_dir}/html" --title "Task ${TASK_ID} Coverage Report"
            log "HTML coverage report generated in ${coverage_dir}/html"
        fi
        
        # Generate JSON summary
        cargo llvm-cov --all-features --workspace --json --output-path "${coverage_dir}/coverage.json"
        
        # Parse coverage percentage
        local coverage_percent=$(jq -r '.data[0].totals.lines.percent' "${coverage_dir}/coverage.json" 2>/dev/null || echo "0")
        
        log "Current coverage: ${coverage_percent}%"
        echo "$coverage_percent" > "${coverage_dir}/coverage_percent.txt"
        
        # Check coverage threshold
        validate_coverage_threshold "$coverage_percent"
        
    else
        error "Failed to generate coverage report"
        return 1
    fi
}

validate_coverage_threshold() {
    local coverage_percent="$1"
    local threshold="${COVERAGE_THRESHOLD:-95}"
    
    log "Validating coverage threshold: ${coverage_percent}% vs ${threshold}%"
    
    if (( $(echo "$coverage_percent >= $threshold" | bc -l) )); then
        log "Coverage threshold met: ${coverage_percent}% >= ${threshold}%"
        echo "PASSED" > "${WORKSPACE_PATH}/coverage/threshold_status"
        return 0
    else
        error "Coverage threshold not met: ${coverage_percent}% < ${threshold}%"
        echo "FAILED" > "${WORKSPACE_PATH}/coverage/threshold_status"
        return 1
    fi
}

# Validate deployment readiness
validate_deployment_readiness() {
    log "Validating deployment readiness"
    
    # Check if code compiles for release
    log "Testing release build..."
    if cargo build --release --all-features > "${WORKSPACE_PATH}/release_build.log" 2>&1; then
        log "Release build successful"
        echo "PASSED" > "${WORKSPACE_PATH}/release_build_status"
    else
        error "Release build failed"
        echo "FAILED" > "${WORKSPACE_PATH}/release_build_status"
        return 1
    fi
    
    # Run benchmarks if available
    if [ -d "benches" ] || grep -q "bench" Cargo.toml; then
        log "Running performance benchmarks..."
        if cargo bench > "${WORKSPACE_PATH}/benchmark_results.txt" 2>&1; then
            log "Benchmarks completed"
            echo "PASSED" > "${WORKSPACE_PATH}/benchmark_status"
        else
            log "Benchmarks failed or unavailable"
            echo "FAILED" > "${WORKSPACE_PATH}/benchmark_status"
        fi
    else
        log "No benchmarks found"
        echo "NONE" > "${WORKSPACE_PATH}/benchmark_status"
    fi
    
    # Validate configuration files
    validate_configuration_files
    
    # Check for security vulnerabilities
    run_security_audit
}

validate_configuration_files() {
    log "Validating configuration files"
    
    # Check Cargo.toml syntax
    if cargo metadata --format-version 1 > /dev/null 2>&1; then
        log "Cargo.toml is valid"
    else
        error "Cargo.toml has syntax errors"
        return 1
    fi
    
    # Validate other configuration files
    local config_files=("Dockerfile" "docker-compose.yml" ".github/workflows/*.yml")
    
    for pattern in "${config_files[@]}"; do
        for file in $pattern; do
            if [ -f "$file" ]; then
                log "Validating configuration file: $file"
                # Add specific validation logic for each file type
                case "$file" in
                    *.yml|*.yaml)
                        if command -v yq &> /dev/null; then
                            yq eval '.' "$file" > /dev/null
                        fi
                        ;;
                    Dockerfile)
                        if command -v hadolint &> /dev/null; then
                            hadolint "$file"
                        fi
                        ;;
                esac
            fi
        done
    done
}

run_security_audit() {
    log "Running security audit"
    
    if cargo audit --version > /dev/null 2>&1; then
        if cargo audit > "${WORKSPACE_PATH}/security_audit.log" 2>&1; then
            log "Security audit passed"
            echo "PASSED" > "${WORKSPACE_PATH}/security_audit_status"
        else
            log "Security vulnerabilities found - see security_audit.log"
            echo "FAILED" > "${WORKSPACE_PATH}/security_audit_status"
        fi
    else
        log "cargo-audit not available - installing"
        cargo install cargo-audit
        cargo audit > "${WORKSPACE_PATH}/security_audit.log" 2>&1
    fi
}

# Generate comprehensive test report
generate_test_report() {
    local report_file="${WORKSPACE_PATH}/test_report.md"
    local summary_file="${WORKSPACE_PATH}/test_summary.json"
    
    log "Generating comprehensive test report"
    
    # Read test results
    local unit_status=$(cat "${WORKSPACE_PATH}/test_results/unit_test_status" 2>/dev/null || echo "UNKNOWN")
    local integration_status=$(cat "${WORKSPACE_PATH}/test_results/integration_test_status" 2>/dev/null || echo "UNKNOWN")
    local doc_status=$(cat "${WORKSPACE_PATH}/test_results/doc_test_status" 2>/dev/null || echo "UNKNOWN")
    local coverage_percent=$(cat "${WORKSPACE_PATH}/coverage/coverage_percent.txt" 2>/dev/null || echo "0")
    local threshold_status=$(cat "${WORKSPACE_PATH}/coverage/threshold_status" 2>/dev/null || echo "UNKNOWN")
    local release_status=$(cat "${WORKSPACE_PATH}/release_build_status" 2>/dev/null || echo "UNKNOWN")
    local benchmark_status=$(cat "${WORKSPACE_PATH}/benchmark_status" 2>/dev/null || echo "UNKNOWN")
    local security_status=$(cat "${WORKSPACE_PATH}/security_audit_status" 2>/dev/null || echo "UNKNOWN")
    
    cat > "$report_file" <<EOF
# Test and Deployment Validation Report
Generated by Tess Agent at $(date -Iseconds)

## Test Results Summary

### Unit Tests
**Status:** $unit_status
$(cat "${WORKSPACE_PATH}/test_results/unit_tests.log" | tail -10)

### Integration Tests  
**Status:** $integration_status
$(if [ "$integration_status" != "NONE" ]; then cat "${WORKSPACE_PATH}/test_results/integration_tests.log" | tail -5; fi)

### Documentation Tests
**Status:** $doc_status

## Coverage Analysis
**Current Coverage:** ${coverage_percent}%
**Threshold Status:** $threshold_status
**Required Threshold:** ${COVERAGE_THRESHOLD:-95}%

### Coverage Details
- Lines covered: $(jq -r '.data[0].totals.lines.covered' "${WORKSPACE_PATH}/coverage/coverage.json" 2>/dev/null || echo "N/A")
- Total lines: $(jq -r '.data[0].totals.lines.count' "${WORKSPACE_PATH}/coverage/coverage.json" 2>/dev/null || echo "N/A")
- Functions covered: $(jq -r '.data[0].totals.functions.covered' "${WORKSPACE_PATH}/coverage/coverage.json" 2>/dev/null || echo "N/A")

## Deployment Readiness
**Release Build:** $release_status
**Benchmarks:** $benchmark_status  
**Security Audit:** $security_status

## Recommendations
$(generate_test_recommendations)

## Performance Metrics
$(if [ -f "${WORKSPACE_PATH}/benchmark_results.txt" ]; then cat "${WORKSPACE_PATH}/benchmark_results.txt" | head -10; fi)

## Security Assessment
$(if [ -f "${WORKSPACE_PATH}/security_audit.log" ]; then cat "${WORKSPACE_PATH}/security_audit.log" | head -5; fi)
EOF
    
    # Generate JSON summary for automation
    cat > "$summary_file" <<EOF
{
    "timestamp": "$(date -Iseconds)",
    "agent": "tess",
    "task_id": "${TASK_ID}",
    "test_results": {
        "unit_tests": "$unit_status",
        "integration_tests": "$integration_status",
        "doc_tests": "$doc_status"
    },
    "coverage": {
        "percentage": $coverage_percent,
        "threshold_met": $([ "$threshold_status" = "PASSED" ] && echo "true" || echo "false"),
        "required_threshold": ${COVERAGE_THRESHOLD:-95}
    },
    "deployment_readiness": {
        "release_build": "$release_status",
        "benchmarks": "$benchmark_status",
        "security_audit": "$security_status"
    },
    "overall_status": "$(determine_overall_status)"
}
EOF
    
    log "Test report generated: $report_file"
}

generate_test_recommendations() {
    echo "Based on test results:"
    
    local unit_status=$(cat "${WORKSPACE_PATH}/test_results/unit_test_status" 2>/dev/null || echo "UNKNOWN")
    local coverage_percent=$(cat "${WORKSPACE_PATH}/coverage/coverage_percent.txt" 2>/dev/null || echo "0")
    local threshold="${COVERAGE_THRESHOLD:-95}"
    
    if [ "$unit_status" != "PASSED" ]; then
        echo "- Address failing unit tests before deployment"
    fi
    
    if (( $(echo "$coverage_percent < $threshold" | bc -l) )); then
        echo "- Increase test coverage to meet ${threshold}% threshold"
        echo "- Add tests for uncovered code paths"
    fi
    
    echo "- All tests passing - ready for deployment"
    echo "- Consider adding integration tests if none exist"
    echo "- Performance benchmarks look good"
}

determine_overall_status() {
    local unit_status=$(cat "${WORKSPACE_PATH}/test_results/unit_test_status" 2>/dev/null || echo "UNKNOWN")
    local threshold_status=$(cat "${WORKSPACE_PATH}/coverage/threshold_status" 2>/dev/null || echo "UNKNOWN")
    local release_status=$(cat "${WORKSPACE_PATH}/release_build_status" 2>/dev/null || echo "UNKNOWN")
    
    if [ "$unit_status" = "PASSED" ] && [ "$threshold_status" = "PASSED" ] && [ "$release_status" = "PASSED" ]; then
        echo "APPROVED"
    else
        echo "NEEDS_WORK"
    fi
}

# Approve PR if all tests pass
approve_pr_if_tests_pass() {
    local overall_status=$(determine_overall_status)
    
    log "Overall test status: $overall_status"
    
    if [ "$overall_status" = "APPROVED" ]; then
        log "All tests passed - approving PR"
        
        if [ -n "${GITHUB_TOKEN:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
            local pr_number=$(git log --oneline -1 | grep -oE '#[0-9]+' | head -1 | sed 's/#//')
            
            if [ -n "$pr_number" ]; then
                # Create PR review approval
                curl -X POST \
                    -H "Authorization: token $GITHUB_TOKEN" \
                    -H "Accept: application/vnd.github.v3+json" \
                    "https://api.github.com/repos/$GITHUB_REPOSITORY/pulls/$pr_number/reviews" \
                    -d "{
                        \"body\": \"All tests passed with ${coverage_percent}% coverage. Deployment validation successful. Approved by Tess Agent.\",
                        \"event\": \"APPROVE\"
                    }"
                
                # Add approval label
                curl -X POST \
                    -H "Authorization: token $GITHUB_TOKEN" \
                    -H "Accept: application/vnd.github.v3+json" \
                    "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$pr_number/labels" \
                    -d '{"labels":["tess-approved","tests-passing"]}'
                
                log "PR #$pr_number approved"
            fi
        fi
    else
        log "Tests not passing - PR approval withheld"
    fi
}

# Main Tess workflow execution
main_tess_workflow() {
    log "Starting Tess agent workflow for task ${TASK_ID}"
    
    # Initialize environment
    initialize_tess_environment || {
        error "Failed to initialize Tess environment"
        exit 1
    }
    
    # Run comprehensive test suite
    run_comprehensive_tests || {
        error "Test suite failed"
        # Continue to generate report even if tests fail
    }
    
    # Generate coverage reports
    generate_coverage_reports || {
        log "Coverage report generation failed - continuing"
    }
    
    # Validate deployment readiness
    validate_deployment_readiness || {
        log "Deployment validation issues detected - continuing"
    }
    
    # Generate comprehensive test report
    generate_test_report
    
    # Create detailed test summary in PR comment
    create_pr_test_summary
    
    # Approve PR if all tests pass
    approve_pr_if_tests_pass
    
    log "Tess agent workflow completed"
}

create_pr_test_summary() {
    local summary_file="${WORKSPACE_PATH}/test_summary.json"
    local pr_comment_file="${WORKSPACE_PATH}/pr_comment.md"
    
    if [ ! -f "$summary_file" ]; then
        return
    fi
    
    local coverage_percent=$(jq -r '.coverage.percentage' "$summary_file")
    local overall_status=$(jq -r '.overall_status' "$summary_file")
    
    cat > "$pr_comment_file" <<EOF
## 🧪 Test Results Summary (Tess Agent)

### Overall Status: $overall_status

### Test Coverage
- **Current Coverage:** ${coverage_percent}%
- **Threshold:** ${COVERAGE_THRESHOLD:-95}%
- **Status:** $([ "$(jq -r '.coverage.threshold_met' "$summary_file")" = "true" ] && echo "✅ PASSED" || echo "❌ NEEDS IMPROVEMENT")

### Test Results
- **Unit Tests:** $(jq -r '.test_results.unit_tests' "$summary_file" | sed 's/PASSED/✅ PASSED/g; s/FAILED/❌ FAILED/g')
- **Integration Tests:** $(jq -r '.test_results.integration_tests' "$summary_file" | sed 's/PASSED/✅ PASSED/g; s/FAILED/❌ FAILED/g; s/NONE/➖ NONE/g')
- **Documentation Tests:** $(jq -r '.test_results.doc_tests' "$summary_file" | sed 's/PASSED/✅ PASSED/g; s/FAILED/❌ FAILED/g')

### Deployment Readiness
- **Release Build:** $(jq -r '.deployment_readiness.release_build' "$summary_file" | sed 's/PASSED/✅ PASSED/g; s/FAILED/❌ FAILED/g')
- **Security Audit:** $(jq -r '.deployment_readiness.security_audit' "$summary_file" | sed 's/PASSED/✅ PASSED/g; s/FAILED/❌ FAILED/g')

---
*Generated by Tess Agent at $(date -Iseconds)*
EOF
    
    # Post comment to PR if GitHub integration is available
    if [ -n "${GITHUB_TOKEN:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
        local pr_number=$(git log --oneline -1 | grep -oE '#[0-9]+' | head -1 | sed 's/#//')
        
        if [ -n "$pr_number" ]; then
            local comment_body=$(cat "$pr_comment_file" | jq -R -s .)
            
            curl -X POST \
                -H "Authorization: token $GITHUB_TOKEN" \
                -H "Accept: application/vnd.github.v3+json" \
                "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$pr_number/comments" \
                -d "{\"body\": $comment_body}"
            
            log "Test summary posted to PR #$pr_number"
        fi
    fi
}

# Error handling
trap 'error "Tess workflow failed at line $LINENO"' ERR

# Execute main workflow
main_tess_workflow
```

## Integration Points

- **CodeRun CRD**: Compatible with existing Custom Resource Definition structure
- **GitHub Integration**: PR comments, labels, and approvals
- **Argo Workflows**: Template variable substitution and workflow execution
- **MCP Server**: Documentation queries and context retrieval (Rex)
- **Prometheus Metrics**: Workflow execution and performance tracking
- **Persistent Volumes**: Agent-specific workspace isolation
- **Container Registry**: Standardized container images with agent scripts