# CTO Lite Packaging & Filesystem Layout

## Overview

This document defines where everything lives - binaries, config, data, and how components find each other.

---

## Application Bundle (macOS)

```
/Applications/CTO Lite.app/
└── Contents/
    ├── MacOS/
    │   └── cto-lite-tauri          # Main Tauri binary
    ├── Resources/
    │   ├── bin/                     # Bundled CLI tools
    │   │   ├── kind                 # Kubernetes in Docker
    │   │   ├── kubectl              # Kubernetes CLI
    │   │   ├── helm                 # Helm package manager
    │   │   ├── cloudflared          # Cloudflare tunnel client
    │   │   └── mcp-lite             # MCP server binary
    │   ├── charts/                  # Helm charts
    │   │   └── cto-lite/            # Our chart (bundled)
    │   ├── templates/               # Workflow templates
    │   │   └── play-workflow-lite.yaml
    │   └── agents/                  # Agent configurations (optional)
    │       └── prompts/             # Agent system prompts
    ├── Info.plist
    └── _CodeSignature/
```

## User Data Directory

```
~/Library/Application Support/ai.5dlabs.cto-lite/
├── cto-lite.db                      # SQLite database (state, history)
├── config.json                      # User configuration
├── logs/                            # Application logs
│   ├── app.log                      # Main app log
│   └── mcp.log                      # MCP server log
├── cache/                           # Cached data
│   └── images/                      # Pulled container images list
└── kubeconfig/                      # Generated kubeconfigs
    └── cto-lite-cluster.yaml        # Kind cluster kubeconfig
```

## MCP Server Configuration

The MCP server needs to be discoverable by IDEs. Two approaches:

### 1. Claude Desktop / Cline Integration

Users add to their MCP config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "cto-lite": {
      "command": "/Applications/CTO Lite.app/Contents/Resources/bin/mcp-lite",
      "args": [],
      "env": {
        "CTO_DATA_DIR": "~/Library/Application Support/ai.5dlabs.cto-lite",
        "KUBECONFIG": "~/Library/Application Support/ai.5dlabs.cto-lite/kubeconfig/cto-lite-cluster.yaml"
      }
    }
  }
}
```

### 2. Auto-Discovery (Future)

The Tauri app could write a well-known file that IDEs discover:
```
~/.cto-lite/mcp.json  →  { "socket": "...", "command": "..." }
```

---

## Binary Resolution

The Tauri app needs to find bundled binaries. Resolution order:

1. **Bundled** (production): `$APP_BUNDLE/Contents/Resources/bin/`
2. **Development**: `$CARGO_MANIFEST_DIR/../target/debug/` or PATH
3. **System PATH** (fallback): Use system-installed versions

```rust
// In Tauri backend
fn get_binary_path(name: &str) -> PathBuf {
    // Production: bundled in app
    #[cfg(not(debug_assertions))]
    {
        let app_path = std::env::current_exe().unwrap();
        app_path
            .parent() // MacOS/
            .unwrap()
            .parent() // Contents/
            .unwrap()
            .join("Resources/bin")
            .join(name)
    }
    
    // Development: use PATH
    #[cfg(debug_assertions)]
    {
        which::which(name).unwrap_or_else(|_| PathBuf::from(name))
    }
}
```

---

## Helm Chart Bundling

The `cto-lite` Helm chart is bundled with the app:

```
$APP_BUNDLE/Contents/Resources/charts/cto-lite/
├── Chart.yaml
├── values.yaml
└── templates/
    ├── _helpers.tpl
    ├── controller-deployment.yaml
    ├── pm-lite-deployment.yaml
    └── serviceaccount.yaml
```

When installing, the app runs:
```bash
helm install cto-lite $APP_BUNDLE/Contents/Resources/charts/cto-lite \
  --namespace cto-lite \
  --create-namespace \
  --set github.appId=$GITHUB_APP_ID \
  --set tunnel.token=$CLOUDFLARE_TOKEN
```

---

## Workflow Template Bundling

The workflow template is installed as a K8s resource:

```bash
kubectl apply -f $APP_BUNDLE/Contents/Resources/templates/play-workflow-lite.yaml
```

---

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `CTO_DATA_DIR` | User data directory | `~/Library/Application Support/ai.5dlabs.cto-lite` |
| `CTO_BIN_DIR` | Bundled binaries | `$APP_BUNDLE/Contents/Resources/bin` |
| `KUBECONFIG` | Kind cluster kubeconfig | `$CTO_DATA_DIR/kubeconfig/cto-lite-cluster.yaml` |
| `CTO_NAMESPACE` | Kubernetes namespace | `cto-lite` |

---

## Cross-Platform Paths

### macOS
- App: `/Applications/CTO Lite.app/`
- Data: `~/Library/Application Support/ai.5dlabs.cto-lite/`
- Logs: `~/Library/Logs/ai.5dlabs.cto-lite/`

### Windows
- App: `C:\Program Files\CTO Lite\`
- Data: `%APPDATA%\ai.5dlabs\cto-lite\`
- Logs: `%APPDATA%\ai.5dlabs\cto-lite\logs\`

### Linux
- App: `/opt/cto-lite/` or `~/.local/share/cto-lite/` (AppImage)
- Data: `~/.local/share/ai.5dlabs.cto-lite/`
- Logs: `~/.local/share/ai.5dlabs.cto-lite/logs/`

---

## Tauri Build Configuration

In `tauri.conf.json`, configure bundling:

```json
{
  "bundle": {
    "resources": [
      "resources/bin/*",
      "resources/charts/**/*",
      "resources/templates/*"
    ],
    "externalBin": []
  }
}
```

The resources are copied from the build directory during `cargo tauri build`.

---

## Build Script

We need a script that:
1. Downloads platform-specific binaries (kind, kubectl, helm, cloudflared)
2. Builds mcp-lite
3. Copies everything to `resources/`
4. Runs `cargo tauri build`

```bash
#!/bin/bash
# scripts/2026-02/build-release.sh

PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Create resources directory
mkdir -p crates/cto-lite/tauri/resources/bin
mkdir -p crates/cto-lite/tauri/resources/charts
mkdir -p crates/cto-lite/tauri/resources/templates

# Download binaries
curl -Lo resources/bin/kind "https://kind.sigs.k8s.io/dl/v0.20.0/kind-${PLATFORM}-${ARCH}"
curl -Lo resources/bin/kubectl "https://dl.k8s.io/release/v1.28.0/bin/${PLATFORM}/${ARCH}/kubectl"
curl -Lo resources/bin/helm.tar.gz "https://get.helm.sh/helm-v3.13.0-${PLATFORM}-${ARCH}.tar.gz"
# ... extract helm, download cloudflared

# Build mcp-lite
cargo build --release -p mcp-lite
cp target/release/mcp-lite resources/bin/

# Copy charts and templates
cp -r infra/charts/cto-lite resources/charts/
cp templates/workflows/play-workflow-lite.yaml resources/templates/

# Make binaries executable
chmod +x resources/bin/*

# Build Tauri app
cargo tauri build
```

---

## Next Steps

1. Create the `resources/` directory structure
2. Update Tauri config to bundle resources
3. Implement binary resolution in Rust backend
4. Create build script for downloading dependencies
5. Test packaging on macOS
