import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  appendMorganCommand,
  createMorganSidecar,
  listMorganTools,
  tailMorganEvents,
} from "./index.js";

test("health reports writable Morgan workspace streams and ready MCP registry", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "morgan-sidecar-health-"));
  try {
    const sidecar = await createMorganSidecar({
      workspaceDir: workspace,
      coderunId: "coderun-health",
      sessionId: "morgan_coderun-health_1770000000",
      now: () => new Date("2026-05-04T00:00:00.000Z"),
    });

    const health = await sidecar.health();

    assert.equal(health.ok, true);
    assert.equal(health.service, "morgan-agent-sidecar");
    assert.deepEqual(health.mcp, { port: 4000, ready: true });
    assert.equal(health.workspace.event_log_writable, true);
    assert.equal(health.workspace.command_log_readable, true);
    assert.equal(health.workspace.status_file_writable, true);
    assert.equal(health.providers.mode, "stub");
    assert.equal(health.providers.livekit, "missing_optional");
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("lists deterministic Morgan and Morgan Meet compatibility MCP tools", () => {
  assert.deepEqual(
    listMorganTools().map((tool) => tool.name),
    [
      "morgan_session_start",
      "morgan_session_stop",
      "morgan_session_status",
      "morgan_say",
      "morgan_set_state",
      "morgan_events_tail",
      "meet_join",
      "meet_leave",
      "meet_get_status",
      "meet_stream_audio",
    ],
  );
});

test("session tools write JSONL events, append commands, and update status snapshot", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "morgan-sidecar-streams-"));
  try {
    const sidecar = await createMorganSidecar({
      workspaceDir: workspace,
      coderunId: "coderun-streams",
      agentId: "morgan",
      sessionId: "morgan_coderun-streams_1770000000",
      now: (() => {
        let tick = 0;
        return () => new Date(`2026-05-04T00:00:0${tick++}.000Z`);
      })(),
    });

    const start = await sidecar.callTool("meet_join", {
      surface: { kind: "meeting_url", url: "https://meet.example/redacted" },
      mode: "symbolic",
    });
    const say = await sidecar.callTool("morgan_say", { text: "Hello from stub Morgan" });
    const stop = await sidecar.callTool("meet_leave", { reason: "unit-test" });

    assert.equal(start.status, "starting");
    assert.equal(say.status, "queued");
    assert.equal(stop.status, "stopped");

    const status = JSON.parse(await readFile(sidecar.paths.statusFile, "utf8"));
    assert.equal(status.session_id, "morgan_coderun-streams_1770000000");
    assert.equal(status.status, "stopped");
    assert.equal(status.state, "idle");
    assert.equal(status.last_event_seq, 4);

    const events = await tailMorganEvents(sidecar.paths.eventLog, 10);
    assert.deepEqual(
      events.map((event) => event.type),
      ["sidecar_ready", "session_started", "turn_queued", "session_stopped"],
    );
    assert.deepEqual(
      events.map((event) => event.seq),
      [1, 2, 3, 4],
    );

    await appendMorganCommand(sidecar.paths.commandLog, {
      source: "acpx",
      session_id: "morgan_coderun-streams_1770000000",
      type: "say",
      payload: { text: "replay me" },
      now: () => new Date("2026-05-04T00:00:09.000Z"),
    });
    const commandLog = await readFile(sidecar.paths.commandLog, "utf8");
    assert.match(commandLog, /"command_id":"cmd_000001"/);
    assert.match(commandLog, /"type":"say"/);
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("unsupported meeting audio streaming returns controlled stub result", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "morgan-sidecar-audio-"));
  try {
    const sidecar = await createMorganSidecar({ workspaceDir: workspace });
    const result = await sidecar.callTool("meet_stream_audio", { url: "https://example.invalid/audio.wav" });

    assert.equal(result.status, "not_implemented");
    assert.equal(result.provider_mode, "stub");
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});
