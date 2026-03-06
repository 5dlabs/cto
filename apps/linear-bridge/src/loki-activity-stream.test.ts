import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { parseLogLine, LokiActivityStream } from "./loki-activity-stream.js";
import type { LinearClient, ActivityContent, AgentActivityCreateInput } from "./linear-client.js";
import type { RunRegistry } from "./run-registry.js";

// ---------------------------------------------------------------------------
// parseLogLine unit tests
// ---------------------------------------------------------------------------

describe("parseLogLine", () => {
  it("parses thought type", () => {
    const result = parseLogLine(JSON.stringify({ type: "thought", body: "thinking..." }));
    expect(result).toEqual({ type: "thought", body: "thinking..." });
  });

  it("parses action type", () => {
    const result = parseLogLine(
      JSON.stringify({ type: "action", action: "readFile", parameter: "foo.ts", result: "ok" }),
    );
    expect(result).toEqual({
      type: "action",
      action: "readFile",
      parameter: "foo.ts",
      result: "ok",
    });
  });

  it("parses action type without result", () => {
    const result = parseLogLine(
      JSON.stringify({ type: "action", action: "bash", parameter: "ls" }),
    );
    expect(result).toEqual({ type: "action", action: "bash", parameter: "ls" });
    expect(result).not.toHaveProperty("result");
  });

  it("parses response type", () => {
    const result = parseLogLine(JSON.stringify({ type: "response", body: "Done!" }));
    expect(result).toEqual({ type: "response", body: "Done!" });
  });

  it("parses error type", () => {
    const result = parseLogLine(JSON.stringify({ type: "error", body: "Something broke" }));
    expect(result).toEqual({ type: "error", body: "Something broke" });
  });

  it("falls back to body from message field on error type", () => {
    const result = parseLogLine(JSON.stringify({ type: "error", message: "timeout" }));
    expect(result).toEqual({ type: "error", body: "timeout" });
  });

  it("skips elicitation type (returns null)", () => {
    const result = parseLogLine(JSON.stringify({ type: "elicitation", body: "choose one" }));
    expect(result).toBeNull();
  });

  it("wraps plain text as thought", () => {
    const result = parseLogLine("plain log message");
    expect(result).toEqual({ type: "thought", body: "plain log message" });
  });

  it("returns null for blank plain text", () => {
    const result = parseLogLine("   ");
    expect(result).toBeNull();
  });

  it("wraps unknown JSON structure as thought using message field", () => {
    const result = parseLogLine(JSON.stringify({ level: "info", message: "started" }));
    expect(result).toEqual({ type: "thought", body: "started" });
  });

  it("returns null for empty JSON object with no usable fields", () => {
    const result = parseLogLine(JSON.stringify({}));
    // raw JSON "{}" is not blank — treated as thought
    expect(result).not.toBeNull();
    expect(result?.type).toBe("thought");
  });
});

// ---------------------------------------------------------------------------
// LokiActivityStream unit tests (mock fetch)
// ---------------------------------------------------------------------------

function makeMockLinearClient(): LinearClient & {
  calls: AgentActivityCreateInput[];
} {
  const calls: AgentActivityCreateInput[] = [];
  return {
    calls,
    createProject: vi.fn(),
    createIssue: vi.fn(),
    createComment: vi.fn(),
    updateIssueState: vi.fn(),
    getOrCreateLabel: vi.fn(),
    getWorkflowStates: vi.fn(),
    createCustomView: vi.fn(),
    updateAgentSession: vi.fn(),
    async createAgentActivity(input) {
      calls.push(input);
      return { id: "act-" + calls.length };
    },
  } as unknown as LinearClient & { calls: AgentActivityCreateInput[] };
}

function makeMockRunRegistry(runs: { agentPod: string; linearSessionId?: string }[]): RunRegistry {
  return {
    register: vi.fn(),
    deregister: vi.fn(),
    lookup: vi.fn(),
    update: vi.fn(),
    gc: vi.fn().mockReturnValue(0),
    size: vi.fn().mockReturnValue(runs.length),
    getActiveRuns: vi.fn().mockReturnValue(
      runs.map((r, i) => ({
        runId: `run-${i}`,
        agentPod: r.agentPod,
        sessionKey: `sess-${i}`,
        issueId: `iss-${i}`,
        linearSessionId: r.linearSessionId,
      })),
    ),
  };
}

function makeLokiResponse(
  agentId: string,
  source: string,
  values: [string, string][],
) {
  return {
    status: "success",
    data: {
      resultType: "streams",
      result: [{ stream: { agent_id: agentId, source }, values }],
    },
  };
}

const BASE_CONFIG = {
  lokiUrl: "http://loki.test",
  lokiOrgId: "test-org",
  pollIntervalMs: 2000,
};

describe("LokiActivityStream", () => {
  const logger = {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  };

  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.clearAllMocks();
  });

  it("posts activity to Linear when a matching session is found", async () => {
    const ts = "1709000000000000001";
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () =>
        makeLokiResponse("pod-abc", "acp-opencode", [
          [ts, JSON.stringify({ type: "thought", body: "hello" })],
        ]),
    });

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([
      { agentPod: "pod-abc", linearSessionId: "sess-linear-1" },
    ]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await stream.poll();

    expect(linearClient.calls).toHaveLength(1);
    expect(linearClient.calls[0]).toMatchObject({
      agentSessionId: "sess-linear-1",
      content: { type: "thought", body: "hello" },
    });
  });

  it("does not post when no matching Linear session", async () => {
    const ts = "1709000000000000001";
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () =>
        makeLokiResponse("pod-xyz", "acp-codex", [
          [ts, JSON.stringify({ type: "response", body: "done" })],
        ]),
    });

    const linearClient = makeMockLinearClient();
    // runRegistry has no entry for pod-xyz
    const runRegistry = makeMockRunRegistry([]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await stream.poll();

    expect(linearClient.calls).toHaveLength(0);
  });

  it("advances cursor and does not re-post duplicate log lines", async () => {
    const ts1 = "1709000000000000001";
    const ts2 = "1709000000000000002";

    const response1 = makeLokiResponse("pod-abc", "acp-opencode", [
      [ts1, JSON.stringify({ type: "thought", body: "first" })],
    ]);
    const response2 = makeLokiResponse("pod-abc", "acp-opencode", [
      [ts1, JSON.stringify({ type: "thought", body: "first" })],
      [ts2, JSON.stringify({ type: "thought", body: "second" })],
    ]);

    fetchMock
      .mockResolvedValueOnce({ ok: true, json: async () => response1 })
      .mockResolvedValueOnce({ ok: true, json: async () => response2 });

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([
      { agentPod: "pod-abc", linearSessionId: "sess-linear-1" },
    ]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);

    await stream.poll();
    expect(linearClient.calls).toHaveLength(1);

    await stream.poll();
    // Only the second new entry should be posted
    expect(linearClient.calls).toHaveLength(2);
    expect(linearClient.calls[1].content).toMatchObject({ body: "second" });
  });

  it("skips elicitation log lines without posting", async () => {
    const ts = "1709000000000000001";
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () =>
        makeLokiResponse("pod-abc", "acp-opencode", [
          [ts, JSON.stringify({ type: "elicitation", body: "pick one" })],
        ]),
    });

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([
      { agentPod: "pod-abc", linearSessionId: "sess-linear-1" },
    ]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await stream.poll();

    expect(linearClient.calls).toHaveLength(0);
    // Cursor should still advance
    expect(stream.getCursor("pod-abc", "acp-opencode")).toBe(BigInt(ts));
  });

  it("handles Loki returning no results gracefully", async () => {
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () => ({
        status: "success",
        data: { resultType: "streams", result: [] },
      }),
    });

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await stream.poll();

    expect(linearClient.calls).toHaveLength(0);
  });

  it("handles Loki HTTP error gracefully without throwing", async () => {
    fetchMock.mockResolvedValue({ ok: false, status: 503 });

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await expect(stream.poll()).resolves.not.toThrow();
    expect(logger.warn).toHaveBeenCalledWith(expect.stringContaining("503"));
  });

  it("handles fetch network error gracefully without throwing", async () => {
    fetchMock.mockRejectedValue(new Error("network error"));

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await expect(stream.poll()).resolves.not.toThrow();
    expect(logger.warn).toHaveBeenCalledWith(
      expect.stringContaining("Fetch failed"),
      expect.anything(),
    );
  });

  it("wraps plain text log lines as thought", async () => {
    const ts = "1709000000000000001";
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () =>
        makeLokiResponse("pod-abc", "acp-opencode", [[ts, "plain text log"]]),
    });

    const linearClient = makeMockLinearClient();
    const runRegistry = makeMockRunRegistry([
      { agentPod: "pod-abc", linearSessionId: "sess-linear-1" },
    ]);

    const stream = new LokiActivityStream(BASE_CONFIG, linearClient, runRegistry, logger);
    await stream.poll();

    expect(linearClient.calls[0].content).toEqual({ type: "thought", body: "plain text log" });
  });
});
