import assert from "node:assert/strict";
import test from "node:test";
import { validatePresenceDiscordEvent, validatePresenceInbound, validatePresenceRoute } from "./presence-types.js";

test("rejects non-string metadata values on route registration", () => {
  const result = validatePresenceRoute({
    runtime: "hermes",
    agent_id: "rex",
    worker_url: "http://worker",
    metadata: { route_kind: "home", home_id: 42 },
  });

  assert.equal(result.ok, false);
  if (!result.ok) {
    assert.match(result.error, /metadata must be a string map/);
  }
});

test("rejects non-string metadata values on normalized Discord events", () => {
  const result = validatePresenceDiscordEvent({
    schema: "cto.presence.v1",
    event_type: "message",
    discord: { account_id: "discord-bot", channel_id: "channel-1" },
    metadata: { route_kind: "home", home_id: 42 },
  });

  assert.equal(result.ok, false);
  if (!result.ok) {
    assert.match(result.error, /metadata must be a string map/);
  }
});

test("rejects non-string metadata values on worker inbound events", () => {
  const result = validatePresenceInbound({
    schema: "cto.presence.v1",
    event_type: "message",
    runtime: "hermes",
    agent_id: "rex",
    discord: { account_id: "discord-bot", channel_id: "channel-1" },
    metadata: { route_kind: "home", home_id: 42 },
  });

  assert.equal(result.ok, false);
  if (!result.ok) {
    assert.match(result.error, /metadata must be a string map/);
  }
});

test("accepts empty text on attachment-only normalized Discord events", () => {
  const result = validatePresenceDiscordEvent({
    schema: "cto.presence.v1",
    event_type: "message",
    discord: { account_id: "discord-bot", channel_id: "channel-1" },
    text: "",
    attachments: [{ url: "https://cdn.example/file.png" }],
  });

  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.value.text, "");
  }
});

test("rejects malformed attachments on normalized Discord events", () => {
  const result = validatePresenceDiscordEvent({
    schema: "cto.presence.v1",
    event_type: "message",
    discord: { account_id: "discord-bot", channel_id: "channel-1" },
    attachments: [{ url: "https://cdn.example/file.png", size: -1 }],
  });

  assert.equal(result.ok, false);
  if (!result.ok) {
    assert.match(result.error, /attachments must be an array of valid attachment objects/);
  }
});

test("accepts rich attachment metadata on worker inbound events", () => {
  const result = validatePresenceInbound({
    schema: "cto.presence.v1",
    event_type: "message",
    runtime: "hermes",
    agent_id: "rex",
    discord: { account_id: "discord-bot", channel_id: "channel-1" },
    attachments: [
      {
        id: "att-1",
        url: "https://cdn.example/file.png",
        content_type: "image/png",
        filename: "file.png",
        size: 12345,
        spoiler: true,
      },
    ],
  });

  assert.equal(result.ok, true);
  if (result.ok) {
    assert.deepEqual(result.value.attachments, [
      {
        id: "att-1",
        url: "https://cdn.example/file.png",
        content_type: "image/png",
        filename: "file.png",
        size: 12345,
        spoiler: true,
      },
    ]);
  }
});
