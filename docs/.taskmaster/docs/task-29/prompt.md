# Autonomous Agent Prompt: Agent-Specific Container Scripts for Multi-Agent Workflows



## Objective

Develop three specialized container scripts (container-rex.sh.hbs, container-cleo.sh.hbs, container-tess.sh.hbs) that implement distinct workflows tailored to each agent's specific responsibilities while maintaining compatibility with the existing CRD structure. Each script must provide agent-specific functionality optimized for their role in the development pipeline.

## Context

You are implementing the execution layer for the Task Master multi-agent system where Rex, Cleo, and Tess agents have distinct responsibilities in the development workflow. Each agent requires a specialized container script that implements their specific workflow while integrating seamlessly with the existing Kubernetes-based orchestration system.

## Agent-Specific Requirements

### Rex Agent (Documentation-Driven Implementation)

Implement documentation-first development workflow:



1. **MCP Documentation Integration**


   - Query MCP documentation server before implementation using `rustdocs_query_rust_docs`


   - Pull relevant API documentation, architecture guides, and existing patterns


   - Create implementation context from documentation analysis


   - Generate implementation plans based on documented patterns



2. **Documentation-First Approach**


   - Implement comprehensive inline documentation referencing source docs


   - Create code with extensive comments explaining architectural decisions


   - Generate README files with usage examples and architectural overview


   - Include documentation validation steps to ensure code matches documented behavior



3. **Git Integration**


   - Stage changes with descriptive commit messages referencing documentation


   - Include co-author attribution to MCP documentation server


   - Create meaningful commit history that explains implementation decisions

### Cleo Agent (Code Quality and Formatting Workflow)

Implement comprehensive code quality pipeline:



1. **Formatting Pipeline**


   - Run `cargo fmt --check` to identify formatting issues


   - Apply `cargo fmt` to auto-fix formatting problems


   - Generate formatting reports showing changes made


   - Ensure consistent code style across entire project



2. **Linting and Quality Checks**


   - Execute `cargo clippy` with strict warning levels (deny warnings)


   - Generate clippy fix suggestions and apply automatically where possible


   - Run additional linters for configuration files (prettier, yamllint)


   - Perform import organization and dead code detection



3. **Quality Reporting**


   - Create comprehensive quality reports summarizing all fixes


   - Generate before/after comparisons for quality metrics


   - Label PR with 'ready-for-qa' after successful quality checks


   - Commit changes with detailed messages explaining quality improvements

### Tess Agent (Testing and Deployment Validation)

Implement comprehensive testing and validation workflow:



1. **Test Suite Execution**
   - Execute comprehensive test suite: `cargo test --all-features`


   - Run integration tests with `cargo test --test '*'`


   - Execute documentation tests and example code


   - Run property-based tests using proptest or quickcheck



2. **Coverage Analysis**


   - Generate coverage reports using cargo llvm-cov or tarpaulin


   - Validate coverage meets thresholds (95% existing, 100% new code)


   - Create detailed coverage reports with line-by-line analysis


   - Identify uncovered code paths and suggest additional tests



3. **Deployment Validation**
   - Perform deployment readiness check: `cargo build --release`
   - Run performance benchmarks if available: `cargo bench`


   - Execute security audits using `cargo audit`


   - Validate configuration files and deployment manifests



4. **PR Management**


   - Approve PR automatically if all tests pass and coverage requirements met


   - Create detailed test summary in PR comments


   - Block deployment if tests fail or coverage is insufficient


   - Generate actionable feedback for test failures

## Template Structure Requirements

### Common Script Elements

All scripts must implement:



1. **Environment Setup**


   - Inherit base environment from shared template variables


   - Use consistent error handling with `set -euo pipefail`


   - Implement structured logging with timestamps and agent identification


   - Support dry-run mode for testing without making changes



2. **Template Variables Integration**
   - `{{github_app}}`: Agent identifier for conditional logic
   - `{{task_id}}`: Task reference for correlation and reporting
   - `{{workspace_path}}`: Agent-specific workspace location
   - `{{github_token}}`: Authentication for GitHub API interactions


   - Agent-specific variables (MCP server URL, coverage thresholds, quality rules)



3. **Error Handling and Retry Logic**


   - Implement retry logic for transient failures (network, API calls)


   - Include telemetry hooks for workflow monitoring


   - Handle GitHub API rate limits gracefully


   - Provide meaningful error messages and debugging information



4. **Workspace Management**


   - Support workspace isolation using agent-specific PVCs


   - Handle concurrent execution without conflicts


   - Implement proper cleanup on failure or timeout


   - Maintain workspace state across workflow stages

## Implementation Specifications

### Script Location and Naming

Place scripts in `infra/charts/controller/claude-templates/`:


- `container-rex.sh.hbs` - Rex agent documentation-driven workflow


- `container-cleo.sh.hbs` - Cleo agent code quality workflow


- `container-tess.sh.hbs` - Tess agent testing and validation workflow

### Integration Requirements

Scripts must integrate with:



1. **CodeRun CRD Structure**


   - Support existing Custom Resource Definition fields


   - Handle workflow stage transitions with proper status updates


   - Emit metrics for Grafana dashboard monitoring


   - Update workflow labels and annotations appropriately



2. **Kubernetes Integration**


   - Support status updates via Kubernetes API


   - Handle pod lifecycle events properly


   - Implement health checks and readiness probes


   - Support graceful shutdown and cleanup



3. **GitHub Integration**


   - Create PR comments with agent-specific reports


   - Update PR labels based on workflow outcomes


   - Handle GitHub API authentication and rate limiting


   - Support PR approvals and status checks



## Expected Deliverables



1. **Rex Agent Script (container-rex.sh.hbs)**


   - MCP documentation server integration


   - Documentation-first implementation workflow


   - Comprehensive inline documentation generation


   - Git integration with descriptive commit messages



2. **Cleo Agent Script (container-cleo.sh.hbs)**


   - Comprehensive formatting and linting pipeline


   - Quality metrics collection and reporting


   - Automatic fix application where possible


   - PR labeling and quality gate implementation



3. **Tess Agent Script (container-tess.sh.hbs)**


   - Comprehensive test suite execution


   - Coverage analysis and threshold validation


   - Deployment readiness validation


   - Automated PR approval based on test results



4. **Integration Components**


   - Template variable handling for all agent types


   - Common error handling and logging functions


   - Shared utility functions for GitHub API interactions


   - Monitoring and metrics emission

## Acceptance Criteria



- Rex correctly queries MCP documentation server and incorporates guidance


- Cleo applies all formatting and linting fixes with comprehensive reporting


- Tess runs complete test suite and validates coverage thresholds accurately


- All scripts integrate properly with existing CodeRun CRD structure


- GitHub PR interactions work correctly (comments, labels, approvals)


- Workspace isolation prevents conflicts between concurrent agents


- Error handling provides meaningful feedback for debugging


- Scripts support both local testing and production deployment

## Quality Standards



- Follow bash scripting best practices with proper error handling


- Implement comprehensive logging with structured output


- Use template variables correctly with proper escaping


- Include inline documentation explaining complex logic


- Test all error paths and edge cases thoroughly


- Ensure backward compatibility with existing workflows


- Implement proper security practices for token handling


- Optimize for performance and resource usage

## Performance Requirements



- Script initialization completes within 30 seconds
- Agent-specific workflows complete within reasonable timeframes:
  - Rex documentation queries: <2 minutes
  - Cleo quality checks: <5 minutes
  - Tess comprehensive testing: <15 minutes


- GitHub API interactions handle rate limits gracefully


- Workspace operations don't impact concurrent workflows


- Memory usage stays within container limits

## Security Considerations



- Handle GitHub tokens securely without logging sensitive data


- Implement input validation for all external data


- Use least-privilege principles for file system access


- Sanitize outputs before logging or displaying


- Validate all template variable inputs


- Implement secure temporary file handling


- Follow container security best practices



## Resources



- Bash scripting best practices and error handling patterns


- GitHub API documentation for PR management and status updates


- Cargo tool documentation for testing, formatting, and linting


- Kubernetes API documentation for status updates and metrics


- Handlebars template syntax for variable substitution


- MCP server API documentation for documentation queries

Focus on creating robust, efficient, and maintainable scripts that provide clear separation of concerns while enabling seamless multi-agent collaboration in the development pipeline.
