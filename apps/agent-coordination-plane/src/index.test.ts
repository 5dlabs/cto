import assert from "node:assert/strict";
import test from "node:test";
import {
  addressKey,
  createEnvelope,
  safeSubjectSegment,
  subjectForAddress,
  validateAgentEnvelope,
  type AgentIdentity,
  type AgentMessage,
} from "./index.js";

const forge: AgentIdentity = {
  agentId: "Forge-1",
  role: "coder",
  runtime: "hermes",
  projectId: "Project Alpha",
  taskId: "Task/42",
};

test("maps direct agent addressing to a stable coordination subject", () => {
  assert.equal(
    subjectForAddress({ kind: "agent", agentId: "Forge-1", projectId: "Project Alpha", taskId: "Task/42" }),
    "cto.agent.v1.agent.forge-1.inbox",
  );
  assert.equal(
    addressKey({ kind: "agent", agentId: "Forge-1", projectId: "Project Alpha", taskId: "Task/42" }),
    "agent:project-alpha:task-42:forge-1",
  );
});

test("maps project, task, role, group, and broadcast addressing", () => {
  assert.equal(
    subjectForAddress({ kind: "task", projectId: "Project Alpha", taskId: "Task/42" }),
    "cto.agent.v1.project.project-alpha.task.task-42.inbox",
  );
  assert.equal(
    subjectForAddress({ kind: "project", projectId: "Project Alpha" }),
    "cto.agent.v1.project.project-alpha.inbox",
  );
  assert.equal(
    subjectForAddress({ kind: "role", role: "Reviewer", projectId: "Project Alpha" }),
    "cto.agent.v1.role.reviewer.inbox",
  );
  assert.equal(
    subjectForAddress({ kind: "group", groupId: "Blue Team" }),
    "cto.agent.v1.group.blue-team.inbox",
  );
  assert.equal(subjectForAddress({ kind: "broadcast" }), "cto.agent.v1.broadcast");
});

test("normalizes invalid subject segments with fallback", () => {
  assert.equal(safeSubjectSegment(" ??? "), "unknown");
  assert.equal(subjectForAddress({ kind: "agent", agentId: " !!! " }), "cto.agent.v1.agent.unknown.inbox");
});

function sampleEnvelope() {
  const message: AgentMessage<{ text: string }> = {
    messageId: "msg-1",
    kind: "request",
    from: forge,
    to: { kind: "role", role: "reviewer", projectId: "Project Alpha", taskId: "Task/42" },
    body: { text: "please review" },
    createdAt: "2026-04-01T00:00:00.000Z",
    priority: "high",
    correlationId: "corr-1",
    metadata: { source: "unit-test" },
  };

  return createEnvelope(message, {
    envelopeId: "env-1",
    sentAt: "2026-04-01T00:00:01.000Z",
    replyTo: "cto.agent.v1.agent.forge-1.inbox",
    ttlMs: 30_000,
    attempt: 1,
    trace: { traceId: "trace-1" },
  });
}

test("creates an agent envelope preserving correlation and transport metadata", () => {
  const envelope = sampleEnvelope();

  assert.equal(envelope.schema, "cto.agent.envelope.v1");
  assert.equal(envelope.envelopeId, "env-1");
  assert.equal(envelope.subject, "cto.agent.v1.role.reviewer.inbox");
  assert.equal(envelope.message.correlationId, "corr-1");
  assert.equal(envelope.replyTo, "cto.agent.v1.agent.forge-1.inbox");
  assert.equal(envelope.ttlMs, 30_000);
  assert.deepEqual(envelope.trace, { traceId: "trace-1" });
});

test("validates coordination envelopes before transport adapters trust them", () => {
  const result = validateAgentEnvelope(sampleEnvelope());

  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.value.envelopeId, "env-1");
  }
});

test("rejects malformed coordination envelopes and non-string metadata", () => {
  const envelope = sampleEnvelope() as unknown as Record<string, unknown>;
  envelope.schema = "cto.presence.v1";
  envelope.ttlMs = -1;
  envelope.message = {
    ...(envelope.message as Record<string, unknown>),
    metadata: { source: 123 },
  };

  const result = validateAgentEnvelope(envelope);

  assert.equal(result.ok, false);
  if (!result.ok) {
    assert.deepEqual(
      result.issues.map((entry) => entry.path),
      ["schema", "ttlMs", "message.metadata"],
    );
  }
});
