#!/usr/bin/env bash

set -euo pipefail

function usage() {
  cat <<'USAGE'
Usage: ./scripts/validate-cli.sh <cli> [options] [-- <cli-args>]

Validate that a CLI image can execute a simple health command.

Arguments:
  <cli>        One of: claude, codex, cursor, factory, opencode, gemini

Options:
  --tag <tag>        Override the container tag (default: latest)
  --image <ref>      Override the full container reference
  -h, --help         Show this help message

Anything after `--` is passed directly to the CLI binary. If omitted, the script
invokes a safe default command (typically `--version`) for the selected CLI.

Examples:
  ./scripts/validate-cli.sh codex
  ./scripts/validate-cli.sh factory --tag v1.2.3
  ./scripts/validate-cli.sh cursor -- --help
USAGE
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

CLI_NAME=""
IMAGE_TAG="latest"
CUSTOM_IMAGE=""
CLI_ARGS=()

CLI_NAME="$1"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      IMAGE_TAG="${2:-}"
      shift 2
      ;;
    --image)
      CUSTOM_IMAGE="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      CLI_ARGS=("$@")
      break
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if ! command -v docker &>/dev/null; then
  echo "❌ docker is required to run CLI validation." >&2
  exit 1
fi

REGISTRY="ghcr.io/5dlabs"
IMAGE=""
BINARY=""
DEFAULT_ARGS=(--version)

case "${CLI_NAME,,}" in
  claude)
    IMAGE="${CUSTOM_IMAGE:-$REGISTRY/claude:$IMAGE_TAG}"
    BINARY="claude"
    ;;
  codex)
    IMAGE="${CUSTOM_IMAGE:-$REGISTRY/codex:$IMAGE_TAG}"
    BINARY="codex"
    ;;
  cursor)
    IMAGE="${CUSTOM_IMAGE:-$REGISTRY/cursor:$IMAGE_TAG}"
    BINARY="cursor-agent"
    ;;
  factory)
    IMAGE="${CUSTOM_IMAGE:-$REGISTRY/factory:$IMAGE_TAG}"
    BINARY="droid"
    ;;
  opencode)
    IMAGE="${CUSTOM_IMAGE:-$REGISTRY/opencode:$IMAGE_TAG}"
    BINARY="opencode"
    ;;
  gemini)
    IMAGE="${CUSTOM_IMAGE:-$REGISTRY/gemini:$IMAGE_TAG}"
    BINARY="gemini"
    ;;
  *)
    echo "Unknown CLI '${CLI_NAME}'. Expected one of: claude, codex, cursor, factory, opencode, gemini." >&2
    exit 1
    ;;
esac

if [[ ${#CLI_ARGS[@]} -eq 0 ]]; then
  CLI_ARGS=("${DEFAULT_ARGS[@]}")
fi

echo "🚀 Running ${CLI_NAME} validation via ${IMAGE}"
set -x
docker run --rm "${IMAGE}" "${BINARY}" "${CLI_ARGS[@]}"

