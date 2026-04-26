# CTO Desktop Rename Notes

This repository now uses `crates/cto-lite` as the single desktop app implementation, with product-facing branding set to **CTO**.

## Renamed (user-facing)

- Tauri product name and window title: `CTO`
- Bundle identifier: `ai.5dlabs.cto`
- UI copy: `CTO Lite` / `CTO App` -> `CTO`
- MCP default server label: `cto`

## Compatibility preserved (internal/runtime IDs)

The following are intentionally unchanged for MVP compatibility with existing local installs and cluster assets:

- Kubernetes namespace/release/cluster naming using `cto-lite`
- Helm chart path/name under `infra/charts/cto-lite`
- Existing database filename (`cto-lite.db`)

## Migration behavior

On startup, the desktop app now migrates legacy app data from:

- macOS/Linux: `.../ai.5dlabs.cto-lite`
- Windows: `.../ai.5dlabs/cto-lite`

to the new app data directory keyed by `ai.5dlabs.cto` when the new directory does not yet exist.

Keychain credentials now use service name `cto` with a fallback read from legacy `cto-lite`, and migrate forward on access.
