import assert from "node:assert/strict";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createServer, type IncomingMessage, type Server } from "node:http";
import { AddressInfo } from "node:net";
import test from "node:test";
import { createAdapterServer } from "./index.js";
import type { AdapterConfig, PresenceInbound } from "./types.js";

async function readJson(req: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(typeof chunk === "string" ? Buffer.from(chunk) : chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

async function listen(server: Server): Promise<string> {
  await new Promise<void>((resolve) => server.listen(0, resolve));
  const { port } = server.address() as AddressInfo;
  return `http://127.0.0.1:${port}`;
}

async function close(server: Server): Promise<void> {
  await new Promise<void>((resolve, reject) => server.close((err) => (err ? reject(err) : resolve())));
}

function event(): PresenceInbound {
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
      message_id: "message-1",
      user_id: "user-1",
      chat_type: "thread",
    },
    text: "run this",
  };
}

function config(overrides: Partial<AdapterConfig> = {}): AdapterConfig {
  const dir = mkdtempSync(path.join(tmpdir(), "hermes-presence-adapter-"));
  return {
    port: 0,
    inboxPath: path.join(dir, "inbox.jsonl"),
    presenceSharedToken: "shared-token",
    ...overrides,
  };
}

test("requires the shared token for inbound worker events", async () => {
  const server = createAdapterServer(config());
  const baseUrl = await listen(server);
  try {
    const response = await fetch(`${baseUrl}/presence/inbound`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(event()),
    });
    assert.equal(response.status, 401);
  } finally {
    await close(server);
  }
});

test("accepts authenticated inbound events and falls back to the inbox when Hermes input is down", async () => {
  const cfg = config({ hermesInputUrl: "http://127.0.0.1:9/input" });
  const server = createAdapterServer(cfg);
  const baseUrl = await listen(server);
  try {
    const response = await fetch(`${baseUrl}/presence/inbound`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer shared-token" },
      body: JSON.stringify(event()),
    });
    assert.equal(response.status, 202);
    const body = (await response.json()) as { accepted: boolean; run: { id: string; status: string } };
    assert.deepEqual(body, { accepted: true, run: { id: "coderun-1", status: "queued" } });
    const inboxLine = readFileSync(cfg.inboxPath, "utf8").trim();
    const queued = JSON.parse(inboxLine) as { event: PresenceInbound };
    assert.equal(queued.event.text, "run this");
  } finally {
    await close(server);
  }
});

test("rejects inbound metadata values that are not strings", async () => {
  const server = createAdapterServer(config());
  const baseUrl = await listen(server);
  try {
    const inbound = event();
    Object.assign(inbound, { metadata: { route_id: "route-1", home_id: 42 } });

    const response = await fetch(`${baseUrl}/presence/inbound`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer shared-token" },
      body: JSON.stringify(inbound),
    });

    assert.equal(response.status, 400);
    assert.match(await response.text(), /metadata must be a string map/);
  } finally {
    await close(server);
  }
});

test("posts non-fatal presence status intents and sends Hermes input payloads", async () => {
  const statuses: unknown[] = [];
  const hermesRequests: unknown[] = [];
  const presenceServer = createServer((req, res) => {
    void (async () => {
      statuses.push(await readJson(req));
      res.writeHead(statuses.length === 1 ? 503 : 202, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: true }));
    })();
  });
  const hermesServer = createServer((req, res) => {
    void (async () => {
      hermesRequests.push(await readJson(req));
      res.writeHead(202, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ id: "run-1", status: "accepted" }));
    })();
  });
  const presenceUrl = await listen(presenceServer);
  const hermesUrl = await listen(hermesServer);
  const adapterServer = createAdapterServer(config({ presenceRouterUrl: presenceUrl, hermesInputUrl: `${hermesUrl}/input` }));
  const adapterUrl = await listen(adapterServer);
  try {
    const response = await fetch(`${adapterUrl}/presence/inbound`, {
      method: "POST",
      headers: { "Content-Type": "application/json", "x-presence-token": "shared-token" },
      body: JSON.stringify(event()),
    });
    assert.equal(response.status, 202);
    assert.equal(statuses.length, 2);
    assert.equal((statuses[1] as { state: string }).state, "running");
    assert.deepEqual(hermesRequests, [
      {
        input: "run this",
        metadata: {
          schema: "cto.presence.v1",
          runtime: "hermes",
          agent_id: "rex",
          project_id: "project-1",
          task_id: "task-1",
          coderun_id: "coderun-1",
          discord_account_id: "discord-bot",
          discord_channel_id: "channel-1",
          discord_thread_id: "thread-1",
          discord_message_id: "message-1",
          session_key: "",
        },
        session: {
          platform: "discord",
          chat_id: "thread-1",
          chat_type: "thread",
          user_id: "user-1",
          thread_id: "thread-1",
        },
      },
    ]);
  } finally {
    await close(adapterServer);
    await close(hermesServer);
    await close(presenceServer);
  }
});

test("forwards deterministic session and home route metadata to Hermes input", async () => {
  const hermesRequests: unknown[] = [];
  const hermesServer = createServer((req, res) => {
    void (async () => {
      hermesRequests.push(await readJson(req));
      res.writeHead(202, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ id: "run-1", status: "accepted" }));
    })();
  });
  const hermesUrl = await listen(hermesServer);
  const adapterServer = createAdapterServer(config({ hermesInputUrl: `${hermesUrl}/input` }));
  const adapterUrl = await listen(adapterServer);
  try {
    const inbound = event();
    inbound.session_key = "discord:discord-bot:guild:guild-1:thread-1";
    inbound.metadata = { route_id: "thread-home", home_id: "home-thread-1", home_route_id: "thread-home" };

    const response = await fetch(`${adapterUrl}/presence/inbound`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer shared-token" },
      body: JSON.stringify(inbound),
    });

    assert.equal(response.status, 202);
    assert.equal(hermesRequests.length, 1);
    assert.equal(
      (hermesRequests[0] as { metadata: Record<string, string> }).metadata.session_key,
      "discord:discord-bot:guild:guild-1:thread-1",
    );
    assert.deepEqual((hermesRequests[0] as { session: Record<string, string> }).session, {
      platform: "discord",
      chat_id: "thread-1",
      chat_type: "thread",
      user_id: "user-1",
      thread_id: "thread-1",
      home_id: "home-thread-1",
      home_route_id: "thread-home",
      route_id: "thread-home",
    });
  } finally {
    await close(adapterServer);
    await close(hermesServer);
  }
});
