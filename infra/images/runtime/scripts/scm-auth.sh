#!/usr/bin/env bash
# SCM Authentication Dispatcher for Container Runtime
# Routes to github-app-auth.sh or gitlab-auth.sh based on CTO_SCM_PROVIDER.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCM_PROVIDER="${CTO_SCM_PROVIDER:-github}"

case "$SCM_PROVIDER" in
  gitlab)
    echo "SCM provider: GitLab" >&2
    source "${SCRIPT_DIR}/gitlab-auth.sh"
    ;;
  github|*)
    echo "SCM provider: GitHub" >&2
    source "${SCRIPT_DIR}/github-app-auth.sh"
    ;;
esac
