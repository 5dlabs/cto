#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
MANIFEST_PATH="${1:-$REPO_ROOT/infra/discord/cto-agent-channels.json}"

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    exit 1
  fi
}

require_tool curl
require_tool jq

if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "manifest not found: $MANIFEST_PATH" >&2
  exit 1
fi

: "${DISCORD_BOT_TOKEN:?set DISCORD_BOT_TOKEN to a bot token with Manage Channels}"

GUILD_ID="$(jq -r '.guild.id' "$MANIFEST_PATH")"
TEXT_CATEGORY_ID="$(jq -r '.categories.text.id' "$MANIFEST_PATH")"
CLEAR_OVERWRITES="$(jq -r '.patchRules.clearChannelPermissionOverwrites // false' "$MANIFEST_PATH")"
CREATE_MISSING="$(jq -r '.patchRules.createMissingChannels // true' "$MANIFEST_PATH")"
MOVE_CHANNELS="$(jq -r '.patchRules.moveChannelsIntoTextCategory // true' "$MANIFEST_PATH")"

API_BASE="https://discord.com/api/v10"
AUTH_HEADER="Authorization: Bot $DISCORD_BOT_TOKEN"
CONTENT_HEADER="Content-Type: application/json"
TMP_CHANNELS="$(mktemp)"
trap 'rm -f "$TMP_CHANNELS" /tmp/discord-sync-body.json' EXIT

refresh_channels() {
  curl -fsS \
    -H "$AUTH_HEADER" \
    "$API_BASE/guilds/$GUILD_ID/channels" >"$TMP_CHANNELS"
}

create_channel() {
  local name="$1"
  local payload
  payload="$(jq -n --arg name "$name" --arg parent_id "$TEXT_CATEGORY_ID" '{
    name: $name,
    type: 0,
    parent_id: $parent_id
  }')"
  curl -fsS \
    -X POST \
    -H "$AUTH_HEADER" \
    -H "$CONTENT_HEADER" \
    "$API_BASE/guilds/$GUILD_ID/channels" \
    -d "$payload" >/tmp/discord-sync-body.json
  jq -r '"CREATE \(.name) \(.id)"' /tmp/discord-sync-body.json
}

patch_channel() {
  local channel_id="$1"
  local name="$2"
  local payload
  payload="$(jq -n \
    --arg name "$name" \
    --arg parent_id "$TEXT_CATEGORY_ID" \
    --argjson move_channels "$MOVE_CHANNELS" \
    --argjson clear_overwrites "$CLEAR_OVERWRITES" '
      {
        name: $name
      }
      + (if $move_channels then {parent_id: $parent_id} else {} end)
      + (if $clear_overwrites then {permission_overwrites: []} else {} end)
    ')"
  curl -fsS \
    -X PATCH \
    -H "$AUTH_HEADER" \
    -H "$CONTENT_HEADER" \
    "$API_BASE/channels/$channel_id" \
    -d "$payload" >/tmp/discord-sync-body.json
  jq -r '"PATCH \(.name) \(.id)"' /tmp/discord-sync-body.json
}

refresh_channels

jq -c '.agents[]' "$MANIFEST_PATH" | while read -r agent; do
  slug="$(jq -r '.slug' <<<"$agent")"
  channel_id="$(jq -r '.channelId' <<<"$agent")"
  current="$(jq -c \
    --arg channel_id "$channel_id" \
    --arg slug "$slug" '
      map(select(.type == 0 and (.id == $channel_id or .name == $slug))) | first // empty
    ' "$TMP_CHANNELS")"

  if [[ -z "$current" || "$current" == "null" ]]; then
    if [[ "$CREATE_MISSING" != "true" ]]; then
      echo "MISS $slug"
      continue
    fi
    create_channel "$slug"
    refresh_channels
    continue
  fi

  current_id="$(jq -r '.id' <<<"$current")"
  current_name="$(jq -r '.name' <<<"$current")"
  current_parent_id="$(jq -r '.parent_id // ""' <<<"$current")"
  current_overwrites="$(jq -c '.permission_overwrites // []' <<<"$current")"

  if [[ "$current_name" != "$slug" ]]; then
    patch_channel "$current_id" "$slug"
    refresh_channels
    continue
  fi

  if [[ "$MOVE_CHANNELS" == "true" && "$current_parent_id" != "$TEXT_CATEGORY_ID" ]]; then
    patch_channel "$current_id" "$slug"
    refresh_channels
    continue
  fi

  if [[ "$CLEAR_OVERWRITES" == "true" && "$current_overwrites" != "[]" ]]; then
    patch_channel "$current_id" "$slug"
    refresh_channels
    continue
  fi

  echo "OK $slug $current_id"
done
