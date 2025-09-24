# AI Agent Prompt: Initialize Project Structure and Dependencies

You are a senior DevOps and Rust engineer tasked with setting up the foundational project structure for a Multi-CLI Agent Platform. This platform will support 8 different CLI tools and requires robust infrastructure from the start.

## Your Mission
Create a comprehensive project structure that includes:
1. Rust workspace with controller implementation
2. TypeScript/Node.js MCP integration layer
3. Docker infrastructure for 8 different CLI containers
4. CI/CD pipeline with GitHub Actions
5. Comprehensive build system with Makefile

## Technical Requirements

### Rust Workspace Setup
- Create `Cargo.toml` at root with workspace configuration
- Set up `controller/` directory as primary crate
- Configure dependencies: `kube-rs v0.95.0`, `tokio v1.41.0`
- Implement basic Kubernetes client in `controller/src/main.rs`
- Follow Rust best practices for error handling and async patterns

### MCP Integration Layer
- Initialize `mcp/` directory with TypeScript setup
- Configure `package.json` with `@modelcontextprotocol/sdk v1.0.4`
- Set up proper TypeScript configuration
- Prepare for Model Context Protocol integration

### Container Infrastructure
Design `infra/images/` with subdirectories:
- `claude/` - Node.js base with Claude Code CLI
- `codex/` - Rust environment with OpenAI Codex
- `opencode/` - TypeScript/Node.js for Opencode
- `gemini/` - TypeScript/Node.js for Google Gemini
- `grok/` - TypeScript/Node.js for Grok CLI
- `qwen/` - TypeScript/Node.js for Qwen CLI
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
1. Set up MCP directory with TypeScript configuration
2. Install and configure development dependencies
3. Create initial package.json with scripts
4. Configure TypeScript compiler options

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
- TypeScript must compile without errors and pass linting
- Dockerfiles should follow best practices (multi-stage, minimal layers)
- All scripts should have error handling and logging
- Documentation should be clear and comprehensive

## Testing Strategy
- Verify Rust workspace builds with `cargo check`
- Test TypeScript setup with `npm install` and compilation
- Validate Docker infrastructure with test builds
- Test GitHub Actions workflow with `act` tool
- Verify all Makefile targets execute successfully

## Success Criteria
Your implementation is successful when:
- ✅ Complete project structure is created
- ✅ Rust workspace builds without errors
- ✅ TypeScript MCP layer compiles successfully
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
1. Complete project directory structure
2. Working Rust workspace with controller foundation
3. Configured TypeScript MCP integration layer
4. Docker infrastructure for all 8 CLI types
5. Functional GitHub Actions CI/CD pipeline
6. Comprehensive Makefile with all targets
7. Proper git repository setup

Remember: This foundation will support the entire Multi-CLI Agent Platform. Quality and extensibility are paramount. Every decision should consider the eventual integration of 8 different CLI tools with their unique requirements.