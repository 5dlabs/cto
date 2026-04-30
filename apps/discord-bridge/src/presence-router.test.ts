import assert from "node:assert/strict";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createServer, type IncomingMessage, type Server } from "node:http";
import { AddressInfo } from "node:net";
import test from "node:test";
import type { DiscordHandle } from "./discord-client.js";
import { createPresenceRouter } from "./presence-router.js";
import type { PresenceInbound } from "./presence-types.js";

const logger = {
  info: () => undefined,
  warn: () => undefined,
  error: () => undefined,
};

function createDiscordStub(): DiscordHandle & { calls: Array<Record<string, string>> } {
  const calls: Array<Record<string, string>> = [];
  return {
    calls,
    initializeRooms: async () => [],
    renameChannel: async () => undefined,
    getOrCreateSessionThread: async () => "thread-created",
    postEmbed: async () => undefined,
    postElicitation: async () => ({ id: "elicitation-message" }) as never,
    updateMessage: async () => undefined,
    postMessage: async (channelId, content) => {
      calls.push({ op: "send", channelId, content });
      return "message-1";
    },
    editPlainMessage: async (channelId, messageId, content) => {
      calls.push({ op: "edit", channelId, messageId, content });
    },
    addReaction: async (channelId, messageId, emoji) => {
      calls.push({ op: "react", channelId, messageId, emoji });
    },
    sendTyping: async (channelId) => {
      calls.push({ op: "typing", channelId });
    },
    onInteraction: () => undefined,
    onMessage: () => undefined,
    destroy: () => undefined,
  };
}

async function readJson(req: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(typeof chunk === "string" ? Buffer.from(chunk) : chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

async function withWorker(
  handler: (req: IncomingMessage, body: unknown) => Promise<void> | void,
  run: (baseUrl: string) => Promise<void>,
): Promise<void> {
  const server: Server = createServer((req, res) => {
    void (async () => {
      const body = await readJson(req);
      await handler(req, body);
      res.writeHead(202, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ accepted: true }));
    })().catch((err) => {
      res.writeHead(500, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: String(err) }));
    });
  });
  await new Promise<void>((resolve) => server.listen(0, resolve));
  try {
    const { port } = server.address() as AddressInfo;
    await run(`http://127.0.0.1:${port}`);
  } finally {
    await new Promise<void>((resolve, reject) => server.close((err) => (err ? reject(err) : resolve())));
  }
}

function inbound(overrides: Partial<PresenceInbound> = {}): PresenceInbound {
  return {
    schema: "cto.presence.v1",
    event_type: "message",
    runtime: "hermes",
    agent_id: "rex",
    project_id: "project-1",
    task_id: "task-1",
    coderun_id: "coderun-1",
    discord: {
      account_id: "discord-bot",
      guild_id: "guild-1",
      channel_id: "channel-1",
      thread_id: "thread-1",
      message_id: "source-message",
      user_id: "user-1",
      chat_type: "thread",
    },
    text: "hello",
    ...overrides,
  };
}

test("routes the most specific Hermes route and signs bridge-to-worker delivery", async () => {
  const storePath = path.join(mkdtempSync(path.join(tmpdir(), "presence-router-")), "routes.json");
  const discord = createDiscordStub();
  const router = createPresenceRouter(discord, logger, storePath, undefined, "shared-token");
  let delivered: unknown;

  await withWorker(
    (req, body) => {
      assert.equal(req.headers.authorization, "Bearer shared-token");
      delivered = body;
    },
    async (workerUrl) => {
      await router.registerRoute({
        route_id: "generic-channel",
        runtime: "hermes",
        agent_id: "rex",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
      });
      await router.registerRoute({
        route_id: "specific-coderun",
        runtime: "hermes",
        agent_id: "rex",
        project_id: "project-1",
        task_id: "task-1",
        coderun_id: "coderun-1",
        worker_url: workerUrl,
        session_key: "session-1",
        discord: { account_id: "discord-bot", guild_id: "guild-1", channel_id: "channel-1", thread_id: "thread-1" },
      });

      const result = await router.routeInbound(inbound());
      assert.equal(result.route.route_id, "specific-coderun");
      assert.equal(result.workerStatus, 202);
      assert.equal((delivered as PresenceInbound).session_key, "session-1");

      const persisted = JSON.parse(readFileSync(storePath, "utf8"));
      assert.equal(persisted.length, 2);
    },
  );
});

test("rejects runtime and agent only routes to avoid cross-talk", async () => {
  const router = createPresenceRouter(createDiscordStub(), logger);
  await router.registerRoute({
    route_id: "too-broad",
    runtime: "hermes",
    agent_id: "rex",
    worker_url: "http://worker",
  });

  await assert.rejects(() => router.routeInbound(inbound()), /No presence route/);
});

test("rejects ambiguous equal-score route matches", async () => {
  const router = createPresenceRouter(createDiscordStub(), logger);
  for (const routeId of ["route-a", "route-b"]) {
    await router.registerRoute({
      route_id: routeId,
      runtime: "hermes",
      agent_id: "rex",
      project_id: "project-1",
      worker_url: "http://worker",
      discord: { account_id: "discord-bot", channel_id: "channel-1" },
    });
  }

  await assert.rejects(() => router.routeInbound(inbound({ coderun_id: undefined, task_id: undefined })), /Ambiguous presence route/);
});

test("fans one normalized Discord event out to matching Hermes OpenClaw and hosted routes", async () => {
  const router = createPresenceRouter(createDiscordStub(), logger, undefined, undefined, "shared-token");
  const delivered: unknown[] = [];

  await withWorker(
    (_req, body) => {
      delivered.push(body);
    },
    async (workerUrl) => {
      for (const route of [
        { route_id: "hermes-rex", runtime: "hermes", agent_id: "rex", coderun_id: "coderun-1" },
        { route_id: "openclaw-rex", runtime: "openclaw", agent_id: "rex", project_id: "project-1" },
        { route_id: "hosted-rex", runtime: "hosted", agent_id: "rex", task_id: "task-1" },
      ] as const) {
        await router.registerRoute({
          ...route,
          worker_url: workerUrl,
          discord: { account_id: "discord-bot", guild_id: "guild-1", channel_id: "channel-1", thread_id: "thread-1" },
        });
      }

      const result = await router.routeDiscordEvent({
        schema: "cto.presence.v1",
        event_type: "message",
        agent_id: "rex",
        project_id: "project-1",
        task_id: "task-1",
        coderun_id: "coderun-1",
        discord: {
          account_id: "discord-bot",
          guild_id: "guild-1",
          channel_id: "channel-1",
          thread_id: "thread-1",
          message_id: "source-message",
          mentioned_agent_ids: ["rex"],
        },
        text: "hello rex",
      });

      assert.equal(result.deliveries.length, 3);
      assert.deepEqual(
        delivered.map((item) => (item as PresenceInbound).runtime).sort(),
        ["hermes", "hosted", "openclaw"],
      );
      assert.deepEqual(
        delivered.map((item) => (item as PresenceInbound).agent_id),
        ["rex", "rex", "rex"],
      );
    },
  );
});

test("normalized mention messages route to non-default mentioned agents", async () => {
  const router = createPresenceRouter(createDiscordStub(), logger, undefined, undefined, "shared-token");
  const delivered: unknown[] = [];

  await withWorker(
    (_req, body) => {
      delivered.push(body);
    },
    async (workerUrl) => {
      await router.registerRoute({
        route_id: "rex",
        runtime: "hermes",
        agent_id: "rex",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
      });

      const result = await router.routeDiscordEvent({
        schema: "cto.presence.v1",
        event_type: "message",
        discord: {
          account_id: "discord-bot",
          channel_id: "channel-1",
          message_id: "source-message",
          mentioned_agent_ids: ["rex"],
        },
        text: "rex only",
      });

      assert.equal(result.deliveries.length, 1);
      assert.equal((delivered[0] as PresenceInbound).agent_id, "rex");
    },
  );
});

test("fanout ignores routes for unmentioned agents when the Discord event has mentions", async () => {
  const router = createPresenceRouter(createDiscordStub(), logger, undefined, undefined, "shared-token");
  const delivered: unknown[] = [];

  await withWorker(
    (_req, body) => {
      delivered.push(body);
    },
    async (workerUrl) => {
      await router.registerRoute({
        route_id: "rex",
        runtime: "hermes",
        agent_id: "rex",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
      });
      await router.registerRoute({
        route_id: "blaze",
        runtime: "hermes",
        agent_id: "blaze",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
      });

      const result = await router.routeDiscordEvent({
        schema: "cto.presence.v1",
        event_type: "message",
        discord: {
          account_id: "discord-bot",
          channel_id: "channel-1",
          message_id: "source-message",
          mentioned_agent_ids: ["rex"],
        },
        text: "rex only",
      });

      assert.equal(result.deliveries.length, 1);
      assert.equal((delivered[0] as PresenceInbound).agent_id, "rex");
    },
  );
});

test("unaddressed shared-channel Discord events do not fan out", async () => {
  const router = createPresenceRouter(createDiscordStub(), logger, undefined, undefined, "shared-token");
  const delivered: unknown[] = [];

  await withWorker(
    (_req, body) => {
      delivered.push(body);
    },
    async (workerUrl) => {
      await router.registerRoute({
        route_id: "rex",
        runtime: "hermes",
        agent_id: "rex",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
      });
      await router.registerRoute({
        route_id: "blaze",
        runtime: "hermes",
        agent_id: "blaze",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
      });

      const result = await router.routeDiscordEvent({
        schema: "cto.presence.v1",
        event_type: "message",
        discord: {
          account_id: "discord-bot",
          channel_id: "channel-1",
          message_id: "source-message",
        },
        text: "not addressed to a worker",
      });

      assert.equal(result.deliveries.length, 0);
      assert.equal(delivered.length, 0);
    },
  );
});

test("applies outbound Discord intents through the bridge-owned client", async () => {
  const discord = createDiscordStub();
  const router = createPresenceRouter(discord, logger);

  await assert.doesNotReject(() =>
    router.handleOutbound({
      schema: "cto.presence.v1",
      op: "send",
      target: { account_id: "discord-bot", channel_id: "channel-1", thread_id: "thread-1" },
      content: "reply",
    }),
  );
  await router.handleOutbound({
    schema: "cto.presence.v1",
    op: "edit",
    target: { account_id: "discord-bot", channel_id: "channel-1" },
    message_id: "message-1",
    content: "edited",
  });
  await router.handleOutbound({
    schema: "cto.presence.v1",
    op: "react",
    target: { account_id: "discord-bot", channel_id: "channel-1" },
    message_id: "message-1",
    emoji: "✅",
  });
  await router.handleOutbound({
    schema: "cto.presence.v1",
    op: "typing",
    target: { account_id: "discord-bot", channel_id: "channel-1" },
    active: true,
  });
  await router.handleOutbound({
    schema: "cto.presence.v1",
    op: "status",
    state: "done",
    target: { account_id: "discord-bot", channel_id: "channel-1" },
    message_id: "message-1",
  });

  assert.deepEqual(discord.calls, [
    { op: "send", channelId: "thread-1", content: "reply" },
    { op: "edit", channelId: "channel-1", messageId: "message-1", content: "edited" },
    { op: "react", channelId: "channel-1", messageId: "message-1", emoji: "✅" },
    { op: "typing", channelId: "channel-1" },
    { op: "react", channelId: "channel-1", messageId: "message-1", emoji: "✅" },
  ]);
});
