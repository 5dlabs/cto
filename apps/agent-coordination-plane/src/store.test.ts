import assert from "node:assert/strict";
import test from "node:test";
import { InMemoryCoordinationPlane, createEnvelope, type AgentIdentity, type AgentMessage } from "./index.js";

const now = "2026-04-01T00:00:00.000Z";
const later = "2026-04-01T00:00:02.000Z";

const planner: AgentIdentity = {
  agentId: "planner-1",
  role: "planner",
  runtime: "hermes",
  projectId: "Project Alpha",
  taskId: "Task/42",
  sessionId: "session-planner",
  podName: "planner-pod",
  model: "hermes-test",
  metadata: { coderun: "cr-1", lane: "planning" },
};

const coder: AgentIdentity = {
  agentId: "coder-1",
  role: "coder",
  runtime: "hermes",
  projectId: "Project Alpha",
  taskId: "Task/42",
  metadata: { coderun: "cr-1", lane: "implementation" },
};

const reviewer: AgentIdentity = {
  agentId: "reviewer-1",
  role: "reviewer",
  runtime: "openclaw",
  projectId: "Project Alpha",
  taskId: "Task/99",
  metadata: { coderun: "cr-2" },
};

const outsider: AgentIdentity = {
  agentId: "outside-1",
  role: "coder",
  runtime: "external",
  projectId: "Project Beta",
  taskId: "Task/1",
};

function message(to: AgentMessage["to"], messageId = `msg-${Math.random()}`): AgentMessage<{ text: string }> {
  return {
    messageId,
    kind: "request",
    from: planner,
    to,
    body: { text: "coordinate" },
    createdAt: now,
    priority: "normal",
    metadata: { source: "unit-test" },
  };
}

function seededPlane() {
  const plane = new InMemoryCoordinationPlane({ now: () => now, defaultAgentTtlMs: 10_000 });
  plane.registerAgent(planner, { now, ttlMs: 10_000, metadata: { registeredBy: "test" } });
  plane.registerAgent(coder, { now, ttlMs: 10_000 });
  plane.registerAgent(reviewer, { now, ttlMs: 10_000 });
  plane.registerAgent(outsider, { now, ttlMs: 10_000 });
  plane.registerGroup({ groupId: "alpha-blue", name: "Alpha Blue", members: [planner, coder], projectId: "Project Alpha" });
  return plane;
}

test("registers agents with identity/project/task/role metadata and expiry", () => {
  const plane = new InMemoryCoordinationPlane({ now: () => now });
  const registered = plane.registerAgent(planner, { now, ttlMs: 1_000, metadata: { registeredBy: "unit" } });

  assert.equal(registered.identity.agentId, "planner-1");
  assert.equal(registered.identity.projectId, "Project Alpha");
  assert.equal(registered.identity.taskId, "Task/42");
  assert.equal(registered.identity.role, "planner");
  assert.deepEqual(registered.identity.metadata, { coderun: "cr-1", lane: "planning" });
  assert.equal(registered.expiresAt, "2026-04-01T00:00:01.000Z");
  assert.deepEqual(registered.metadata, { registeredBy: "unit" });

  assert.deepEqual(
    plane.lookup({ kind: "agent", agentId: "planner-1" }, now).map((agent) => agent.agentId),
    ["planner-1"],
  );
  assert.deepEqual(plane.lookup({ kind: "agent", agentId: "planner-1" }, later), []);
});

test("looks up agents by agent, project, task, role, group, and broadcast", () => {
  const plane = seededPlane();

  assert.deepEqual(
    plane.lookup({ kind: "agent", agentId: "coder-1", projectId: "Project Alpha", taskId: "Task/42" }).map((agent) => agent.agentId),
    ["coder-1"],
  );
  assert.deepEqual(
    plane.lookup({ kind: "project", projectId: "Project Alpha" }).map((agent) => agent.agentId),
    ["planner-1", "coder-1", "reviewer-1"],
  );
  assert.deepEqual(
    plane.lookup({ kind: "task", projectId: "Project Alpha", taskId: "Task/42" }).map((agent) => agent.agentId),
    ["planner-1", "coder-1"],
  );
  assert.deepEqual(
    plane.lookup({ kind: "role", role: "coder" }).map((agent) => agent.agentId),
    ["coder-1", "outside-1"],
  );
  assert.deepEqual(
    plane.lookup({ kind: "role", role: "coder", projectId: "Project Alpha" }).map((agent) => agent.agentId),
    ["coder-1"],
  );
  assert.deepEqual(
    plane.lookup({ kind: "group", groupId: "alpha-blue" }).map((agent) => agent.agentId),
    ["planner-1", "coder-1"],
  );
  assert.deepEqual(
    plane.lookup({ kind: "broadcast" }).map((agent) => agent.agentId),
    ["planner-1", "coder-1", "reviewer-1", "outside-1"],
  );
});

test("sends to project/task/role/group/broadcast targets and stores per-agent inbox messages", () => {
  const plane = seededPlane();

  const projectDelivery = plane.sendMessage(message({ kind: "project", projectId: "Project Alpha" }, "msg-project"), { now });
  assert.deepEqual(projectDelivery.recipients.map((agent) => agent.agentId), ["planner-1", "coder-1", "reviewer-1"]);

  const taskDelivery = plane.sendMessage(message({ kind: "task", projectId: "Project Alpha", taskId: "Task/42" }, "msg-task"), { now });
  assert.deepEqual(taskDelivery.recipients.map((agent) => agent.agentId), ["planner-1", "coder-1"]);

  const roleDelivery = plane.sendMessage(message({ kind: "role", role: "coder", projectId: "Project Alpha" }, "msg-role"), { now });
  assert.deepEqual(roleDelivery.recipients.map((agent) => agent.agentId), ["coder-1"]);

  const groupDelivery = plane.sendMessage(message({ kind: "group", groupId: "alpha-blue" }, "msg-group"), { now });
  assert.deepEqual(groupDelivery.recipients.map((agent) => agent.agentId), ["planner-1", "coder-1"]);

  const broadcastMessage = message({ kind: "agent", agentId: "nobody" }, "msg-broadcast");
  const { to: _to, ...broadcastBody } = broadcastMessage;
  const broadcastDelivery = plane.broadcast(broadcastBody, { now });
  assert.deepEqual(broadcastDelivery.recipients.map((agent) => agent.agentId), ["planner-1", "coder-1", "reviewer-1", "outside-1"]);

  assert.deepEqual(
    plane.readInbox("coder-1", { now }).map((entry) => entry.envelope.message.messageId),
    ["msg-project", "msg-task", "msg-role", "msg-group", "msg-broadcast"],
  );
});

test("acks inbox messages by delivery id or envelope id and keeps unacked messages durable while in memory", () => {
  const plane = seededPlane();
  const first = plane.sendMessage(message({ kind: "agent", agentId: "coder-1" }, "msg-ack-1"), { now });
  const second = plane.sendMessage(message({ kind: "agent", agentId: "coder-1" }, "msg-ack-2"), { now, envelopeId: "env-ack-2" });

  assert.deepEqual(
    plane.readInbox("coder-1", { now }).map((entry) => entry.envelope.message.messageId),
    ["msg-ack-1", "msg-ack-2"],
  );
  assert.equal(plane.ack("coder-1", first.deliveries[0].deliveryId), true);
  assert.deepEqual(
    plane.readInbox("coder-1", { now }).map((entry) => entry.envelope.message.messageId),
    ["msg-ack-2"],
  );
  assert.equal(plane.ack("coder-1", second.envelope.envelopeId), true);
  assert.deepEqual(plane.readInbox("coder-1", { now }), []);
  assert.equal(plane.ack("coder-1", "missing"), false);
});

test("filters expired messages from reads and does not deliver already-expired envelopes", () => {
  const plane = seededPlane();
  plane.sendMessage(message({ kind: "agent", agentId: "coder-1" }, "msg-expiring"), { now, ttlMs: 1_000 });

  assert.deepEqual(
    plane.readInbox("coder-1", { now }).map((entry) => entry.envelope.message.messageId),
    ["msg-expiring"],
  );
  assert.deepEqual(plane.readInbox("coder-1", { now: later }), []);

  const expiredEnvelope = createEnvelope(message({ kind: "agent", agentId: "coder-1" }, "msg-too-late"), {
    sentAt: now,
    ttlMs: 1_000,
  });
  const result = plane.sendEnvelope(expiredEnvelope, { now: later });

  assert.deepEqual(result.recipients, []);
  assert.deepEqual(plane.readInbox("coder-1", { now: later }), []);
});

test("increments delivery attempts and moves messages to dead letter deterministically", () => {
  const plane = new InMemoryCoordinationPlane({ now: () => now, maxDeliveryAttempts: 2 });
  plane.registerAgent(coder, { now, ttlMs: 10_000 });
  const sent = plane.sendMessage(message({ kind: "agent", agentId: "coder-1" }, "msg-retry"), { now });

  const retried = plane.failDelivery("coder-1", sent.envelope.envelopeId, { now: later, reason: "consumer unavailable" });
  assert.equal(retried?.attempts, 1);
  assert.deepEqual(plane.readDeadLetters("coder-1"), []);
  assert.deepEqual(
    plane.readInbox("coder-1", { now }).map((entry) => entry.attempts),
    [1],
  );

  const deadLetter = plane.failDelivery("coder-1", sent.deliveries[0].deliveryId, { now: later, reason: "still unavailable" });
  assert.equal(deadLetter?.attempts, 2);
  assert.deepEqual(plane.readInbox("coder-1", { now }), []);
  assert.deepEqual(
    plane.readDeadLetters("coder-1").map((entry) => ({ messageId: entry.envelope.message.messageId, reason: entry.reason })),
    [{ messageId: "msg-retry", reason: "still unavailable" }],
  );
});
