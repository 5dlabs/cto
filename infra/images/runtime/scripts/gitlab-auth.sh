#!/usr/bin/env bash
# GitLab Authentication for Container Runtime
# Parallel to github-app-auth.sh for dual SCM support.
# Uses GITLAB_TOKEN (PAT) - no JWT/App installation flow needed.

set -euo pipefail

GL_HOST="${GITLAB_HOST:-git.5dlabs.ai}"

if [ -z "${GITLAB_TOKEN:-}" ]; then
  echo "ERROR: GITLAB_TOKEN is not set" >&2
  exit 1
fi

# Configure git credentials
git config --global credential.helper store
echo "https://oauth2:${GITLAB_TOKEN}@${GL_HOST}" > ~/.git-credentials
chmod 600 ~/.git-credentials
git config --global "url.https://${GL_HOST}/.insteadOf" "git@${GL_HOST}:"

# Export for child processes
export GIT_AUTHOR_NAME="${GIT_AUTHOR_NAME:-cto-bot}"
export GIT_AUTHOR_EMAIL="${GIT_AUTHOR_EMAIL:-bot@${GL_HOST}}"
export GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME"
export GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"

# Configure glab if present
if command -v glab >/dev/null 2>&1; then
  echo "${GITLAB_TOKEN}" | glab auth login --hostname "${GL_HOST}" --stdin 2>/dev/null || true
fi

echo "GitLab auth configured for ${GL_HOST}" >&2
