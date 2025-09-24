# Task 1: Initialize Project Structure and Dependencies

## Overview
Set up the foundational project structure with Rust workspace for controller, TypeScript for MCP integration, and Docker build infrastructure for multi-CLI support. This task establishes the foundation for the entire Multi-CLI Agent Platform project.

## Context
The 5D Labs Agent Platform currently operates exclusively with Claude Code CLI. This task begins the implementation of CLI-agnostic support by establishing the proper project structure that will support 8 different CLI tools (Claude, Codex, Opencode, Gemini, Grok, Qwen, Cursor, OpenHands).

## Technical Specification

### 1. Rust Workspace Setup
- **Location**: Root directory with `Cargo.toml`
- **Structure**: Workspace with controller crate
- **Dependencies**:
  - `kube-rs v0.95.0` for Kubernetes client integration
  - `tokio v1.41.0` for async runtime
  - Additional Rust crates for CLI adapters

### 2. Controller Implementation
- **Path**: `controller/src/main.rs`
- **Purpose**: Basic Kubernetes client initialization
- **Features**:
  - Kubernetes cluster connection
  - CRD handling setup
  - Error handling patterns

### 3. MCP Integration Setup
- **Location**: `mcp/` directory
- **Technology**: TypeScript/Node.js
- **Dependencies**: `@modelcontextprotocol/sdk v1.0.4`
- **Purpose**: Model Context Protocol integration layer

### 4. Container Infrastructure
- **Path**: `infra/images/`
- **Structure**: Subdirectories for each CLI:
  - `claude/` - Claude Code CLI
  - `codex/` - OpenAI Codex CLI
  - `opencode/` - Opencode CLI
  - `gemini/` - Google Gemini CLI
  - `grok/` - Grok CLI
  - `qwen/` - Qwen CLI
  - `cursor/` - Cursor CLI
  - `openhands/` - OpenHands CLI

### 5. CI/CD Pipeline
- **Path**: `.github/workflows/build-images.yml`
- **Purpose**: Automated multi-arch container builds
- **Features**:
  - Docker buildx integration
  - Multi-architecture support (amd64, arm64)
  - Container registry push

### 6. Build System
- **File**: Root `Makefile`
- **Targets**:
  - `build`: Build all components
  - `test`: Run test suites
  - `deploy`: Deploy to cluster
  - `clean`: Clean build artifacts

## Implementation Steps

### Phase 1: Workspace Initialization
1. Create root `Cargo.toml` with workspace configuration
2. Initialize `controller/` directory with basic crate structure
3. Set up initial dependencies in `controller/Cargo.toml`

### Phase 2: MCP Setup
1. Create `mcp/` directory with `package.json`
2. Install TypeScript dependencies and MCP SDK
3. Set up TypeScript configuration

### Phase 3: Container Infrastructure
1. Create `infra/images/` directory structure
2. Add placeholder Dockerfiles for each CLI
3. Set up base image configurations

### Phase 4: CI/CD Integration
1. Create GitHub Actions workflow
2. Configure Docker buildx for multi-arch builds
3. Set up container registry integration

### Phase 5: Build System
1. Create comprehensive Makefile
2. Add development and deployment targets
3. Configure git repository with appropriate .gitignore

## Dependencies
- Docker and Docker buildx
- Rust toolchain (1.70+)
- Node.js (18+) and npm
- Kubernetes cluster access
- GitHub Actions environment

## Success Criteria
- `cargo check` passes for Rust workspace
- `npm install` succeeds in MCP directory
- Docker build infrastructure validates
- GitHub Actions workflow can be tested locally
- All Makefile targets execute correctly

## Files Created
```
├── Cargo.toml (workspace)
├── controller/
│   ├── Cargo.toml
│   └── src/main.rs
├── mcp/
│   ├── package.json
│   └── tsconfig.json
├── infra/images/
│   ├── claude/Dockerfile
│   ├── codex/Dockerfile
│   ├── opencode/Dockerfile
│   ├── gemini/Dockerfile
│   ├── grok/Dockerfile
│   ├── qwen/Dockerfile
│   ├── cursor/Dockerfile
│   └── openhands/Dockerfile
├── .github/workflows/build-images.yml
├── Makefile
└── .gitignore
```

## Next Steps
After completion, this task enables:
- Task 2: CLI-Aware Model Validation Framework
- Task 3: CLI Adapter Trait System
- Container builds for specific CLI implementations

## Risk Mitigation
- Version pin all dependencies to ensure reproducible builds
- Use multi-stage Docker builds to optimize image sizes
- Implement comprehensive error handling in controller
- Set up automated security scanning for containers