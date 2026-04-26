#!/usr/bin/env bash
# SCM dispatch helpers for dual GitHub/GitLab operation.
# Source this file to get provider-agnostic SCM functions.
# Provider selected via CTO_SCM_PROVIDER (default: github).

SCM_PROVIDER="${CTO_SCM_PROVIDER:-github}"
SCM_GITLAB_HOST="${GITLAB_HOST:-git.5dlabs.ai}"
SCM_GITLAB_API="${GITLAB_API_BASE:-https://git.5dlabs.ai/api/v4}"

scm_create_repo() {
  local full_name="$1" visibility="$2" description="${3:-}"
  case "$SCM_PROVIDER" in
    github)
      gh repo create "$full_name" --"$visibility" --description "$description" --clone=false 2>&1 || true
      ;;
    gitlab)
      local group="${full_name%%/*}"
      local name="${full_name##*/}"
      curl -sf -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
        -H "Content-Type: application/json" \
        "${SCM_GITLAB_API}/projects" \
        -d "{\"name\":\"${name}\",\"namespace_path\":\"${group}\",\"visibility\":\"${visibility}\",\"initialize_with_readme\":true,\"description\":\"${description}\"}" 2>&1 || true
      ;;
    *)
      echo "ERROR: unknown SCM_PROVIDER=$SCM_PROVIDER" >&2
      return 1
      ;;
  esac
}

scm_create_pr() {
  local title="$1" body="$2" base="${3:-main}" extra="${4:-}"
  case "$SCM_PROVIDER" in
    github)
      gh pr create --title "$title" --body "$body" --base "$base" $extra 2>&1
      ;;
    gitlab)
      local branch
      branch=$(git rev-parse --abbrev-ref HEAD)
      local pid
      pid=$(scm_project_id)
      curl -sf -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
        -H "Content-Type: application/json" \
        "${SCM_GITLAB_API}/projects/${pid}/merge_requests" \
        -d "{\"title\":\"${title}\",\"description\":\"${body}\",\"source_branch\":\"${branch}\",\"target_branch\":\"${base}\"}" 2>&1
      ;;
  esac
}

scm_init_file() {
  local repo_full="$1" file_path="$2" content_b64="$3" message="${4:-chore: initialize}"
  case "$SCM_PROVIDER" in
    github)
      gh api "repos/${repo_full}/contents/${file_path}" \
        -X PUT -f message="$message" -f content="$content_b64" --silent 2>&1 || true
      ;;
    gitlab)
      local pid
      pid=$(urlencoding "$repo_full")
      local encoded_path
      encoded_path=$(urlencoding "$file_path")
      curl -sf -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
        -H "Content-Type: application/json" \
        "${SCM_GITLAB_API}/projects/${pid}/repository/files/${encoded_path}" \
        -X POST \
        -d "{\"branch\":\"main\",\"content\":\"$(echo "$content_b64" | base64 -d)\",\"commit_message\":\"${message}\"}" 2>&1 || true
      ;;
  esac
}

scm_create_webhook() {
  local repo_full="$1" webhook_url="$2"
  case "$SCM_PROVIDER" in
    github)
      gh api "repos/${repo_full}/hooks" \
        -X POST \
        -f name=web \
        -f "config[url]=${webhook_url}" \
        -f "config[content_type]=json" \
        -f "config[insecure_ssl]=0" \
        -f "events[]=pull_request" \
        -f "events[]=check_run" \
        -f "events[]=issue_comment" \
        -f active=true \
        --silent 2>&1 || echo "Webhook may already exist" >&2
      ;;
    gitlab)
      local pid
      pid=$(urlencoding "$repo_full")
      local token_payload=""
      if [ -n "${GITLAB_WEBHOOK_SECRET:-}" ]; then
        token_payload=",\"token\":\"${GITLAB_WEBHOOK_SECRET}\""
      fi
      curl -sf -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
        -H "Content-Type: application/json" \
        "${SCM_GITLAB_API}/projects/${pid}/hooks" \
        -X POST \
        -d "{\"url\":\"${webhook_url}\",\"merge_requests_events\":true,\"pipeline_events\":true,\"note_events\":true,\"push_events\":false,\"enable_ssl_verification\":true${token_payload}}" 2>&1 || echo "Webhook may already exist" >&2
      ;;
  esac
}

scm_pr_url_pattern() {
  case "$SCM_PROVIDER" in
    github) echo 'https://github\.com/[^[:space:]]+/pull/[0-9]+' ;;
    gitlab) echo "https://${SCM_GITLAB_HOST}/[^[:space:]]+-/merge_requests/[0-9]+" ;;
  esac
}

scm_repo_url() {
  local full_name="$1"
  case "$SCM_PROVIDER" in
    github) echo "https://github.com/${full_name}" ;;
    gitlab) echo "https://${SCM_GITLAB_HOST}/${full_name}" ;;
  esac
}

scm_project_id() {
  local remote_url
  remote_url=$(git remote get-url origin 2>/dev/null || echo "")
  case "$SCM_PROVIDER" in
    gitlab)
      local path
      path=$(echo "$remote_url" | sed "s|https://${SCM_GITLAB_HOST}/||;s|\.git$||;s|git@${SCM_GITLAB_HOST}:||")
      urlencoding "$path"
      ;;
    github)
      echo "$remote_url" | sed 's|https://github.com/||;s|\.git$||;s|git@github.com:||'
      ;;
  esac
}

urlencoding() {
  python3 -c "import urllib.parse; print(urllib.parse.quote('$1', safe=''))" 2>/dev/null || echo "$1" | sed 's|/|%2F|g'
}
