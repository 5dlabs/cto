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

### Step 1: Current Architecture Analysis
1. Analyze existing Rust workspace structure in `controller/` directory
2. Document current Kubernetes controller implementation and CRDs
3. Review existing MCP server capabilities in `mcp/` directory
4. Assess current CLI integration patterns and supported types

### Step 2: CLI Documentation Research
1. **Use CLI documentation tools extensively** to research each CLI:
   - `agent_docs_codex_query` for OpenAI Codex installation, configuration, and MCP integration
   - `agent_docs_cursor_query` for Cursor CLI setup, authentication, and usage patterns
   - `agent_docs_opencode_query` for OpenCode installation, MCP server support, and configuration
   - `agent_docs_gemini_query` for Google Gemini setup, authentication, and model configuration
   - `agent_docs_grok_query` for Grok CLI installation, X.AI integration, and MCP tools support
   - `agent_docs_qwen_query` for Qwen setup, model handling, and API configuration
   - `agent_docs_openhands_query` for OpenHands Python dependencies, CLI usage, and automation
2. Document CLI-specific requirements, authentication patterns, and configuration formats
3. Identify which CLIs support MCP natively vs. need wrapper integration

### Step 3: Container Infrastructure Assessment
1. Review existing Docker images in `infra/images/` for all 8 CLI types
2. Test build processes and identify functional vs. broken images
3. Cross-reference CLI documentation research with actual Dockerfile implementations
4. Document configuration requirements per CLI based on research findings

### Step 4: Deployment and CI/CD Review
1. Analyze existing GitOps configurations in `infra/gitops/`
2. Review current GitHub Actions workflows and automation
3. Test current ArgoCD application deployments
4. Document manual deployment steps and automation gaps

### Step 5: Research Findings Documentation
1. **Create comprehensive CLI Research Findings Document** consolidating all research
2. Provide clear gap analysis between current state and requirements
3. Prioritize missing functionality and recommend next development phases
4. Document CLI-specific integration patterns and authentication requirements

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
Your analysis and research is successful when:
- ✅ **Comprehensive CLI Research Findings Document** is created with detailed analysis
- ✅ All CLI documentation tools have been used to research each CLI type
- ✅ Current architecture state is thoroughly documented
- ✅ Container infrastructure status is assessed and documented
- ✅ Gap analysis clearly identifies what exists vs. what's needed
- ✅ Prioritized recommendations provide clear next steps
- ✅ CLI-specific integration patterns and requirements are documented
- ✅ Research findings enable informed development decisions

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

Remember: Your research and analysis will guide the development of the entire Multi-CLI Agent Platform. Thorough documentation and comprehensive CLI research are paramount. Use all available CLI documentation tools extensively to provide the most informed analysis possible for the 8 different CLI tools and their unique requirements.