# Toolman Guide: Ready-for-QA Label Logic

## Overview

This task implements the critical handoff mechanism between Cleo's code quality work and Tess's comprehensive testing phase. You'll create logic for Cleo to add 'ready-for-qa' label to PRs as an explicit signal that triggers Tess workflow resumption.

## Tool Selection Strategy

### Primary Development Tools

**filesystem** - Essential for script development and container template modification
- Create and modify container-cleo.sh.hbs with ready-for-qa workflow
- Develop GitHub API integration scripts for label management
- Create CI status monitoring and validation scripts
- Organize workflow coordination and helper scripts

**git** - Required for repository interaction and branch analysis
- Extract task ID and context from branch names
- Test commit and push operations in quality workflow
- Track changes to container templates and scripts
- Validate branch-based PR discovery logic

**github** - Critical for GitHub API operations and PR management
- Test GitHub API authentication and token generation
- Implement PR discovery and label management operations
- Test GitHub CLI integration for label operations
- Validate PR status and CI check monitoring

### Research and Documentation Tools

**memory_create_entities** - Store implementation knowledge
- Document ready-for-qa workflow sequence and requirements
- Track GitHub API patterns and authentication approaches
- Remember CI status checking logic and timeout strategies
- Store testing scenarios and validation approaches

**brave_web_search** - Supplemental research tool
- Research GitHub API label management best practices
- Find GitHub Actions CI status checking examples
- Research GitHub CLI usage patterns and authentication
- Lookup event-driven workflow coordination patterns

## Implementation Workflow

### Phase 1: Research and Planning
```
Tools: filesystem, memory_create_entities, brave_web_search
```

1. **Analyze Existing Container Templates**
   - Use `filesystem` to examine current container script patterns
   - Study existing GitHub authentication and API integration
   - Document current Cleo workflow structure and capabilities

2. **Research GitHub API Integration**
   - Use `brave_web_search` for GitHub API label management examples
   - Study GitHub Actions CI status API and monitoring patterns
   - Plan authentication flow using GitHub App credentials

3. **Design Ready-for-QA Workflow**
   - Use `memory_create_entities` to document complete workflow sequence
   - Plan CI status monitoring strategy and timeout handling
   - Design event-driven integration with Argo Events sensors

### Phase 2: Script Development
```
Tools: filesystem, git, github, memory_create_entities
```

1. **Create CI Status Monitoring Script**
   ```bash
   # Focus areas for wait-for-ci-success.sh
   - GitHub Actions status API integration
   - Robust polling with timeout and error handling
   - Check state analysis (pending, success, failure)
   - Clear logging and progress reporting
   ```

2. **Implement Label Management Script**
   ```bash
   # Focus areas for add-ready-for-qa-label.sh
   - Idempotent label addition logic
   - GitHub API authentication and error handling
   - Label existence verification before addition
   - Success confirmation and validation
   ```

3. **Create PR Context Setup Script**
   ```bash
   # Focus areas for setup-pr-context.sh
   - Branch analysis and task ID extraction
   - PR discovery using branch or task correlation
   - Context export for coordination between scripts
   - Error handling for missing or invalid PRs
   ```

### Phase 3: Container Template Integration
```
Tools: filesystem, git, memory_create_entities
```

1. **Update Cleo Container Template**
   ```handlebars
   # Focus areas for container-cleo.sh.hbs
   - Integration of ready-for-qa workflow sequence
   - GitHub authentication setup and token generation
   - Script coordination and error propagation
   - Status reporting and workflow completion signaling
   ```

2. **Test Workflow Coordination**
   - Test script integration and parameter passing
   - Validate error handling and rollback scenarios
   - Test GitHub API operations within container context

3. **Create Tess Prerequisites Logic**
   ```handlebars
   # Focus areas for container-tess.sh.hbs integration
   - Ready-for-qa label validation before starting
   - Prerequisites check and error reporting
   - Graceful waiting and workflow coordination
   ```

### Phase 4: Event Integration and Testing
```
Tools: filesystem, github, memory_create_entities
```

1. **Create Argo Events Sensor Configuration**
   ```yaml
   # Focus areas for ready-for-qa sensor
   - GitHub webhook event filtering for label events
   - Sender validation (5DLabs-Cleo[bot] only)
   - Task ID extraction and workflow correlation
   - Workflow resumption trigger configuration
   ```

2. **Integration Testing**
   - Test complete Cleo workflow with real PRs
   - Test event detection and workflow resumption
   - Test concurrent workflow coordination

3. **Error Scenario Testing**
   - Test CI failure handling and workflow termination
   - Test GitHub API error recovery and retry logic
   - Test missing PR and invalid context scenarios

## Best Practices

### Workflow Design Principles
- **Explicit Handoff**: Clear, visible signal between agent phases
- **Quality Gates**: No label addition until all quality requirements met
- **Idempotent Operations**: All operations safe to retry without side effects
- **Error Recovery**: Graceful handling of all failure scenarios

### GitHub API Integration
- **Authentication Security**: Secure handling of GitHub App credentials
- **Rate Limit Handling**: Respect GitHub API rate limits and retry logic
- **Error Resilience**: Handle API failures and network issues gracefully
- **Audit Logging**: Log all API operations for debugging and monitoring

### Script Organization
- **Single Responsibility**: Each script has clear, focused responsibility
- **Error Propagation**: Failures properly communicated up the workflow chain
- **Context Sharing**: Clean parameter passing and context management
- **Resource Cleanup**: Proper cleanup of temporary files and resources

## Tool Usage Examples

### Reading Existing Templates
```bash
# Use filesystem to examine current container scripts
filesystem.read_file("infra/charts/controller/claude-templates/container.sh.hbs")
filesystem.list_directory("infra/charts/controller/claude-templates/")
```

### Testing GitHub Operations
```bash
# Use github for API operations testing
github.get_file("README.md")  # Test authentication
github.create_pull_request({"title": "Test PR", "body": "Testing"})  # Test PR operations
```

### Branch Analysis
```bash
# Use git for branch and context analysis
git.status()  # Check current branch state
git.log()     # Analyze recent commits and branch history
git.show()    # Examine specific commits and changes
```

### Creating Scripts and Templates
```bash
# Use filesystem to create workflow scripts
filesystem.write_file("scripts/wait-for-ci-success.sh", ci_script)
filesystem.write_file("scripts/add-ready-for-qa-label.sh", label_script)
filesystem.write_file("scripts/setup-pr-context.sh", context_script)
```

## Common Pitfalls to Avoid

1. **Non-Idempotent Operations**: Ensure all label operations safe to retry
2. **CI Status Misinterpretation**: Correctly identify relevant CI checks and states
3. **GitHub API Authentication**: Properly handle token generation and authentication
4. **Race Conditions**: Handle concurrent operations and event ordering safely
5. **Context Loss**: Maintain PR and task context across workflow steps
6. **Error Masking**: Don't hide errors that should cause workflow failures

## Workflow Implementation Patterns

### Cleo Workflow Structure
```handlebars
{{#if (eq github_app "5DLabs-Cleo")}}
# Phase 1: Setup and Authentication
source setup-pr-context.sh
export GITHUB_TOKEN=$(generate-github-token.sh)

# Phase 2: Code Quality Work
run-quality-checks.sh
commit-and-push-fixes.sh

# Phase 3: CI Validation
wait-for-ci-success.sh "$PR_NUMBER"

# Phase 4: Ready-for-QA Handoff
add-ready-for-qa-label.sh "$PR_NUMBER"
{{/if}}
```

### GitHub API Script Pattern
```bash
#!/bin/bash
set -euo pipefail

# Input validation
PR_NUMBER="$1"
if [ -z "$PR_NUMBER" ]; then
    echo "Usage: script.sh <pr_number>"
    exit 1
fi

# Authentication check
if [ -z "${GITHUB_TOKEN:-}" ]; then
    echo "Error: GITHUB_TOKEN not set"
    exit 1
fi

# Main operation with error handling
if ! gh pr edit "$PR_NUMBER" --add-label "ready-for-qa"; then
    echo "Error: Failed to add label"
    exit 1
fi

echo "Success: Label added to PR #$PR_NUMBER"
```

### Tess Prerequisites Pattern
```handlebars
{{#if (eq github_app "5DLabs-Tess")}}
# Validate ready-for-qa prerequisite
if ! validate-ready-for-qa-prerequisite.sh; then
    echo "Prerequisites not met, exiting gracefully"
    exit 0
fi

echo "Ready-for-qa confirmed, starting comprehensive testing"
{{/if}}
```

## Success Validation

### Workflow Quality Checks
- [ ] Complete Cleo workflow executes without errors
- [ ] CI status monitoring correctly identifies test completion
- [ ] Ready-for-qa label added only after all prerequisites met
- [ ] Tess detects and responds to ready-for-qa label correctly

### GitHub Integration Quality Checks
- [ ] GitHub API authentication works reliably
- [ ] PR discovery and context setup works for all branch patterns
- [ ] Label operations are idempotent and error-resistant
- [ ] GitHub CLI integration functions correctly in container environment

### Event Integration Quality Checks
- [ ] Argo Events sensor detects ready-for-qa label events
- [ ] Task ID extraction and workflow correlation works correctly
- [ ] Workflow resumption triggers at correct stage
- [ ] Multiple concurrent workflows don't interfere

### Error Handling Quality Checks
- [ ] CI failures prevent ready-for-qa label addition
- [ ] GitHub API failures handled gracefully with proper error messages
- [ ] Missing PR scenarios handled without crashing
- [ ] Network issues and timeouts handled appropriately

This implementation requires careful attention to GitHub API integration and event-driven coordination. Focus on creating reliable, idempotent operations that provide clear handoff signals while handling all error scenarios gracefully.