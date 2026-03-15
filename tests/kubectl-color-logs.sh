#!/usr/bin/env bash
# =============================================================================
# kubectl-color-logs.sh — Colored log viewer for multi-CLI agent pods
# =============================================================================
# Colorizes OpenClaw agent logs by CLI/component. Each CLI backend and
# system component gets a distinct ANSI color for visual separation.
#
# Usage:
#   ./kubectl-color-logs.sh [pod-name] [namespace]
#   ./kubectl-color-logs.sh openclaw-metal-openclaw-0 openclaw
#   ./kubectl-color-logs.sh                # defaults: metal pod in openclaw ns
#
# Colors:
#   Blue    — Claude Code        (ACP sessions + cliBackend)
#   Yellow  — Codex              (ACP sessions + cliBackend)
#   Magenta — Droid / Factory    (cliBackend only)
#   Cyan    — OpenCode           (ACP sessions + cliBackend)
#   Green   — Gemini             (ACP sessions + cliBackend)
#   Red     — Errors / rate limits
#   White   — Gateway / diagnostic / other
# =============================================================================

set -euo pipefail

POD="${1:-openclaw-metal-openclaw-0}"
NS="${2:-openclaw}"

# ANSI color codes
BLUE='\033[34m'
YELLOW='\033[33m'
MAGENTA='\033[35m'
CYAN='\033[36m'
GREEN='\033[32m'
RED='\033[31m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

echo -e "${BOLD}Streaming colored logs from ${POD} in ${NS}${RESET}"
echo -e "${BLUE}Claude${RESET} | ${YELLOW}Codex${RESET} | ${MAGENTA}Droid${RESET} | ${CYAN}OpenCode${RESET} | ${GREEN}Gemini${RESET} | ${RED}Error${RESET}"
echo "---"

kubectl logs "${POD}" -n "${NS}" -c agent -f --tail=50 2>/dev/null | while IFS= read -r line; do
  case "$line" in
    *claude*|*Claude*|*anthropic*|*claude-agent-acp*)
      printf "${BLUE}%s${RESET}\n" "$line"
      ;;
    *codex*|*Codex*|*codex-acp*)
      printf "${YELLOW}%s${RESET}\n" "$line"
      ;;
    *droid*|*Droid*|*factory*|*Factory*)
      printf "${MAGENTA}%s${RESET}\n" "$line"
      ;;
    *opencode*|*OpenCode*|*opencode-ai*)
      printf "${CYAN}%s${RESET}\n" "$line"
      ;;
    *gemini*|*Gemini*|*gemini-cli*)
      printf "${GREEN}%s${RESET}\n" "$line"
      ;;
    *error*|*Error*|*ERROR*|*rate.limit*|*isError=true*)
      printf "${RED}%s${RESET}\n" "$line"
      ;;
    *\[diagnostic\]*)
      printf "${DIM}%s${RESET}\n" "$line"
      ;;
    *)
      printf "%s\n" "$line"
      ;;
  esac
done
