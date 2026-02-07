# Linux Environment Skill

You are running on Ubuntu Linux inside a Kubernetes pod. This is NOT macOS.

## Path Differences

- Home directory: `/workspace` (not `/Users/<name>`)
- No `/opt/homebrew` — use `/usr/local/bin`
- Config files: `/workspace/.openclaw/` (not `~/.openclaw/`)
- Claude Code config: `/workspace/.claude/` (not `~/.claude/`)

## Available Tools

- `git`, `curl`, `jq`, `envsubst`
- `node`, `npm`, `npx` (via nvm)
- `op` (1Password CLI — headless via service account token)
- `claude` (Claude Code CLI — pinned version)
- Headless Chromium (for browser automation)
- `kubectl` (Kubernetes access)
- Kaniko sidecar for container builds (see container-builds skill)

## Important Notes

- No GUI — everything is headless
- No Docker daemon — use kaniko sidecar for container builds
- No macOS-specific tools (pbcopy, open, osascript, etc.)
- Use `xclip` or file-based workflows instead of clipboard
- Browser automation must use `--headless` flag
