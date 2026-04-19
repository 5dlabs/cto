#!/usr/bin/env bash
# Mirror every active 5dlabs GitHub repo to gitlab.5dlabs.ai/5dlabs/<repo>.
#
# Pre-reqs:
#   - gh authenticated with org `5dlabs` (admin) access
#   - /tmp/gl_pat containing a GitLab admin PAT (api scope)
#   - GITHUB org secret GITLAB_PUSH_TOKEN + variable MIRROR_TO_GITLAB=true set
#     (see activate-github-mirror-secrets.sh for the one-liners)
#
# For each repo it:
#   1. Creates the GitLab project under group 5dlabs (id=3) if missing
#   2. Mirror-pushes current content GitHub -> GitLab
#   3. Adds .github/workflows/mirror-to-gitlab.yml if missing (direct commit to default branch)
#   4. Adds a minimal .gitlab-ci.yml if missing so the runner exercises the project
#
# Idempotent: re-running is safe; steps short-circuit when already done.

set -euo pipefail

GL_HOST="gitlab.5dlabs.ai"
GL_GROUP_ID=3
GL_GROUP_PATH="5dlabs"
GL_PAT="$(cat /tmp/gl_pat)"
GH_ORG="5dlabs"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

MIRROR_WORKFLOW_SRC="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)/.github/workflows/mirror-to-gitlab.yml"
STUB_CI_BODY='# Stub pipeline — GitLab mirror is active; runner will smoke-test the project.
# Replace with real CI as needed.
include:
  - project: "5dlabs/cto"
    ref: main
    file: ".gitlab/ci/templates/smoke.yml"
'

# Targets: active non-archived repos. Forks are included; test-only repos are excluded.
TARGETS=(
  swarm-openclaw-agent
  cto-agents
  agent-platform
  cto-pay
  conduit
  lab-eco
  cto-blockchain-operator
  alerthub
  documentation
  mcp-proxy
  cto-dev-cluster-setup
  cto-dev-cluster
  cto-webapp
  trading-platform
  toolman
  cto-apps
  agent-docs
  charts
  projects
  questdb-operator
  docs
  toolman-integration-test
  sigma-1
  big-balls
  cto-cameras
  Solana-tests
  # Forks
  code-server
  agentic-qe
  NemoClaw
  secureclaw
  ironclaw
  # Phase A backfill (2025-11)
  sniper
  web3-message-board
  solana
  ingestor
  golang-grpc-template
  solana-discovery
  trader-platform-old
  trader-platform
  talos-mcp
  trader
  rust-basic-api
)

gl_api() {
  curl -sk -H "PRIVATE-TOKEN: $GL_PAT" "$@"
}

ensure_gitlab_project() {
  local name="$1"
  local lc_name; lc_name="$(echo "$name" | tr '[:upper:]' '[:lower:]')"
  # GitLab project paths are case-insensitive; use lowercase path everywhere.
  local path_enc="${GL_GROUP_PATH}%2F${lc_name}"
  if gl_api -o /dev/null -w '%{http_code}' "https://${GL_HOST}/api/v4/projects/${path_enc}" | grep -q '^200$'; then
    echo "  gitlab project exists: ${GL_GROUP_PATH}/${lc_name}"
    return 0
  fi
  echo "  creating gitlab project ${GL_GROUP_PATH}/${lc_name}"
  gl_api -X POST \
    -H 'Content-Type: application/json' \
    -d "{\"name\":\"${lc_name}\",\"path\":\"${lc_name}\",\"namespace_id\":${GL_GROUP_ID},\"visibility\":\"private\",\"initialize_with_readme\":false}" \
    "https://${GL_HOST}/api/v4/projects" >/dev/null
}

mirror_push() {
  local name="$1"
  local lc_name; lc_name="$(echo "$name" | tr '[:upper:]' '[:lower:]')"
  local dir="$WORK/${name}.git"
  echo "  mirror-push github.com/${GH_ORG}/${name} -> ${GL_HOST}/${GL_GROUP_PATH}/${lc_name}"
  git clone --mirror --quiet "https://github.com/${GH_ORG}/${name}.git" "$dir"
  git -C "$dir" remote set-url --push origin "https://oauth2:${GL_PAT}@${GL_HOST}/${GL_GROUP_PATH}/${lc_name}.git"
  git -C "$dir" push --mirror --quiet 2>&1 | sed 's/^/    /'
}

add_file_if_missing() {
  local repo="$1" path="$2" message="$3" content_file="$4"
  if gh api "repos/${GH_ORG}/${repo}/contents/${path}" >/dev/null 2>&1; then
    echo "  ${path} already present"
    return 0
  fi
  echo "  adding ${path}"
  local b64; b64="$(base64 < "$content_file" | tr -d '\n')"
  local default_branch; default_branch="$(gh api "repos/${GH_ORG}/${repo}" --jq '.default_branch')"
  gh api -X PUT "repos/${GH_ORG}/${repo}/contents/${path}" \
    -f message="$message" \
    -f content="$b64" \
    -f branch="$default_branch" >/dev/null
}

FAILED=()
for repo in "${TARGETS[@]}"; do
  echo "== $repo =="
  (
    set -e
    if ! gh api "repos/${GH_ORG}/${repo}" --jq '.full_name' >/dev/null 2>&1; then
      echo "  github repo missing; skipping"; exit 0
    fi
    if [[ "$(gh api "repos/${GH_ORG}/${repo}" --jq '.archived')" == "true" ]]; then
      echo "  archived; skipping"; exit 0
    fi
    ensure_gitlab_project "$repo"
    mirror_push "$repo"

    tmp_stub="$WORK/gitlab-ci-stub.yml"
    printf '%s' "$STUB_CI_BODY" > "$tmp_stub"

    is_fork="$(gh api "repos/${GH_ORG}/${repo}" --jq '.fork')"
    if [[ "$is_fork" == "true" ]]; then
      echo "  fork — skipping workflow/stub commits (avoid diverging from upstream)"
    else
      add_file_if_missing "$repo" ".github/workflows/mirror-to-gitlab.yml" \
        "ci: mirror pushes to gitlab.5dlabs.ai" "$MIRROR_WORKFLOW_SRC"
      add_file_if_missing "$repo" ".gitlab-ci.yml" \
        "ci: add GitLab smoke pipeline" "$tmp_stub"
    fi
  ) || { echo "  !! repo $repo failed, continuing"; FAILED+=("$repo"); }
done

echo "done."
if [[ ${#FAILED[@]} -gt 0 ]]; then
  echo "FAILED repos:"
  printf '  %s\n' "${FAILED[@]}"
fi
