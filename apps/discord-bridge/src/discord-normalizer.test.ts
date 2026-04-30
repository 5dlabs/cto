import assert from "node:assert/strict";
import test from "node:test";
import { normalizeDiscordMessage } from "./discord-normalizer.js";

function message(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    id: "message-1",
    content: "<@bot> rex please investigate",
    author: { id: "user-1", username: "edge", bot: false },
    guildId: "guild-1",
    channelId: "channel-1",
    channel: { id: "channel-1", type: "GuildText" },
    mentions: {
      users: new Map([
        ["bot", { id: "bot", username: "Coder", bot: true }],
        ["rex-snowflake", { id: "123456789012345678", username: "Rex", bot: true }],
      ]),
    },
    attachments: new Map([
      ["att-1", { url: "https://cdn.example/file.png", contentType: "image/png", name: "file.png" }],
    ]),
    ...overrides,
  };
}

test("normalizes guild messages into cto.presence.v1 without Discord credentials", () => {
  const event = normalizeDiscordMessage(message(), { accountId: "coder-control", defaultAgentId: "coder", botUserId: "bot" });

  assert.equal(event?.schema, "cto.presence.v1");
  assert.equal(event?.event_type, "message");
  assert.equal(event?.agent_id, undefined);
  assert.equal(event?.discord.account_id, "coder-control");
  assert.equal(event?.discord.guild_id, "guild-1");
  assert.equal(event?.discord.channel_id, "channel-1");
  assert.equal(event?.discord.message_id, "message-1");
  assert.equal(event?.discord.user_id, "user-1");
  assert.equal(event?.discord.user_name, "edge");
  assert.equal(event?.discord.chat_type, "group");
  assert.equal(event?.text, "<@bot> rex please investigate");
  assert.deepEqual(event?.discord.mentioned_agent_ids, ["123456789012345678", "rex"]);
  assert.deepEqual(event?.attachments, [{ url: "https://cdn.example/file.png", content_type: "image/png", filename: "file.png" }]);
  assert.equal(JSON.stringify(event).includes("DISCORD_BRIDGE_TOKEN"), false);
});

test("normalizes thread messages with parent channel id", () => {
  const event = normalizeDiscordMessage(
    message({ channelId: "thread-1", channel: { id: "thread-1", isThread: () => true, parentId: "channel-1" } }),
    { accountId: "coder-control", defaultAgentId: "coder" },
  );

  assert.equal(event?.discord.channel_id, "channel-1");
  assert.equal(event?.discord.thread_id, "thread-1");
  assert.equal(event?.discord.parent_channel_id, "channel-1");
  assert.equal(event?.discord.chat_type, "thread");
});

test("ignores bot-authored messages", () => {
  const event = normalizeDiscordMessage(message({ author: { id: "bot-author", username: "bot", bot: true } }), {
    accountId: "coder-control",
    defaultAgentId: "coder",
  });

  assert.equal(event, undefined);
});
