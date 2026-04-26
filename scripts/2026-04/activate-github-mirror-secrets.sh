#!/usr/bin/env bash
# One-shot: mint a GitLab group access token and wire it into the 5dlabs
# GitHub org as secret GITLAB_PUSH_TOKEN + variable MIRROR_TO_GITLAB=true.
#
# Re-running rotates the token.

set -euo pipefail

GL_HOST="gitlab.5dlabs.ai"
GL_GROUP_ID=3
GL_PAT="$(cat /tmp/gl_pat)"
GH_ORG="5dlabs"

# GitLab group access tokens max out at 1 year; pick ~364 days.
EXPIRES="$(date -v+364d +%Y-%m-%d 2>/dev/null || date -d '+364 days' +%Y-%m-%d)"

echo "Minting GitLab group access token (expires $EXPIRES)..."
resp="$(curl -sk -X POST -H "PRIVATE-TOKEN: $GL_PAT" \
  -H 'Content-Type: application/json' \
  -d "{\"name\":\"gh-mirror-sync\",\"scopes\":[\"api\",\"write_repository\"],\"access_level\":40,\"expires_at\":\"$EXPIRES\"}" \
  "https://${GL_HOST}/api/v4/groups/${GL_GROUP_ID}/access_tokens")"
token="$(printf '%s' "$resp" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("token",""))')"
if [[ -z "$token" ]]; then
  echo "Failed to mint token. Response:"; echo "$resp"; exit 1
fi
echo "Token minted (len=${#token})."

echo "Setting GitHub org secret GITLAB_PUSH_TOKEN..."
gh secret set GITLAB_PUSH_TOKEN --org "$GH_ORG" --visibility all --body "$token"

echo "Setting GitHub org variable MIRROR_TO_GITLAB=true..."
gh variable set MIRROR_TO_GITLAB --org "$GH_ORG" --visibility all --body "true"

echo "Done. Verify:"
echo "  gh secret list --org $GH_ORG"
echo "  gh variable list --org $GH_ORG"
