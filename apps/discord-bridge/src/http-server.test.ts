import assert from "node:assert/strict";
import type { Server } from "node:http";
import { AddressInfo } from "node:net";
import test from "node:test";
import type { Bridge } from "./bridge.js";
import type { DiscordHandle } from "./discord-client.js";
import { createHttpServer } from "./http-server.js";
import type { PresenceRouter } from "./presence-router.js";

const logger = {
  info: () => undefined,
  warn: () => undefined,
  error: () => undefined,
};

async function requestJson(
  port: number,
  path: string,
  body: unknown,
  token = "shared-token",
): Promise<{ status: number; body: Record<string, unknown> }> {
  const response = await fetch(`http://127.0.0.1:${port}${path}`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  return {
    status: response.status,
    body: (await response.json()) as Record<string, unknown>,
  };
}

function bridgeStub(): Bridge {
  return {
    handleMessage: () => undefined,
    stop: () => undefined,
  };
}

function discordStub(): DiscordHandle {
  return {
    initializeRooms: async () => [],
    renameChannel: async () => undefined,
    getOrCreateSessionThread: async () => "thread-created",
    postEmbed: async () => undefined,
    postElicitation: async () => ({ id: "elicitation-message" }) as never,
    updateMessage: async () => undefined,
    postMessage: async () => "message-1",
    editPlainMessage: async () => undefined,
    addReaction: async () => undefined,
    sendTyping: async () => undefined,
    onInteraction: () => undefined,
    onMessage: () => undefined,
    destroy: () => undefined,
  };
}

function presenceStub(): PresenceRouter & { routed?: unknown } {
  const route = { route_id: "route", runtime: "hermes" as const, agent_id: "rex", worker_url: "http://worker" };
  return {
    registerRoute: async () => route,
    deleteRoute: async () => true,
    listRoutes: () => [],
    routeInbound: async () => ({ accepted: true, route, workerStatus: 202 }),
    routeDiscordEvent: async function (payload) {
      if (!payload || typeof payload !== "object" || !("discord" in payload)) {
        throw new Error("discord must be an object");
      }
      this.routed = payload;
      return { accepted: true, deliveries: [{ route, workerStatus: 202 }] };
    },
    handleOutbound: async () => ({ accepted: true }),
  };
}

async function withHttp<T>(presence: PresenceRouter, run: (port: number) => Promise<T>): Promise<T> {
  const http = createHttpServer(0, bridgeStub(), undefined, logger, undefined, discordStub(), presence, "shared-token");
  await http.start();
  try {
    const server = (http as unknown as { server?: Server }).server;
    assert.ok(server, "test server should expose its bound address");
    return await run((server.address() as AddressInfo).port);
  } finally {
    await http.stop();
  }
}

test("/presence/discord-events fans a synthetic normalized event through the router", async () => {
  const presence = presenceStub();

  await withHttp(presence, async (port) => {
    const payload = {
      schema: "cto.presence.v1",
      event_type: "message",
      discord: { account_id: "discord-bridge", channel_id: "channel-1", message_id: "message-1" },
      text: "hello rex",
    };

    const response = await requestJson(port, "/presence/discord-events", payload);

    assert.equal(response.status, 202);
    assert.equal((response.body.deliveries as unknown[]).length, 1);
    assert.deepEqual(presence.routed, payload);
  });
});

test("/presence/discord-events requires the shared presence token", async () => {
  const presence = presenceStub();

  await withHttp(presence, async (port) => {
    const response = await requestJson(
      port,
      "/presence/discord-events",
      { schema: "cto.presence.v1", event_type: "message", discord: { account_id: "discord-bridge", channel_id: "channel-1" } },
      "wrong-token",
    );

    assert.equal(response.status, 401);
    assert.equal(presence.routed, undefined);
  });
});

test("/presence/discord-events reports invalid normalized event payloads", async () => {
  const presence = presenceStub();

  await withHttp(presence, async (port) => {
    const response = await requestJson(port, "/presence/discord-events", { schema: "cto.presence.v1", event_type: "message" });

    assert.equal(response.status, 400);
    assert.match(String(response.body.error), /discord must be an object/);
    assert.equal(presence.routed, undefined);
  });
});
