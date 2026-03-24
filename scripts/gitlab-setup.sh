#!/usr/bin/env bash
# GitLab CE Post-Deploy Setup
# Run after GitLab is accessible at git.5dlabs.ai
#
# This script automates:
#   1. Wait for GitLab to be ready
#   2. Create OAuth application for CTO agent platform
#   3. Import repositories from GitHub
#   4. Register runner and store token in OpenBao
#   5. Create scheduled pipeline for agent builds
#   6. Store secrets in OpenBao
#
# Prerequisites:
#   - GitLab CE running at git.5dlabs.ai
#   - kubectl access to the cluster
#   - GITLAB_ROOT_PASSWORD set or available from OpenBao
#   - GITHUB_TOKEN set (for repo import)

set -euo pipefail

GITLAB_URL="${GITLAB_URL:-https://git.5dlabs.ai}"
GITLAB_API="${GITLAB_URL}/api/v4"
GITHUB_ORG="5dlabs"
GITLAB_GROUP="5dlabs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warning() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $*"; }
log_header()  { echo -e "\n${BLUE}━━━ $* ━━━${NC}"; }

# Get GitLab root token (PAT created via API with root password)
get_root_token() {
    if [[ -n "${GITLAB_TOKEN:-}" ]]; then
        echo "$GITLAB_TOKEN"
        return
    fi

    # Try to get root password from OpenBao
    local root_password
    root_password=$(kubectl exec -n openbao openbao-0 -- bao kv get -field=root-password secret/gitlab 2>/dev/null || echo "")

    if [[ -z "$root_password" ]]; then
        root_password="${GITLAB_ROOT_PASSWORD:-}"
    fi

    if [[ -z "$root_password" ]]; then
        log_error "No GitLab root password available. Set GITLAB_ROOT_PASSWORD or GITLAB_TOKEN"
        exit 1
    fi

    # Create a PAT via the session API
    log_info "Creating root PAT via GitLab API..."
    local token_response
    token_response=$(curl -sf "${GITLAB_API}/personal_access_tokens" \
        -H "Content-Type: application/json" \
        --user "root:${root_password}" \
        -d '{
            "name": "cto-platform-setup",
            "scopes": ["api", "read_api", "read_repository", "write_repository", "admin_mode"],
            "expires_at": "'"$(date -v+1y +%Y-%m-%d 2>/dev/null || date -d '+1 year' +%Y-%m-%d)"'"
        }' 2>/dev/null || echo "")

    if [[ -z "$token_response" ]]; then
        log_error "Failed to create root PAT. GitLab may not be ready."
        exit 1
    fi

    echo "$token_response" | jq -r '.token'
}

# Wait for GitLab to be ready
wait_for_gitlab() {
    log_header "Waiting for GitLab"
    local max_attempts=60
    local attempt=0

    while [[ $attempt -lt $max_attempts ]]; do
        if curl -sf "${GITLAB_URL}/api/v4/version" > /dev/null 2>&1; then
            log_success "GitLab is ready at ${GITLAB_URL}"
            return
        fi
        attempt=$((attempt + 1))
        log_info "Attempt $attempt/$max_attempts — waiting 10s..."
        sleep 10
    done

    log_error "GitLab did not become ready in time"
    exit 1
}

# Create the 5dlabs group
create_group() {
    log_header "Creating group: ${GITLAB_GROUP}"

    local existing
    existing=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/groups?search=${GITLAB_GROUP}" | jq -r '.[0].id // empty')

    if [[ -n "$existing" ]]; then
        log_success "Group '${GITLAB_GROUP}' already exists (ID: ${existing})"
        echo "$existing"
        return
    fi

    local response
    response=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        -H "Content-Type: application/json" \
        "${GITLAB_API}/groups" \
        -d '{
            "name": "5D Labs",
            "path": "'"${GITLAB_GROUP}"'",
            "visibility": "private",
            "description": "5D Labs — AI Agent Platform"
        }')

    local group_id
    group_id=$(echo "$response" | jq -r '.id')
    log_success "Created group '${GITLAB_GROUP}' (ID: ${group_id})"
    echo "$group_id"
}

# Create OAuth application for CTO agent platform
create_oauth_app() {
    log_header "Creating OAuth Application"

    # Check if it already exists
    local existing
    existing=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/applications" | jq -r '.[] | select(.name == "CTO Agent Platform") | .id // empty')

    if [[ -n "$existing" ]]; then
        log_success "OAuth app 'CTO Agent Platform' already exists"
        return
    fi

    local response
    response=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        -H "Content-Type: application/json" \
        "${GITLAB_API}/applications" \
        -d '{
            "name": "CTO Agent Platform",
            "redirect_uri": "urn:ietf:wg:oauth:2.0:oob\nhttps://app.5dlabs.ai/oauth/callback",
            "scopes": "api read_api read_repository write_repository create_runner k8s_proxy manage_runner read_user",
            "trusted": true,
            "confidential": true
        }')

    local client_id client_secret
    client_id=$(echo "$response" | jq -r '.application_id')
    client_secret=$(echo "$response" | jq -r '.secret')

    log_success "Created OAuth app (client_id: ${client_id})"

    # Store in OpenBao
    log_info "Storing OAuth credentials in OpenBao..."
    kubectl exec -n openbao openbao-0 -- bao kv put secret/gitlab-oauth \
        "client-id=${client_id}" \
        "client-secret=${client_secret}" 2>/dev/null || log_warning "Failed to store in OpenBao"

    log_success "OAuth credentials stored in OpenBao at secret/gitlab-oauth"
}

# Import repositories from GitHub
import_repos() {
    log_header "Importing Repositories from GitHub"

    local github_token="${GITHUB_TOKEN:-}"
    if [[ -z "$github_token" ]]; then
        github_token=$(kubectl exec -n openbao openbao-0 -- bao kv get -field=GITHUB_PERSONAL_ACCESS_TOKEN secret/tools-github 2>/dev/null || echo "")
    fi

    if [[ -z "$github_token" ]]; then
        log_warning "No GITHUB_TOKEN available — skipping repo import"
        return
    fi

    local repos=("cto" "angel" "kotal" "openclaw-helm")
    local group_id="$1"

    for repo in "${repos[@]}"; do
        log_info "Importing ${GITHUB_ORG}/${repo}..."

        # Check if project already exists
        local existing
        existing=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
            "${GITLAB_API}/projects?search=${repo}" | jq -r ".[] | select(.path == \"${repo}\") | .id // empty")

        if [[ -n "$existing" ]]; then
            log_success "${repo} already exists (ID: ${existing})"
            continue
        fi

        local response
        response=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
            -H "Content-Type: application/json" \
            "${GITLAB_API}/import/github" \
            -d '{
                "personal_access_token": "'"${github_token}"'",
                "repo_id": 0,
                "new_name": "'"${repo}"'",
                "target_namespace": "'"${GITLAB_GROUP}"'",
                "github_hostname": "github.com",
                "optional_stages": {
                    "single_endpoint_issue_events_import": true,
                    "single_endpoint_notes_import": true,
                    "attachments_import": true,
                    "collaborators_import": false
                }
            }' 2>/dev/null || echo "")

        if [[ -n "$response" ]]; then
            local project_id
            project_id=$(echo "$response" | jq -r '.id // "pending"')
            log_success "Import started for ${repo} (project: ${project_id})"
        else
            # Try name-based import as fallback
            log_info "Trying alternative import method for ${repo}..."
            curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
                -H "Content-Type: application/json" \
                "${GITLAB_API}/projects" \
                -d '{
                    "name": "'"${repo}"'",
                    "namespace_id": '"${group_id}"',
                    "import_url": "https://'"${github_token}"'@github.com/'"${GITHUB_ORG}/${repo}"'.git",
                    "visibility": "private"
                }' > /dev/null 2>&1 && log_success "Import started for ${repo}" || log_warning "Failed to import ${repo}"
        fi
    done
}

# Register GitLab Runner and store token
setup_runner() {
    log_header "Registering GitLab Runner"

    local response
    response=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        -H "Content-Type: application/json" \
        "${GITLAB_API}/user/runners" \
        -d '{
            "runner_type": "instance_type",
            "description": "k8s-runner",
            "tag_list": "k8s,docker,linux",
            "run_untagged": true,
            "access_level": "not_protected"
        }' 2>/dev/null || echo "")

    if [[ -z "$response" ]]; then
        log_warning "Failed to register runner — may already exist"
        return
    fi

    local runner_token
    runner_token=$(echo "$response" | jq -r '.token')

    if [[ -n "$runner_token" && "$runner_token" != "null" ]]; then
        log_success "Runner registered"

        # Store in OpenBao
        kubectl exec -n openbao openbao-0 -- bao kv put secret/gitlab-runner \
            "registration-token=${runner_token}" 2>/dev/null || log_warning "Failed to store runner token in OpenBao"

        log_success "Runner token stored in OpenBao at secret/gitlab-runner"

        # Trigger ExternalSecrets refresh
        kubectl annotate externalsecret -n gitlab-runners --all "force-sync=$(date +%s)" --overwrite 2>/dev/null || true
    else
        log_warning "Runner token was empty"
    fi
}

# Create scheduled pipeline for daily agent builds
setup_scheduled_pipeline() {
    log_header "Creating Scheduled Pipeline"

    # Find the CTO project
    local project_id
    project_id=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/projects?search=cto" | jq -r ".[] | select(.path == \"cto\" and .namespace.path == \"${GITLAB_GROUP}\") | .id // empty")

    if [[ -z "$project_id" ]]; then
        log_warning "CTO project not found — skipping scheduled pipeline"
        return
    fi

    # Check if schedule already exists
    local existing
    existing=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/projects/${project_id}/pipeline_schedules" | jq -r '.[] | select(.description == "Daily Agent Build") | .id // empty')

    if [[ -n "$existing" ]]; then
        log_success "Scheduled pipeline already exists (ID: ${existing})"
        return
    fi

    local response
    response=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        -H "Content-Type: application/json" \
        "${GITLAB_API}/projects/${project_id}/pipeline_schedules" \
        -d '{
            "description": "Daily Agent Build",
            "ref": "main",
            "cron": "0 6 * * *",
            "cron_timezone": "UTC",
            "active": true
        }')

    log_success "Created scheduled pipeline: Daily Agent Build (6 AM UTC)"
}

# Set up CI/CD variables
setup_ci_variables() {
    log_header "Setting CI/CD Variables"

    local project_id
    project_id=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/projects?search=cto" | jq -r ".[] | select(.path == \"cto\" and .namespace.path == \"${GITLAB_GROUP}\") | .id // empty")

    if [[ -z "$project_id" ]]; then
        log_warning "CTO project not found — skipping CI variables"
        return
    fi

    # Get Cloudflare credentials from OpenBao
    local cf_token cf_account
    cf_token=$(kubectl exec -n openbao openbao-0 -- bao kv get -field=api-key secret/cloudflare 2>/dev/null || echo "")
    cf_account="${CLOUDFLARE_ACCOUNT_ID:-b73ec19faa187789b3f9d1deb0e0d95f}"

    if [[ -n "$cf_token" ]]; then
        # Set CLOUDFLARE_API_TOKEN
        curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
            "${GITLAB_API}/projects/${project_id}/variables" \
            --form "key=CLOUDFLARE_API_TOKEN" \
            --form "value=${cf_token}" \
            --form "protected=true" \
            --form "masked=true" > /dev/null 2>&1 || \
        curl -sf -X PUT -H "PRIVATE-TOKEN: ${TOKEN}" \
            "${GITLAB_API}/projects/${project_id}/variables/CLOUDFLARE_API_TOKEN" \
            --form "value=${cf_token}" \
            --form "protected=true" \
            --form "masked=true" > /dev/null 2>&1

        # Set CLOUDFLARE_ACCOUNT_ID
        curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
            "${GITLAB_API}/projects/${project_id}/variables" \
            --form "key=CLOUDFLARE_ACCOUNT_ID" \
            --form "value=${cf_account}" \
            --form "protected=true" > /dev/null 2>&1 || \
        curl -sf -X PUT -H "PRIVATE-TOKEN: ${TOKEN}" \
            "${GITLAB_API}/projects/${project_id}/variables/CLOUDFLARE_ACCOUNT_ID" \
            --form "value=${cf_account}" \
            --form "protected=true" > /dev/null 2>&1

        log_success "Set CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID"
    else
        log_warning "Cloudflare credentials not found in OpenBao"
    fi
}

register_pm_webhook() {
    log_header "Registering PM Webhook"

    local project_id
    project_id=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/projects?search=cto" | jq -r ".[] | select(.path == \"cto\" and .namespace.path == \"${GITLAB_GROUP}\") | .id // empty")

    if [[ -z "$project_id" ]]; then
        log_warning "CTO project not found — skipping webhook registration"
        return
    fi

    local webhook_url="${PM_WEBHOOK_URL:-https://cto.5dlabs.ai/webhooks/gitlab/events}"
    local webhook_secret
    webhook_secret=$(kubectl exec -n openbao openbao-0 -- bao kv get -field=webhook-secret secret/gitlab 2>/dev/null || echo "")

    # Check if webhook already exists
    local existing
    existing=$(curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        "${GITLAB_API}/projects/${project_id}/hooks" | jq -r ".[] | select(.url == \"${webhook_url}\") | .id // empty")

    if [[ -n "$existing" ]]; then
        log_success "PM webhook already registered (ID: ${existing})"
        return
    fi

    local token_param=""
    if [[ -n "$webhook_secret" ]]; then
        token_param=",\"token\":\"${webhook_secret}\""
    fi

    curl -sf -H "PRIVATE-TOKEN: ${TOKEN}" \
        -H "Content-Type: application/json" \
        "${GITLAB_API}/projects/${project_id}/hooks" \
        -d "{
            \"url\": \"${webhook_url}\",
            \"merge_requests_events\": true,
            \"pipeline_events\": true,
            \"note_events\": true,
            \"push_events\": false,
            \"enable_ssl_verification\": true${token_param}
        }" > /dev/null 2>&1 && log_success "PM webhook registered" || log_warning "Failed to register PM webhook"
}

# Store GitLab secrets in OpenBao
seed_gitlab_secrets() {
    log_header "Seeding GitLab Secrets in OpenBao"

    # Generate root password if not already set
    local existing_password
    existing_password=$(kubectl exec -n openbao openbao-0 -- bao kv get -field=root-password secret/gitlab 2>/dev/null || echo "")

    if [[ -z "$existing_password" ]]; then
        local root_password
        root_password=$(openssl rand -base64 32 | tr -d '=/+' | head -c 24)
        kubectl exec -n openbao openbao-0 -- bao kv put secret/gitlab \
            "root-password=${root_password}" 2>/dev/null || log_warning "Failed to store root password"
        log_success "Generated and stored GitLab root password in OpenBao"
    else
        log_success "GitLab root password already exists in OpenBao"
    fi

    # Create registry docker config
    local registry_user="root"
    local registry_password="${GITLAB_ROOT_PASSWORD:-$(kubectl exec -n openbao openbao-0 -- bao kv get -field=root-password secret/gitlab 2>/dev/null || echo '')}"

    if [[ -n "$registry_password" ]]; then
        local auth
        auth=$(echo -n "${registry_user}:${registry_password}" | base64)
        local dockerconfig="{\"auths\":{\"registry.5dlabs.ai\":{\"username\":\"${registry_user}\",\"password\":\"${registry_password}\",\"auth\":\"${auth}\"}}}"
        kubectl exec -n openbao openbao-0 -- bao kv put secret/gitlab-registry \
            ".dockerconfigjson=${dockerconfig}" 2>/dev/null || log_warning "Failed to store registry credentials"
        log_success "Stored GitLab registry credentials in OpenBao"
    fi
}

# ============================================================================
# Main
# ============================================================================

main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║           GitLab CE Post-Deploy Setup                        ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""

    wait_for_gitlab
    seed_gitlab_secrets

    TOKEN=$(get_root_token)
    if [[ -z "$TOKEN" ]]; then
        log_error "Failed to obtain GitLab API token"
        exit 1
    fi
    log_success "Authenticated with GitLab API"

    local group_id
    group_id=$(create_group)
    create_oauth_app
    import_repos "$group_id"
    setup_runner
    setup_scheduled_pipeline
    setup_ci_variables
    register_pm_webhook

    echo ""
    log_header "Setup Complete"
    echo ""
    log_success "GitLab CE is configured at ${GITLAB_URL}"
    log_info "Group: ${GITLAB_URL}/${GITLAB_GROUP}"
    log_info "Registry: https://registry.5dlabs.ai"
    log_info ""
    log_info "Refresh ExternalSecrets:"
    log_info "  kubectl annotate externalsecret -A --all force-sync=\"\$(date +%s)\" --overwrite"
    log_info ""
    log_info "Verify repo imports:"
    log_info "  curl -H 'PRIVATE-TOKEN: ${TOKEN:0:8}...' '${GITLAB_API}/projects?owned=true' | jq '.[].path_with_namespace'"
}

main "$@"
