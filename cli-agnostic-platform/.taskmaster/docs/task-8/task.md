# Task 8: Create Multi-CLI Container Images

## Overview
Build optimized Docker images for each CLI with proper runtime environments, dependencies, and health checks. This provides the containerized execution environment for all 8 CLI tools.

## Technical Specification

### 1. Container Strategy by CLI Type

#### Node.js CLIs (Claude, Opencode, Gemini, Grok, Qwen)
```dockerfile
FROM node:22-slim
RUN apt-get update && apt-get install -y curl git
RUN npm install -g @anthropic-ai/claude-code
COPY entrypoint.sh /usr/local/bin/
HEALTHCHECK --interval=30s CMD claude --version
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
```

#### Rust CLI (Codex)
```dockerfile
FROM rust:1.83-slim
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo install --locked @openai/codex
COPY entrypoint.sh /usr/local/bin/
HEALTHCHECK --interval=30s CMD codex --version
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
```

#### Python CLIs (Cursor, OpenHands)
```dockerfile
FROM python:3.12-slim
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"
RUN pip install cursor-agent openhands-ai
COPY entrypoint.sh /usr/local/bin/
HEALTHCHECK --interval=30s CMD python -c "import cursor; print('OK')"
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
```

### 2. Multi-Stage Build Optimization
```dockerfile
# Build stage
FROM node:22 as builder
WORKDIR /build
COPY package*.json ./
RUN npm ci --production

# Runtime stage
FROM node:22-slim
COPY --from=builder /build/node_modules ./node_modules
COPY --from=builder /usr/local/bin/claude /usr/local/bin/
```

### 3. Security and Compliance
- Trivy vulnerability scanning in CI pipeline
- SBOM generation with syft
- Image signing with cosign
- Non-root user execution
- Minimal attack surface

## Implementation Steps

### Phase 1: Base Images
1. Create Dockerfiles for each CLI type
2. Implement multi-stage builds for size optimization
3. Add health check scripts
4. Configure entrypoint scripts

### Phase 2: Security Hardening
1. Add Trivy scanning to CI pipeline
2. Implement SBOM generation
3. Set up cosign image signing
4. Security baseline compliance

### Phase 3: Multi-Architecture Support
1. Configure buildx for linux/amd64 and linux/arm64
2. Matrix builds in GitHub Actions
3. Layer caching optimization
4. Build performance optimization

## Success Criteria
- Image sizes: <500MB (Node), <700MB (Rust), <800MB (Python)
- Health checks return proper status codes
- CLI commands execute successfully in containers
- Multi-arch images work on both amd64 and arm64
- Zero high/critical vulnerabilities in scans
- Signed images verify with cosign