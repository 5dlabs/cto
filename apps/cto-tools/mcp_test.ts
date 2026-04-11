// mcp_test.ts — Deno tests for the MCP tool-calling runtime
//
// Strategy: env vars are read at module-load time, so we set them BEFORE the
// dynamic import. globalThis.fetch is stubbed per-test to intercept HTTP calls.

import {
  assert,
  assertEquals,
  assertRejects,
  assertStringIncludes,
} from "https://deno.land/std@0.224.0/assert/mod.ts";

// ── Env setup (must happen before the module is imported) ────────────────────

Deno.env.set("TOOLS_SERVER_URL", "http://test-server/mcp");
Deno.env.set("LOCAL_TOOLS_URL", "http://local-server/mcp");
Deno.env.set("LOCAL_TOOLS", "filesystem,memory");
Deno.env.set("CTO_AGENT_ID", "test-agent");
Deno.env.set("CTO_AGENT_PREWARM", "");

// Now import the module under test (env vars are already set).
const {
  listTools,
  describeTool,
  callTool,
  escalate,
  ToolError,
  ErrorCodes,
} = await import("./mcp.ts");

// ── Fetch stub helpers ───────────────────────────────────────────────────────

interface FetchCall {
  url: string;
  init: RequestInit;
  body: Record<string, unknown>;
}

const originalFetch = globalThis.fetch;

/** Replace globalThis.fetch with a stub that records calls and returns canned responses. */
function stubFetch(
  handler: (url: string, body: Record<string, unknown>) => Response | Promise<Response>,
): { calls: FetchCall[]; restore: () => void } {
  const calls: FetchCall[] = [];

  globalThis.fetch = async (input: string | URL | Request, init?: RequestInit): Promise<Response> => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url;
    const body = JSON.parse((init?.body as string) ?? "{}");
    calls.push({ url, init: init ?? {}, body });
    return handler(url, body);
  };

  return {
    calls,
    restore: () => {
      globalThis.fetch = originalFetch;
    },
  };
}

/** Build a JSON-RPC success response. */
function jsonRpcOk(id: number, result: unknown): Response {
  return new Response(JSON.stringify({ jsonrpc: "2.0", id, result }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

/** Build a JSON-RPC error response. */
function jsonRpcErr(id: number, code: number, message: string, data?: unknown): Response {
  return new Response(
    JSON.stringify({ jsonrpc: "2.0", id, error: { code, message, ...(data !== undefined && { data }) } }),
    { status: 200, headers: { "Content-Type": "application/json" } },
  );
}

// ── Tests ────────────────────────────────────────────────────────────────────

Deno.test("listTools() groups tools by server prefix", async () => {
  const tools = [
    { name: "github_search_code", description: "Search code" },
    { name: "github_get_file", description: "Get file" },
    { name: "linear_create_issue", description: "Create issue" },
  ];

  const { calls, restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, { tools }),
  );

  try {
    const result = await listTools();

    // Correct grouping
    assertEquals(Object.keys(result).sort(), ["github", "linear"]);
    assertEquals(result["github"].length, 2);
    assertEquals(result["linear"].length, 1);
    assertEquals(result["github"][0].name, "github_search_code");
    assertEquals(result["linear"][0].name, "linear_create_issue");

    // Sent to remote server URL
    assertEquals(calls[0].url, "http://test-server/mcp");

    // X-Agent-Id header present
    const headers = calls[0].init.headers as Record<string, string>;
    assertEquals(headers["X-Agent-Id"], "test-agent");

    // Correct JSON-RPC method
    assertEquals(calls[0].body.method, "tools/list");
  } finally {
    restore();
  }
});

Deno.test("describeTool() returns matching tool info", async () => {
  const tools = [
    { name: "github_search_code", description: "Search code", inputSchema: { type: "object" } },
    { name: "github_get_file", description: "Get file" },
  ];

  const { restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, { tools }),
  );

  try {
    const info = await describeTool("github_search_code");
    assertEquals(info.name, "github_search_code");
    assertEquals(info.description, "Search code");
    assertEquals(info.inputSchema, { type: "object" });
  } finally {
    restore();
  }
});

Deno.test("describeTool() throws TOOL_NOT_FOUND for unknown tool", async () => {
  const { restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, { tools: [] }),
  );

  try {
    await assertRejects(
      () => describeTool("nonexistent_tool"),
      ToolError,
      "Tool not found: nonexistent_tool",
    );
  } finally {
    restore();
  }
});

Deno.test("describeTool() routes local tools to LOCAL_TOOLS_URL", async () => {
  const tools = [{ name: "filesystem_read_file", description: "Read a file" }];

  const { calls, restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, { tools }),
  );

  try {
    await describeTool("filesystem_read_file");

    // Should hit the local server, not the remote one
    assertEquals(calls[0].url, "http://local-server/mcp");
  } finally {
    restore();
  }
});

Deno.test("callTool() parses JSON text content", async () => {
  const { calls, restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, {
      content: [{ type: "text", text: '{"count": 5}' }],
    }),
  );

  try {
    const result = await callTool<{ count: number }>("github_search_code", { q: "test" });

    assertEquals(result, { count: 5 });

    // Verify request body
    assertEquals(calls[0].body.method, "tools/call");
    assertEquals((calls[0].body.params as Record<string, unknown>).name, "github_search_code");
    assertEquals(
      (calls[0].body.params as Record<string, unknown>).arguments,
      { q: "test" },
    );
  } finally {
    restore();
  }
});

Deno.test("callTool() returns raw string when content is not JSON", async () => {
  const { restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, {
      content: [{ type: "text", text: "plain text result" }],
    }),
  );

  try {
    const result = await callTool<string>("github_search_code", { q: "test" });
    assertEquals(result, "plain text result");
  } finally {
    restore();
  }
});

Deno.test("callTool() throws ToolError on JSON-RPC error", async () => {
  const { restore } = stubFetch((_url, body) =>
    jsonRpcErr(body.id as number, -32403, "policy denied"),
  );

  try {
    try {
      await callTool("github_search_code", { q: "test" });
      throw new Error("Expected ToolError");
    } catch (err) {
      assert(err instanceof ToolError, "Expected ToolError");
      assertEquals(err.code, ErrorCodes.POLICY_DENIED);
      assertStringIncludes(err.message, "policy denied");
    }
  } finally {
    restore();
  }
});

Deno.test("callTool() throws ToolError when no text content", async () => {
  const { restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, { content: [] }),
  );

  try {
    try {
      await callTool("github_search_code", {});
      throw new Error("Expected ToolError");
    } catch (err) {
      assert(err instanceof ToolError, "Expected ToolError");
      assertEquals(err.code, ErrorCodes.SERVER_ERROR);
      assertStringIncludes(err.message, "returned no text content");
    }
  } finally {
    restore();
  }
});

Deno.test("rpc() retries on 503 then succeeds", async () => {
  let attempt = 0;

  const { calls, restore } = stubFetch((_url, body) => {
    attempt++;
    if (attempt === 1) {
      return new Response("Service Unavailable", { status: 503 });
    }
    return jsonRpcOk(body.id as number, {
      content: [{ type: "text", text: '"ok"' }],
    });
  });

  try {
    const result = await callTool<string>("github_search_code", { q: "retry" });
    assertEquals(result, "ok");
    // First call got 503, second succeeded
    assertEquals(calls.length, 2);
  } finally {
    restore();
  }
});

Deno.test("escalate() calls tools_request_capability with correct args", async () => {
  const grantedTool = { name: "admin_delete", description: "Delete resource" };

  const { calls, restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, {
      content: [{ type: "text", text: JSON.stringify(grantedTool) }],
    }),
  );

  try {
    const result = await escalate("admin_delete", "need cleanup access");

    assertEquals(result.name, "admin_delete");
    assertEquals(result.description, "Delete resource");

    // Verify the RPC call
    const params = calls[0].body.params as Record<string, unknown>;
    assertEquals(params.name, "tools_request_capability");
    assertEquals(params.arguments, {
      tool_name: "admin_delete",
      reason: "need cleanup access",
    });
  } finally {
    restore();
  }
});

Deno.test("callTool() routes local tool to LOCAL_TOOLS_URL", async () => {
  const { calls, restore } = stubFetch((_url, body) =>
    jsonRpcOk(body.id as number, {
      content: [{ type: "text", text: '"data"' }],
    }),
  );

  try {
    await callTool("memory_store", { key: "x", value: "y" });
    assertEquals(calls[0].url, "http://local-server/mcp");
  } finally {
    restore();
  }
});
