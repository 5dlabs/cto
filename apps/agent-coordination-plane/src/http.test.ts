import assert from "node:assert/strict";
import test from "node:test";
import type { AddressInfo } from "node:net";
import { createCoordinationHttpServer, type AgentIdentity, type AgentMessage } from "./index.js";

const token = "unit-secret";
const now = "2026-04-01T00:00:00.000Z";
const later = "2026-04-01T00:00:02.000Z";

const planner: AgentIdentity = {
  agentId: "planner-http",
  role: "planner",
  runtime: "hermes",
  projectId: "Alpha",
  taskId: "T-1",
};

const coder: AgentIdentity = {
  agentId: "coder-http",
  role: "coder",
  runtime: "openclaw",
  projectId: "Alpha",
  taskId: "T-1",
};

async function withServer<T>(fn: (baseUrl: string) => Promise<T>): Promise<T> {
  const server = createCoordinationHttpServer({ sharedToken: token, now: () => now });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address() as AddressInfo;
  try {
    return await fn(`http://127.0.0.1:${address.port}`);
  } finally {
    await new Promise<void>((resolve, reject) => server.close((error) => (error ? reject(error) : resolve())));
  }
}

async function request(baseUrl: string, path: string, init: RequestInit = {}) {
  const headers = new Headers(init.headers);
  headers.set("authorization", `Bearer ${token}`);
  if (init.body !== undefined && !headers.has("content-type")) {
    headers.set("content-type", "application/json");
  }
  return fetch(`${baseUrl}${path}`, { ...init, headers });
}

async function json(response: Response): Promise<Record<string, unknown>> {
  return (await response.json()) as Record<string, unknown>;
}

function message(to: AgentMessage["to"], messageId = "http-msg-1"): AgentMessage<{ text: string }> {
  return {
    messageId,
    kind: "request",
    from: planner,
    to,
    body: { text: "coordinate over HTTP" },
    createdAt: now,
    priority: "normal",
    metadata: { source: "http-contract-test" },
  };
}

test("coordination HTTP service gates mutating APIs with bearer auth", async () => {
  await withServer(async (baseUrl) => {
    const health = await fetch(`${baseUrl}/healthz`);
    assert.equal(health.status, 200);
    assert.deepEqual(await json(health), { ok: true });

    const missing = await fetch(`${baseUrl}/v1/lookup`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ address: { kind: "broadcast" } }),
    });
    assert.equal(missing.status, 401);
    assert.deepEqual(await json(missing), {
      error: { code: "unauthorized", message: "missing or invalid bearer token" },
    });
  });
});

test("coordination HTTP service registers agents, looks up routes, delivers messages, and acks inbox entries", async () => {
  await withServer(async (baseUrl) => {
    const plannerRegistration = await request(baseUrl, "/v1/agents", {
      method: "POST",
      body: JSON.stringify({ identity: planner, ttlMs: 10_000, metadata: { registeredBy: "test" } }),
    });
    assert.equal(plannerRegistration.status, 201);
    const plannerBody = await json(plannerRegistration);
    assert.equal((plannerBody.identity as AgentIdentity).agentId, "planner-http");

    const coderRegistration = await request(baseUrl, "/v1/agents", {
      method: "POST",
      body: JSON.stringify({ identity: coder, ttlMs: 10_000 }),
    });
    assert.equal(coderRegistration.status, 201);

    const lookup = await request(baseUrl, "/v1/lookup", {
      method: "POST",
      body: JSON.stringify({ address: { kind: "task", projectId: "Alpha", taskId: "T-1" } }),
    });
    assert.equal(lookup.status, 200);
    const lookupBody = await json(lookup);
    assert.deepEqual(
      ((lookupBody.agents as AgentIdentity[]) ?? []).map((agent) => agent.agentId),
      ["planner-http", "coder-http"],
    );

    const send = await request(baseUrl, "/v1/messages", {
      method: "POST",
      body: JSON.stringify({ message: message({ kind: "role", role: "coder", projectId: "Alpha" }), envelopeId: "env-http-1", now }),
    });
    assert.equal(send.status, 202);
    const sendBody = await json(send);
    assert.deepEqual(
      ((sendBody.recipients as AgentIdentity[]) ?? []).map((agent) => agent.agentId),
      ["coder-http"],
    );

    const inbox = await request(baseUrl, "/v1/inbox/coder-http");
    assert.equal(inbox.status, 200);
    const inboxBody = await json(inbox);
    const messages = inboxBody.messages as Array<{ deliveryId: string; envelope: { envelopeId: string; message: { messageId: string } } }>;
    assert.equal(messages.length, 1);
    assert.equal(messages[0].envelope.envelopeId, "env-http-1");

    const ack = await request(baseUrl, `/v1/inbox/coder-http/acks/${encodeURIComponent(messages[0].deliveryId)}`, {
      method: "POST",
      body: JSON.stringify({}),
    });
    assert.equal(ack.status, 200);
    assert.deepEqual(await json(ack), { acked: true });

    const emptyInbox = await request(baseUrl, "/v1/inbox/coder-http");
    assert.deepEqual((await json(emptyInbox)).messages, []);
  });
});

test("coordination HTTP service supports groups and deterministic dead letters", async () => {
  await withServer(async (baseUrl) => {
    for (const identity of [planner, coder]) {
      const registered = await request(baseUrl, "/v1/agents", {
        method: "POST",
        body: JSON.stringify({ identity, ttlMs: 10_000 }),
      });
      assert.equal(registered.status, 201);
    }

    const group = await request(baseUrl, "/v1/groups", {
      method: "POST",
      body: JSON.stringify({ group: { groupId: "alpha-pair", name: "Alpha Pair", members: [planner, coder] } }),
    });
    assert.equal(group.status, 201);

    const sent = await request(baseUrl, "/v1/messages", {
      method: "POST",
      body: JSON.stringify({ message: message({ kind: "group", groupId: "alpha-pair" }, "group-msg"), envelopeId: "env-group", now }),
    });
    assert.equal(sent.status, 202);
    assert.deepEqual(
      (((await json(sent)).recipients as AgentIdentity[]) ?? []).map((agent) => agent.agentId),
      ["planner-http", "coder-http"],
    );

    for (const marker of ["one", "two", "three"]) {
      const failed = await request(baseUrl, `/v1/inbox/coder-http/failures/${encodeURIComponent("env-group")}`, {
        method: "POST",
        body: JSON.stringify({ now: later, reason: `failed-${marker}` }),
      });
      assert.equal(failed.status, 200);
    }

    const deadLetters = await request(baseUrl, "/v1/dead-letters/coder-http");
    assert.equal(deadLetters.status, 200);
    const body = await json(deadLetters);
    const messages = body.messages as Array<{ envelope: { envelopeId: string }; reason: string }>;
    assert.deepEqual(
      messages.map((entry) => ({ envelopeId: entry.envelope.envelopeId, reason: entry.reason })),
      [{ envelopeId: "env-group", reason: "failed-three" }],
    );
  });
});
