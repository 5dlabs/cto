# AI Agent Prompt: Initialize Project Structure and Dependencies

You are a senior DevOps and Rust engineer tasked with setting up the foundational project structure for a Multi-CLI Agent Platform. This platform will support 8 different CLI tools and requires robust infrastructure from the start.

## Your Mission
Analyze the existing comprehensive project structure that includes:
1. Rust workspace with controller implementation
2. Rust-based MCP integration layer  
3. Existing Docker infrastructure for 8 different CLI containers
4. CI/CD pipeline with GitHub Actions
5. Current build system and development workflow

## Technical Requirements

### Rust Workspace Setup
- Create `Cargo.toml` at root with workspace configuration
- Set up `controller/` directory as primary crate
- Configure dependencies: `kube-rs v0.95.0`, `tokio v1.41.0`
- Implement basic Kubernetes client in `controller/src/main.rs`
- Follow Rust best practices for error handling and async patterns

### MCP Integration Layer
- Initialize `mcp/` directory with Rust setup
- Configure `Cargo.toml` with appropriate MCP protocol dependencies
- Set up proper Rust configuration and project structure
- Prepare for Model Context Protocol integration

### Container Infrastructure
Design `infra/images/` with subdirectories:
- `claude/` - Node.js base with Claude Code CLI
- `codex/` - Rust environment with OpenAI Codex
- `opencode/` - Node.js environment for OpenCode CLI
- `gemini/` - Node.js environment for Google Gemini CLI
- `grok/` - Bun/Node.js environment for Grok CLI
- `qwen/` - Node.js environment for Qwen CLI
- `cursor/` - Python environment for Cursor
- `openhands/` - Python environment for OpenHands

### GitHub Actions Configuration
Create `.github/workflows/build-images.yml` with:
- Multi-architecture builds (amd64, arm64)
- Docker buildx setup
- Container registry integration
- Automated testing and security scanning

### Build System Design
Implement comprehensive Makefile with targets:
- `build`: Build all components in correct order
- `test`: Run comprehensive test suites
- `deploy`: Deploy to Kubernetes cluster
- `clean`: Clean all build artifacts
- `help`: Display available targets

## Implementation Approach

### Step 1: Project Foundation
1. Initialize git repository with comprehensive .gitignore
2. Create workspace Cargo.toml with member crates
3. Set up controller crate with basic structure
4. Configure initial logging and error handling

### Step 2: Development Environment
1. Set up MCP directory with Rust configuration
2. Install and configure Rust development dependencies
3. Create initial Cargo.toml with proper workspace setup
4. Configure Rust project structure and modules

### Step 3: Container Strategy
1. Create Dockerfile templates for each CLI type
2. Plan base image strategy for optimal layer sharing
3. Set up health checks and security scanning
4. Configure multi-stage builds for size optimization

### Step 4: CI/CD Pipeline
1. Design GitHub Actions workflow for automated builds
2. Configure Docker buildx for cross-platform builds
3. Set up container registry push with proper tagging
4. Implement security scanning and compliance checks

### Step 5: Build Automation
1. Create comprehensive Makefile with all targets
2. Add development convenience scripts
3. Configure proper dependency management
4. Set up local testing capabilities

## Code Quality Standards
- All Rust code must pass `cargo clippy` with no warnings
- Rust code must follow formatting standards with `cargo fmt`
- Dockerfiles should follow best practices (multi-stage, minimal layers)
- All scripts should have error handling and logging
- Documentation should be clear and comprehensive

## Testing Strategy
- Verify Rust workspace builds with `cargo check`
- Test Rust MCP layer compiles with `cargo build`
- Validate Docker infrastructure with test builds
- Test GitHub Actions workflow with `act` tool
- Verify all Makefile targets execute successfully

## Success Criteria
Your implementation is successful when:
- ✅ Complete project structure is created
- ✅ Rust workspace builds without errors
- ✅ Rust-based MCP layer compiles successfully
- ✅ All container Dockerfiles are valid
- ✅ GitHub Actions workflow is properly configured
- ✅ Makefile targets execute correctly
- ✅ Git repository is properly initialized with .gitignore

## Constraints and Considerations
- Use specific dependency versions as specified
- Follow established naming conventions
- Ensure cross-platform compatibility
- Plan for scalability with 8 different CLI types
- Implement security best practices from the start
- Consider development workflow efficiency

## Deliverables
1. **CLI Research Findings Document**: Comprehensive report documenting your research findings, including:
   - Current architecture analysis (Rust controller, MCP server, existing infrastructure)
   - CLI-specific research results from documentation queries
   - Container infrastructure assessment  
   - Gap analysis between documented requirements and current implementation
   - Prioritized recommendations for next development phases
2. Assessment of existing Rust workspace and controller implementation
3. Analysis of current Rust-based MCP integration capabilities
4. Status report on existing Docker infrastructure for all 8 CLI types
5. Review of current CI/CD pipeline and deployment processes
6. Documentation of current build system and development workflow
7. Clear roadmap for completing the CLI-agnostic platform

Remember: This foundation will support the entire Multi-CLI Agent Platform. Quality and extensibility are paramount. Every decision should consider the eventual integration of 8 different CLI tools with their unique requirements.