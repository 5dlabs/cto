#!/usr/bin/env bash
#
# Research Authentication Helper
#
# This script helps set up Twitter authentication for the research pipeline.
# It can either:
# 1. Run interactive browser auth and export to Vault
# 2. Guide manual cookie extraction
#
# Usage:
#   ./scripts/research-auth.sh [--vault|--manual]
#
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

usage() {
    cat <<EOF
Research Authentication Helper

Usage: $0 [OPTIONS]

Options:
  --vault     Run browser auth and export directly to Vault
  --manual    Show manual cookie extraction instructions
  --file      Run browser auth and save to local file
  -h, --help  Show this help message

Examples:
  $0 --vault     # Interactive auth, export to Vault
  $0 --manual    # Manual cookie extraction guide
  $0 --file      # Save cookies to .twitter-session.json
EOF
}

manual_instructions() {
    cat <<EOF
=============================================================================
Manual Cookie Extraction Instructions
=============================================================================

1. Open Twitter/X in your browser and log in

2. Open Developer Tools (F12 or Cmd+Option+I)

3. Go to Application > Storage > Cookies > https://x.com

4. Find these cookies:
   - auth_token: Long-lived authentication token
   - ct0: CSRF token (optional, regenerates automatically)

5. Copy the values and run:

   vault kv put secret/research-twitter \\
     TWITTER_AUTH_TOKEN=<auth_token-value> \\
     TWITTER_CT0=<ct0-value>

Note: The auth_token cookie is typically valid for 6+ months.
      The ct0 cookie regenerates automatically, so it's optional.

=============================================================================
EOF
}

run_browser_auth() {
    local export_to_vault="${1:-false}"
    local output_file="${2:-.twitter-session.json}"

    info "Starting browser-based authentication..."
    info "A browser window will open. Please log in to Twitter/X."

    if [[ "$export_to_vault" == "true" ]]; then
        cargo run -p research -- auth --export-to-vault
    else
        cargo run -p research -- auth --output="$output_file"
        info "Session saved to: $output_file"
        info ""
        info "To upload to Vault, run:"
        info "  vault kv put secret/research-twitter TWITTER_AUTH_TOKEN=\$(jq -r .auth_token $output_file)"
    fi
}

main() {
    local mode=""

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --vault)
                mode="vault"
                shift
                ;;
            --manual)
                mode="manual"
                shift
                ;;
            --file)
                mode="file"
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    if [[ -z "$mode" ]]; then
        echo ""
        echo "Research Authentication Helper"
        echo "=============================="
        echo ""
        echo "Choose an option:"
        echo "  1) Browser auth → export to Vault"
        echo "  2) Browser auth → save to local file"
        echo "  3) Manual cookie extraction instructions"
        echo ""
        read -rp "Enter choice [1-3]: " choice

        case "$choice" in
            1) mode="vault" ;;
            2) mode="file" ;;
            3) mode="manual" ;;
            *)
                error "Invalid choice"
                exit 1
                ;;
        esac
    fi

    case "$mode" in
        vault)
            run_browser_auth true
            ;;
        file)
            run_browser_auth false
            ;;
        manual)
            manual_instructions
            ;;
    esac
}

main "$@"

