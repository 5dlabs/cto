import assert from "node:assert/strict";
import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import { AddressInfo } from "node:net";
import test from "node:test";
import type { Bridge } from "./bridge.js";
import type { DiscordHandle } from "./discord-client.js";
import { createHttpServer } from "./http-server.js";
import { createPresenceRouter } from "./presence-router.js";
import type { PresenceInbound } from "./presence-types.js";

const PRESENCE_TOKEN = "shared-presence-token";

const logger = {
  info: () => undefined,
  warn: () => undefined,
  error: () => undefined,
};

function bridgeStub(): Bridge {
  return {
    handleMessage: () => undefined,
    stop: () => undefined,
  };
}

function discordStub(): DiscordHandle & { calls: Array<Record<string, string>> } {
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

function writeJson(res: ServerResponse, status: number, body: unknown): void {
  res.writeHead(status, { "Content-Type": "application/json" });
  res.end(JSON.stringify(body));
}

async function listen(server: Server): Promise<string> {
  await new Promise<void>((resolve) => server.listen(0, resolve));
  const { port } = server.address() as AddressInfo;
  return `http://127.0.0.1:${port}`;
}

async function close(server: Server): Promise<void> {
  await new Promise<void>((resolve, reject) => server.close((err) => (err ? reject(err) : resolve())));
}

test("hosted generic worker registers, receives authenticated normalized inbound, and requests outbound via bridge", async () => {
  const discord = discordStub();
  const router = createPresenceRouter(discord, logger, undefined, undefined, PRESENCE_TOKEN);
  const bridgeHttp = createHttpServer(0, bridgeStub(), undefined, logger, undefined, discord, router, PRESENCE_TOKEN);
  const inboundDeliveries: PresenceInbound[] = [];
  let bridgeUrl = "";

  const hostedWorker = createServer((req, res) => {
    void (async () => {
      if (req.method !== "POST" || req.url !== "/presence/inbound") {
        writeJson(res, 404, { error: "Not found" });
        return;
      }
      assert.equal(req.headers.authorization, `Bearer ${PRESENCE_TOKEN}`);
      const inbound = (await readJson(req)) as PresenceInbound;
      assert.equal(inbound.schema, "cto.presence.v1");
      assert.equal(inbound.runtime, "hosted");
      assert.equal(inbound.agent_id, "hosted-generic-worker");
      assert.equal(inbound.discord.channel_id, "channel-1");
      assert.equal(inbound.metadata?.route_id, "hosted-generic-worker-route");
      inboundDeliveries.push(inbound);

      const outboundHeaders = {
        "Content-Type": "application/json",
        Authorization: `Bearer ${PRESENCE_TOKEN}`,
      };
      const statusResponse = await fetch(`${bridgeUrl}/presence/outbound`, {
        method: "POST",
        headers: outboundHeaders,
        body: JSON.stringify({
          op: "status",
          state: "running",
          detail: "hosted worker accepted normalized inbound",
          target: {
            account_id: inbound.discord.account_id,
            channel_id: inbound.discord.channel_id,
            thread_id: inbound.discord.thread_id,
          },
          message_id: inbound.discord.message_id,
        }),
      });
      assert.equal(statusResponse.status, 200);

      const sendResponse = await fetch(`${bridgeUrl}/presence/outbound`, {
        method: "POST",
        headers: outboundHeaders,
        body: JSON.stringify({
          op: "send",
          target: {
            account_id: inbound.discord.account_id,
            channel_id: inbound.discord.channel_id,
            thread_id: inbound.discord.thread_id,
          },
          content: `hosted echo: ${inbound.text}`,
        }),
      });
      assert.equal(sendResponse.status, 200);

      writeJson(res, 202, { accepted: true });
    })().catch((err) => {
      writeJson(res, 500, { error: String(err instanceof Error ? err.message : err) });
    });
  });

  const workerUrl = await listen(hostedWorker);
  await bridgeHttp.start();
  try {
    const bridgeServer = (bridgeHttp as unknown as { server?: Server }).server;
    assert.ok(bridgeServer, "bridge server should expose its bound address");
    bridgeUrl = `http://127.0.0.1:${(bridgeServer.address() as AddressInfo).port}`;

    const registerResponse = await fetch(`${bridgeUrl}/presence/routes`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${PRESENCE_TOKEN}` },
      body: JSON.stringify({
        route_id: "hosted-generic-worker-route",
        runtime: "hosted",
        agent_id: "hosted-generic-worker",
        worker_url: workerUrl,
        discord: { account_id: "discord-bot", channel_id: "channel-1" },
        metadata: { worker_kind: "generic", route_kind: "hosted-example" },
      }),
    });
    assert.equal(registerResponse.status, 200);
    const registered = (await registerResponse.json()) as { route: { runtime: string; route_id: string } };
    assert.equal(registered.route.runtime, "hosted");
    assert.equal(registered.route.route_id, "hosted-generic-worker-route");

    const inboundResponse = await fetch(`${bridgeUrl}/presence/discord-events`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${PRESENCE_TOKEN}` },
      body: JSON.stringify({
        schema: "cto.presence.v1",
        event_type: "message",
        agent_id: "hosted-generic-worker",
        discord: {
          account_id: "discord-bot",
          channel_id: "channel-1",
          thread_id: "thread-1",
          message_id: "source-message-1",
          user_id: "user-1",
          chat_type: "thread",
        },
        text: "hello hosted worker",
      }),
    });
    assert.equal(inboundResponse.status, 202);
    const inboundResult = (await inboundResponse.json()) as {
      deliveries: Array<{ route: { route_id: string; runtime: string; agent_id: string }; workerStatus: number }>;
    };
    assert.equal(inboundResult.deliveries.length, 1);
    assert.equal(inboundResult.deliveries[0].route.route_id, "hosted-generic-worker-route");
    assert.equal(inboundResult.deliveries[0].route.runtime, "hosted");
    assert.equal(inboundResult.deliveries[0].route.agent_id, "hosted-generic-worker");
    assert.equal(inboundResult.deliveries[0].workerStatus, 202);
    assert.equal(inboundDeliveries.length, 1);
    assert.equal(inboundDeliveries[0].session_key, "discord:discord-bot:dm:user-1:thread-1");
    assert.deepEqual(discord.calls, [
      { op: "react", channelId: "thread-1", messageId: "source-message-1", emoji: "👀" },
      { op: "send", channelId: "thread-1", content: "hosted echo: hello hosted worker" },
    ]);
  } finally {
    await bridgeHttp.stop();
    await close(hostedWorker);
  }
});
