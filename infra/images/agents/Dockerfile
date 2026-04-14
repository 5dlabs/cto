# syntax=docker/dockerfile:1
# =============================================================================
# Unified Agent Image
# =============================================================================
#
# Single container image for ALL CTO platform agents. Agent identity is
# determined at runtime via environment variables and mounted config, NOT
# by which image is used.
#
# This replaces the previous per-agent image model (openclaw, claude, cursor,
# codex, etc.) with a single image that contains every tool an agent might
# need. The OpenClaw orchestrator selects which CLI to invoke based on the
# agent's configuration in cto-config.json.
#
# Runtime identity (set via env vars or K8s pod spec):
#   AGENT_NAME      - Agent identifier (e.g., "rex", "morgan", "blaze")
#   AGENT_ROLE      - Role description (e.g., "rust-engineer", "pm", "frontend")
#   AGENT_CLI       - Preferred coding CLI (claude, cursor, codex, etc.)
#   AGENT_MODEL     - Default LLM model for this agent
#   GITHUB_APP_NAME - GitHub App identity (e.g., "5DLabs-Rex")
#
# Mounted at runtime:
#   /etc/cto/config.json   - Canonical cto-config.json (read-only)
#   /etc/cto/workflows/    - Lobster workflow definitions
#   /etc/cto/templates/    - Agent prompt templates and skills
#
# Build:
#   docker build -t registry.5dlabs.ai/5dlabs/agents .
#   docker build --build-arg OPENCLAW_VERSION=1.2.3 -t registry.5dlabs.ai/5dlabs/agents:1.2.3 .
#
# =============================================================================

ARG BASE_IMAGE=registry.5dlabs.ai/5dlabs/runtime:full

FROM ${BASE_IMAGE}

LABEL org.opencontainers.image.source="https://github.com/5dlabs/cto"
LABEL org.opencontainers.image.description="CTO AI Agent - Unified multi-agent platform image"
LABEL org.opencontainers.image.licenses="MIT"

USER root

# =============================================================================
# 1Password CLI (op) - headless secret access for agents
# =============================================================================
RUN set -eux; \
    OP_VERSION="v2.30.3"; \
    ARCH=$(dpkg --print-architecture); \
    case "$ARCH" in \
      amd64) OP_ARCH=amd64 ;; \
      arm64) OP_ARCH=arm64 ;; \
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac; \
    curl -fsSL "https://cache.agilebits.com/dist/1P/op2/pkg/${OP_VERSION}/op_linux_${OP_ARCH}_${OP_VERSION}.zip" -o /tmp/op.zip && \
    unzip -o /tmp/op.zip -d /usr/local/bin && \
    rm -f /tmp/op.zip && \
    chmod +x /usr/local/bin/op && \
    op --version

# =============================================================================
# npm CLIs - All coding agent backends
# =============================================================================

# Bun runtime - required to compile intake-util during image build
RUN set -eux; \
    curl -fsSL https://bun.sh/install | bash; \
    ln -sf /root/.bun/bin/bun /usr/local/bin/bun; \
    ln -sf /root/.bun/bin/bunx /usr/local/bin/bunx; \
    bun --version

# Deno runtime - sandbox for agent TypeScript code execution (MCP tool scripts)
RUN set -eux; \
    curl -fsSL https://deno.land/install.sh | sh; \
    ln -sf /root/.deno/bin/deno /usr/local/bin/deno; \
    deno --version

# Core coding CLIs (claude, copilot, codex, every-code, gemini, openai SDK)
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund \
      @anthropic-ai/claude-code \
      @github/copilot \
      @openai/codex \
      @just-every/code \
      @google/gemini-cli \
      openai@latest \
    && echo "npm CLIs installed: claude, copilot, codex, code, gemini, openai"

# =============================================================================
# OpenClaw (agent CLI) - orchestrator that delegates to claude/codex/cursor/etc.
# =============================================================================
ARG OPENCLAW_VERSION=2026.3.24
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    if [ "$OPENCLAW_VERSION" = "latest" ]; then \
        npm install -g --no-audit --no-fund --ignore-scripts openclaw; \
    else \
        npm install -g --no-audit --no-fund --ignore-scripts openclaw@${OPENCLAW_VERSION}; \
    fi && \
    # --ignore-scripts skips postinstall that resolves deps like tslog.
    # Run a scoped `npm install` inside the package dir to pull missing deps.
    cd /usr/local/share/npm-global/lib/node_modules/openclaw && \
    npm install --no-audit --no-fund --ignore-scripts=false 2>&1 | tail -10 && \
    cd / && \
    echo "--- OpenClaw (agent CLI) included ---" && \
    openclaw --version

# =============================================================================
# acpx — ACP runtime for dispatching to external coding CLIs
# =============================================================================
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund acpx@latest && \
    echo "acpx installed" && \
    acpx --version

# =============================================================================
# ACP adapters — pre-installed so acpx doesn't need runtime npm downloads
# CLIs with native ACP: droid, cursor, gemini, copilot, kimi, opencode
# CLIs needing adapters: codex → codex-acp, claude → claude-agent-acp
# =============================================================================
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund \
      @zed-industries/codex-acp@latest \
      @agentclientprotocol/claude-agent-acp@latest && \
    echo "ACP adapters installed:" && \
    codex-acp --help 2>&1 | head -1 || true && \
    claude-agent-acp --help 2>&1 | head -1 || true

# Patch acpx: enable shell mode for terminal/create on all platforms.
# Without this, agents that send full command strings (e.g. "git status")
# in terminal/create requests fail with ENOENT on Linux/macOS because
# Node.js spawn() without shell:true treats the string as a literal binary.
# Upstream: https://github.com/openclaw/acpx/issues/169 (fixed only for Windows)
RUN ACPX_PROMPT_TURN=$(find "$(npm root -g)/acpx/dist" -name 'prompt-turn-*.js' ! -name '*.map' -type f | head -1) && \
    if [ -n "$ACPX_PROMPT_TURN" ]; then \
      LINE=$(grep -n 'shouldUseWindowsBatchShell(command, platform, env)) return options' "$ACPX_PROMPT_TURN" | head -1 | cut -d: -f1) && \
      sed -i "${LINE}s/.*/\t\/\/ PATCHED: always use shell for terminal commands/" "$ACPX_PROMPT_TURN" && \
      echo "acpx patched line $LINE in $ACPX_PROMPT_TURN" && \
      sed -n "${LINE}p" "$ACPX_PROMPT_TURN"; \
    else \
      echo "WARNING: acpx prompt-turn file not found, patch skipped"; \
    fi

# =============================================================================
# ClawHub CLI — skill registry search and installation
# =============================================================================
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund clawhub@latest && \
    echo "clawhub installed" && \
    clawhub --cli-version

# =============================================================================
# intake-util — deterministic pipeline operations (tally, sync-linear, write-files)
# =============================================================================
COPY apps/intake-util/ /tmp/intake-util-build/
RUN cd /tmp/intake-util-build && \
    npm ci --ignore-scripts && \
    npm run build && \
    cp intake-util /usr/local/bin/intake-util && \
    chmod +x /usr/local/bin/intake-util && \
    cd / && \
    rm -rf /tmp/intake-util-build && \
    intake-util --help >/dev/null 2>&1 || true

# =============================================================================
# Cursor Agent CLI
# =============================================================================
RUN set -eux; \
    curl https://cursor.com/install -fsS | bash; \
    export PATH="$HOME/.local/bin:$PATH"; \
    CURSOR_SHARE="$HOME/.local/share/cursor-agent"; \
    VERSION_DIR="$(cd "$CURSOR_SHARE/versions" && ls -1 | sort | tail -n1)"; \
    install -d /usr/local/lib/cursor-agent; \
    cp -r "$CURSOR_SHARE/versions/$VERSION_DIR"/. /usr/local/lib/cursor-agent/; \
    rm -f /usr/local/lib/cursor-agent/node; \
    ln -sf "$(which node)" /usr/local/lib/cursor-agent/node; \
    cp "$HOME/.local/bin/agent" /usr/local/bin/agent; \
    chmod +x /usr/local/bin/agent; \
    sed -i 's|SCRIPT_DIR=.*|SCRIPT_DIR="/usr/local/lib/cursor-agent"|' /usr/local/bin/agent; \
    ln -sf agent /usr/local/bin/cursor-agent; \
    ln -sf agent /usr/local/bin/cursor; \
    chown -R 1000:1000 /usr/local/lib/cursor-agent; \
    rm -rf "$CURSOR_SHARE"; \
    agent --version

# =============================================================================
# Factory Droid CLI
# =============================================================================
RUN set -eux; \
    curl -fsSL https://app.factory.ai/cli | sh; \
    install -m 0755 /root/.local/bin/droid /usr/local/bin/droid; \
    rm -f /root/.local/bin/droid; \
    install -d -m 0755 /home/node/.factory/bin /home/node/.factory/logs /home/node/.factory/cache; \
    ln -sf /usr/local/bin/rg /home/node/.factory/bin/rg 2>/dev/null || true; \
    printf '{}' > /home/node/.factory/settings.json; \
    chown -R 1000:1000 /home/node/.factory; \
    FACTORY_DISABLE_TELEMETRY=1 droid --version

# =============================================================================
# Kimi Code CLI
# =============================================================================
RUN set -eux; \
    curl -LsSf https://astral.sh/uv/install.sh | bash; \
    . "$HOME/.local/bin/env"; \
    UV_TOOL_DIR=/opt/uv-tools UV_TOOL_BIN_DIR=/usr/local/bin uv tool install kimi-cli; \
    chmod -R a+rX /opt/uv-tools; \
    kimi --version || echo "kimi installed (version check may require config)"

# =============================================================================
# OpenCode CLI - binary from GitHub releases
# =============================================================================
ARG OPENCODE_VERSION=1.1.44
RUN set -eux; \
    ARCH=$(uname -m); \
    case "$ARCH" in \
        x86_64) ARCH_NAME="x64" ;; \
        aarch64|arm64) ARCH_NAME="arm64" ;; \
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/anomalyco/opencode/releases/download/v${OPENCODE_VERSION}/opencode-linux-${ARCH_NAME}.tar.gz" -o /tmp/opencode.tar.gz; \
    tar -xzf /tmp/opencode.tar.gz -C /tmp; \
    install -m 0755 /tmp/opencode /usr/local/bin/opencode; \
    rm -rf /tmp/opencode.tar.gz /tmp/opencode; \
    opencode --version

# =============================================================================
# GitHub CLI (gh) - PR creation, issue management, repo operations
# =============================================================================
RUN (type -p wget >/dev/null || (apt-get update && apt-get install wget -y)) \
    && mkdir -p -m 755 /etc/apt/keyrings \
    && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg | tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
    && chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install gh -y \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# =============================================================================
# kubectl — Kubernetes cluster access for agent operations
# =============================================================================
RUN ARCH=$(dpkg --print-architecture) && \
    KUBE_VERSION=$(curl -sL https://dl.k8s.io/release/stable.txt) && \
    curl -sLO "https://dl.k8s.io/release/${KUBE_VERSION}/bin/linux/${ARCH}/kubectl" && \
    install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl && \
    rm kubectl && \
    kubectl version --client

# =============================================================================
# ArgoCD CLI — GitOps application sync and status
# =============================================================================
RUN ARCH=$(dpkg --print-architecture) && \
    ARGOCD_VERSION=$(curl -sL https://raw.githubusercontent.com/argoproj/argo-cd/stable/VERSION) && \
    curl -sLO "https://github.com/argoproj/argo-cd/releases/download/v${ARGOCD_VERSION}/argocd-linux-${ARCH}" && \
    install -o root -g root -m 0755 "argocd-linux-${ARCH}" /usr/local/bin/argocd && \
    rm "argocd-linux-${ARCH}" && \
    argocd version --client

# =============================================================================
# Solana CLI — blockchain development toolchain (Block agent)
# Anchor requires Rust/Cargo which is installed at task time if needed.
# =============================================================================
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
      SOLANA_VERSION="2.2.14" && \
      curl -sSfL "https://github.com/anza-xyz/agave/releases/download/v${SOLANA_VERSION}/solana-release-x86_64-unknown-linux-gnu.tar.bz2" \
        -o /tmp/solana.tar.bz2 && \
      tar xjf /tmp/solana.tar.bz2 -C /tmp && \
      cp /tmp/solana-release/bin/solana /usr/local/bin/ && \
      cp /tmp/solana-release/bin/spl-token /usr/local/bin/ && \
      cp /tmp/solana-release/bin/solana-keygen /usr/local/bin/ && \
      cp /tmp/solana-release/bin/solana-test-validator /usr/local/bin/ && \
      rm -rf /tmp/solana.tar.bz2 /tmp/solana-release && \
      solana --version; \
    else \
      echo "Skipping Solana CLI install for ${ARCH} (amd64 only)"; \
    fi

# =============================================================================
# Foundry (EVM toolchain — forge, cast, anvil, chisel)
# =============================================================================
RUN curl -sSfL https://foundry.paradigm.xyz | bash && \
    /root/.foundry/bin/foundryup && \
    cp /root/.foundry/bin/forge /usr/local/bin/ && \
    cp /root/.foundry/bin/cast /usr/local/bin/ && \
    cp /root/.foundry/bin/anvil /usr/local/bin/ && \
    cp /root/.foundry/bin/chisel /usr/local/bin/ && \
    forge --version

# =============================================================================
# System packages — SSH server, browser automation, file watcher
# =============================================================================
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      openssh-server \
      inotify-tools \
      gettext-base \
      wget \
      gnupg2 \
      ca-certificates \
      ffmpeg && \
    ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
      mkdir -p /etc/apt/keyrings && \
      wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /etc/apt/keyrings/google-chrome.gpg && \
      echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/google-chrome.gpg] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list && \
      apt-get update && \
      DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        google-chrome-stable; \
    else \
      echo "Skipping Google Chrome install for ${ARCH}; Playwright-managed browsers will be used on arm64."; \
    fi && \
    mkdir -p /run/sshd && \
    rm -rf /var/lib/apt/lists/*

# =============================================================================
# Agent runtime directories and config mount points
# =============================================================================

# Per-CLI home directories
RUN mkdir -p /home/node/.openclaw /home/node/.claude /home/node/.code \
             /home/node/.codex /home/node/.copilot /home/node/.cursor /home/node/.gemini \
             /home/node/.kimi /home/node/.opencode /home/node/.config /home/node/.cache \
             /home/node/.local /workspace && \
    chown -R 1000:1000 /home/node/.openclaw /home/node/.claude /home/node/.code \
                       /home/node/.codex /home/node/.copilot /home/node/.cursor /home/node/.gemini \
                       /home/node/.kimi /home/node/.opencode /home/node/.factory /home/node/.config \
                       /home/node/.cache /home/node/.local /workspace

# Config mount points (populated at runtime via K8s ConfigMap/Secret mounts)
RUN mkdir -p /etc/cto/workflows /etc/cto/templates /etc/cto/skills && \
    chown -R 1000:1000 /etc/cto

# Configure git identity (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

# =============================================================================
# Agent entrypoint script
# =============================================================================
# This script reads AGENT_* env vars to configure the agent's identity at
# startup, then delegates to openclaw which orchestrates the actual work.
COPY infra/images/agents/entrypoint.sh /usr/local/bin/agent-entrypoint
RUN chmod +x /usr/local/bin/agent-entrypoint

ENV HOME=/home/node

# =============================================================================
# Version manifest — bake every CLI version into the image
# (Must run before USER node since /etc/cto requires root or explicit chown)
# =============================================================================
RUN set -e; \
    V_OPENCLAW=$(openclaw --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_ACPX=$(acpx --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_CLAUDE=$(claude --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_COPILOT=$(copilot --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_CODEX=$(codex --version 2>&1 | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -1 || echo "unknown"); \
    V_CODE=$(code --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_GEMINI=$(timeout 10 gemini --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_CURSOR=$(agent --version 2>&1 | head -1 | sed 's/^[[:space:]]*//' || echo "unknown"); \
    V_DROID=$(FACTORY_DISABLE_TELEMETRY=1 droid --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_KIMI=$(kimi --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_OPENCODE=$(opencode --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_CLAWHUB=$(clawhub --cli-version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_GH=$(gh --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_KUBECTL=$(kubectl version --client -o json 2>&1 | grep -oE '"gitVersion":"v[0-9]+\.[0-9]+\.[0-9]+"' | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown"); \
    V_ARGOCD=$(argocd version --client --short 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_SOLANA=$(solana --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "n/a"); \
    V_FORGE=$(forge --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_OP=$(op --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_NODE=$(node --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    V_BUN=$(bun --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown"); \
    BUILD_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ); \
    printf '{\n  "buildDate": "%s",\n  "clis": {\n    "openclaw": "%s",\n    "acpx": "%s",\n    "claude": "%s",\n    "copilot": "%s",\n    "codex": "%s",\n    "code": "%s",\n    "gemini": "%s",\n    "cursor": "%s",\n    "droid": "%s",\n    "kimi": "%s",\n    "opencode": "%s",\n    "clawhub": "%s"\n  },\n  "tools": {\n    "gh": "%s",\n    "kubectl": "%s",\n    "argocd": "%s",\n    "op": "%s",\n    "node": "%s",\n    "bun": "%s"\n  }\n}\n' \
      "$BUILD_DATE" \
      "$V_OPENCLAW" "$V_ACPX" "$V_CLAUDE" "$V_COPILOT" "$V_CODEX" "$V_CODE" "$V_GEMINI" \
      "$V_CURSOR" "$V_DROID" "$V_KIMI" "$V_OPENCODE" "$V_CLAWHUB" \
      "$V_GH" "$V_KUBECTL" "$V_ARGOCD" "$V_OP" "$V_NODE" "$V_BUN" \
      > /etc/cto/versions.json; \
    echo "=== /etc/cto/versions.json ==="; \
    cat /etc/cto/versions.json; \
    echo ""; \
    echo "openclaw       $V_OPENCLAW"; \
    echo "acpx           $V_ACPX"; \
    echo "claude         $V_CLAUDE"; \
    echo "copilot        $V_COPILOT"; \
    echo "codex          $V_CODEX"; \
    echo "code           $V_CODE"; \
    echo "gemini         $V_GEMINI"; \
    echo "cursor-agent   $V_CURSOR"; \
    echo "droid          $V_DROID"; \
    echo "kimi           $V_KIMI"; \
    echo "opencode       $V_OPENCODE"; \
    echo "clawhub        $V_CLAWHUB"; \
    echo "gh             $V_GH"; \
    echo "kubectl        $V_KUBECTL"; \
    echo "argocd         $V_ARGOCD"; \
    echo "solana         $V_SOLANA"; \
    echo "forge          $V_FORGE"; \
    echo "op             $V_OP"; \
    echo "node           $V_NODE"; \
    echo "bun            $V_BUN"; \
    intake-util --help >/dev/null 2>&1 && echo "intake-util    OK" || true

# Final ownership sweep — everything under /home/node must be writable by node (uid 1000).
# Root-owned dotfiles (e.g. .gitconfig, .npmrc) created during image build break runtime git/npm.
RUN find /home/node -maxdepth 1 -not -user 1000 -exec chown 1000:1000 {} + 2>/dev/null || true

# Drop privileges back to non-root user
USER node
WORKDIR /workspace

# Health check — confirms the openclaw gateway process is running
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD ps aux | grep -q "[o]penclaw gateway" || exit 1

# Default: run the agent entrypoint which reads identity from env vars
ENTRYPOINT ["agent-entrypoint"]
CMD ["--help"]
