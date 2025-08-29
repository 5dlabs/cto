# Autonomous Agent Prompt: Enhance Rex Container for Remediation

## Your Mission
You are tasked with creating a separate, specialized Rex remediation container that handles targeted fixes for QA feedback. This container must differ fundamentally from the normal Rex implementation container by focusing on surgical fixes rather than broad implementation.

## Context
The existing Rex container is designed for feature implementation from scratch. Remediation requires a different approach:

**Current Rex Container**: Builds features with full context and broad implementation scope  
**New Remediation Container**: Fixes specific issues with targeted, surgical changes

Your implementation will create a dedicated remediation workflow that understands both original requirements and specific feedback, enabling precise fixes while preserving working functionality.

## Required Actions

### 1. Create Separate Rex Remediation Container Script
Build a new dedicated remediation script at `infra/images/rex-remediation/container-rex-remediation.sh`:
- Implement strict validation requiring `REMEDIATION_MODE=true`
- Create clear error messages for incorrect usage
- Separate completely from normal Rex implementation script
- Design script structure with modular functions
- Add comprehensive logging and progress indicators

### 2. Implement Original Task Context Fetching
Create robust task context retrieval system:
- Search multiple potential sources for task documentation
- Try `/workspace/docs/task-{id}.md`, `.taskmaster/docs/task-{id}/task.md`
- Fallback to Task Master API if files unavailable
- Handle missing context gracefully with appropriate warnings
- Export context for template generation

### 3. Add GitHub API Comment Fetching for Feedback
Integrate with GitHub CLI for feedback retrieval:
- Use `gh api` to fetch comment by ID from environment variable
- Implement retry logic with exponential backoff
- Validate authentication and repository access
- Extract comment body, author, and metadata
- Handle API rate limits and network failures

### 4. Build Feedback Parser for Remediation Context
Create intelligent feedback processing:
- Parse markdown content for severity indicators
- Extract issue types (Bug, Performance, Security, Documentation)
- Identify specific issues from bullet points and numbered lists
- Count and track checkbox completion status
- Format parsed data for AI context generation

### 5. Implement Iteration Limit Checking
Build escalation system for maximum iterations:
- Check `ITERATION_COUNT` against maximum of 10
- Generate escalation comments for PR when limit reached
- Tag appropriate team members (@platform-team, @cto)
- Provide clear summary of remediation attempts
- Terminate process gracefully when escalation needed

### 6. Create Remediation-Specific AI Context
Generate targeted context file (`CLAUDE.md`):
- Combine original task requirements with specific feedback
- Emphasize fix-focused instructions vs. implementation
- Include iteration count and urgency indicators
- Provide clear DO/DON'T guidelines for targeted fixes
- Format for optimal AI comprehension and action

### 7. Configure Container Image and Deployment
Set up container infrastructure:
- Create Dockerfile with required tools (bash, gh CLI, jq, curl)
- Install Claude runner and set proper permissions
- Copy remediation scripts with executable permissions
- Configure workspace volume mount
- Set appropriate entrypoint for remediation mode

### 8. Test Remediation Container End-to-End
Build comprehensive test suite:
- Unit tests for all bash functions with mocked dependencies
- Integration tests with real GitHub API
- Container build and deployment verification
- End-to-end remediation workflow testing
- Performance and resource usage validation

## Technical Requirements

### Container Script Structure
```bash
#!/bin/bash
# Rex Remediation Container Script - Separate from normal Rex
set -euo pipefail

if [ "$REMEDIATION_MODE" != "true" ]; then
    echo "❌ Error: This script requires REMEDIATION_MODE=true"
    exit 1
fi

# Main workflow
main() {
    check_iteration_limits
    fetch_original_task_context
    fetch_feedback_comment  
    parse_feedback_content
    generate_remediation_context
    exec /usr/local/bin/claude-runner
}
```

### Environment Variables
- `REMEDIATION_MODE`: Must be "true" for script activation
- `TASK_ID`: Identifier for task context retrieval
- `PR_NUMBER`: Pull request number for API calls
- `FEEDBACK_COMMENT_ID`: GitHub comment ID to retrieve
- `ITERATION_COUNT`: Current remediation iteration number

### Context Template Structure
```markdown
# REMEDIATION MODE - Fix Required Issues

**Mode**: Remediation (Iteration X/10)
**Task ID**: {task-id}

## Original Task Requirements
{original-context}

## Issues to Fix (Priority: {severity})
{feedback-content}

## Remediation Instructions
1. Address ALL issues in feedback
2. Preserve working functionality  
3. Make targeted fixes only
4. Focus on specific problems
```

## Implementation Checklist

### Core Development
- [ ] Create separate remediation script with strict validation
- [ ] Implement original task context fetching from multiple sources
- [ ] Build GitHub API integration with proper error handling
- [ ] Create intelligent feedback parsing and categorization
- [ ] Implement iteration limit checking with escalation
- [ ] Generate remediation-specific AI context templates

### Container Infrastructure  
- [ ] Create Dockerfile with all required dependencies
- [ ] Configure proper entrypoint and permissions
- [ ] Set up workspace volume mounting
- [ ] Ensure GitHub CLI authentication works
- [ ] Install and configure Claude runner

### Testing and Validation
- [ ] Build unit test suite for all functions
- [ ] Create integration tests with GitHub API
- [ ] Test container build and deployment process
- [ ] Validate end-to-end remediation workflow
- [ ] Performance test with various feedback sizes

### Integration
- [ ] Ensure compatibility with existing CodeRun CRD
- [ ] Verify integration with play workflow sensors
- [ ] Test authentication with existing GitHub App
- [ ] Validate workspace access and permissions

## Expected Outputs

1. **Remediation Script**: Complete bash script with all required functions
2. **Container Image**: Docker image with proper dependencies and configuration
3. **Context Template**: AI-optimized template for remediation guidance
4. **Test Suite**: Comprehensive testing for all functionality
5. **Documentation**: Clear usage instructions and troubleshooting guide

## Success Validation

Your implementation is successful when:
1. Script only activates with `REMEDIATION_MODE=true`
2. Successfully retrieves original task context from various sources
3. Fetches GitHub PR comments reliably via API
4. Parses feedback intelligently with metadata extraction
5. Enforces iteration limits with proper escalation
6. Generates targeted AI context focused on fixes
7. Container builds and runs without errors
8. Integration with existing workflow functions correctly
9. Tests validate all functionality thoroughly
10. Performance meets operational requirements

## Technical Constraints

### Separation Requirements
- Complete separation from normal Rex implementation container
- Different script, different approach, different context generation
- Clear validation that prevents incorrect usage
- Distinct logging and error messages for clarity

### GitHub Integration
- Use existing GitHub App authentication
- Respect API rate limits and implement backoff
- Handle various comment formats and sizes
- Extract meaningful metadata for context

### Context Generation  
- Focus on fix-specific instructions vs. broad implementation
- Preserve original requirements for context
- Emphasize targeted changes over reimplementation
- Provide clear success criteria for fixes

## Common Pitfalls to Avoid

- Don't reuse normal Rex container logic - create separate approach
- Avoid hardcoding repository or authentication details
- Don't ignore iteration limits - enforce strictly
- Ensure robust error handling for all external API calls
- Don't generate context that encourages reimplementation
- Avoid complex bash constructs that are hard to test
- Don't skip validation of required environment variables

## Resources and References

- GitHub CLI documentation: https://cli.github.com/manual/
- GitHub API reference for issue comments
- Existing Rex container implementation for patterns
- CodeRun CRD specification for integration requirements
- Bash scripting best practices and testing frameworks

## Support and Troubleshooting

If you encounter issues:
1. Validate all environment variables are properly set
2. Test GitHub CLI authentication independently
3. Check API rate limits and quotas
4. Verify repository access and permissions
5. Test container builds in isolation
6. Validate workspace volume mounting
7. Check Claude runner installation and permissions

Begin by examining the existing Rex container structure to understand current patterns, then create a fundamentally different approach optimized for remediation workflows. Focus on surgical precision rather than broad implementation scope.