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

// ── Helpers for config / stdio tests ─────────────────────────────────────────

const testBaseDir = new URL(".", import.meta.url).pathname;

/** Save CLIENT_CONFIG_PATH, set a new value, return a restore function. */
function setConfigPath(path: string): () => void {
  const prev = Deno.env.get("CLIENT_CONFIG_PATH");
  Deno.env.set("CLIENT_CONFIG_PATH", path);
  return () => {
    if (prev !== undefined) Deno.env.set("CLIENT_CONFIG_PATH", prev);
    else Deno.env.delete("CLIENT_CONFIG_PATH");
  };
}

/**
 * Minimal MCP server that speaks JSON-RPC over stdio.
 * Handles initialize, tools/list, and tools/call.
 */
const ECHO_SERVER_CODE = `\
const encoder = new TextEncoder();
let buffer = "";
const reader = Deno.stdin.readable.getReader();
const writer = Deno.stdout.writable.getWriter();
try {
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    buffer += new TextDecoder().decode(value);
    let nlIdx;
    while ((nlIdx = buffer.indexOf("\\n")) !== -1) {
      const line = buffer.slice(0, nlIdx).trim();
      buffer = buffer.slice(nlIdx + 1);
      if (!line) continue;
      try {
        const msg = JSON.parse(line);
        let resp;
        if (msg.method === "initialize") {
          resp = { jsonrpc: "2.0", id: msg.id, result: { protocolVersion: "2024-11-05", capabilities: { tools: {} }, serverInfo: { name: "echo-test" } } };
        } else if (msg.method === "tools/list") {
          resp = { jsonrpc: "2.0", id: msg.id, result: { tools: [{ name: "echosvr_ping", description: "Echo ping" }, { name: "echosvr_greet", description: "Echo greet" }] } };
        } else if (msg.method === "tools/call") {
          resp = { jsonrpc: "2.0", id: msg.id, result: { content: [{ type: "text", text: JSON.stringify({ echoed: true, tool: msg.params?.name }) }] } };
        } else { continue; }
        await writer.write(encoder.encode(JSON.stringify(resp) + "\\n"));
      } catch { /* skip non-JSON lines */ }
    }
  }
} catch { /* stdin closed */ }
`;

const echoServerPath = `${testBaseDir}_test_echo_mcp_server.ts`;
const echoConfigPath = `${testBaseDir}_test_echo_config.json`;

/** Write the echo server + config, import a fresh mcp module, run fn, clean up. */
// deno-lint-ignore no-explicit-any
async function withEchoServer(fn: (mod: any) => Promise<void>, cacheKey: string): Promise<void> {
  await Deno.writeTextFile(echoServerPath, ECHO_SERVER_CODE);
  await Deno.writeTextFile(
    echoConfigPath,
    JSON.stringify({
      remoteTools: [],
      localServers: {
        echosvr: {
          command: Deno.execPath(),
          args: ["run", "--no-check", echoServerPath],
          tools: ["echosvr_ping", "echosvr_greet"],
        },
      },
    }),
  );

  const restoreCfg = setConfigPath(echoConfigPath);
  try {
    const mod = await import(`./mcp.ts?_stdio=${cacheKey}_${Date.now()}`);
    await fn(mod);
  } finally {
    globalThis.dispatchEvent(new Event("unload"));
    await new Promise((r) => setTimeout(r, 100));
    restoreCfg();
    await Deno.remove(echoServerPath).catch(() => {});
    await Deno.remove(echoConfigPath).catch(() => {});
  }
}

// ── loadClientConfig tests ───────────────────────────────────────────────────

Deno.test({
  name: "loadClientConfig: valid config routes tool to stdio, not HTTP",
  sanitizeResources: false,
  sanitizeOps: false,
  async fn() {
    const cfgPath = `${testBaseDir}_test_cfg_valid.json`;
    await Deno.writeTextFile(
      cfgPath,
      JSON.stringify({
        remoteTools: [],
        localServers: {
          mysvr: { command: "cat", args: [], tools: ["mysvr_hello"] },
        },
      }),
    );
    const restoreCfg = setConfigPath(cfgPath);

    try {
      const mod = await import(`./mcp.ts?_lc=valid_${Date.now()}`);

      let fetchCalled = false;
      const savedFetch = globalThis.fetch;
      globalThis.fetch = (() => {
        fetchCalled = true;
        return Promise.resolve(new Response("", { status: 500 }));
      }) as typeof fetch;

      try {
        await mod.describeTool("mysvr_hello");
      } catch {
        // Expected: "cat" is not an MCP server
      }

      globalThis.fetch = savedFetch;
      assertEquals(fetchCalled, false, "Tool in localServers should route to stdio, not HTTP");
    } finally {
      globalThis.dispatchEvent(new Event("unload"));
      restoreCfg();
      await Deno.remove(cfgPath).catch(() => {});
    }
  },
});

Deno.test("loadClientConfig: missing file falls back to HTTP routing", async () => {
  const restoreCfg = setConfigPath("/nonexistent/no_such_config.json");

  try {
    const mod = await import(`./mcp.ts?_lc=missing_${Date.now()}`);
    const { calls, restore } = stubFetch((_url, body) =>
      jsonRpcOk(body.id as number, { tools: [{ name: "github_test", description: "Test" }] }),
    );

    try {
      await mod.describeTool("github_test");
      assert(calls.length > 0, "With missing config, tool should route to HTTP");
      assertEquals(calls[0].url, "http://test-server/mcp");
    } finally {
      restore();
    }
  } finally {
    restoreCfg();
  }
});

Deno.test("loadClientConfig: malformed JSON falls back to HTTP routing", async () => {
  const cfgPath = `${testBaseDir}_test_cfg_bad.json`;
  await Deno.writeTextFile(cfgPath, "{NOT VALID JSON!!!");
  const restoreCfg = setConfigPath(cfgPath);

  try {
    const mod = await import(`./mcp.ts?_lc=malformed_${Date.now()}`);
    const { calls, restore } = stubFetch((_url, body) =>
      jsonRpcOk(body.id as number, { tools: [{ name: "github_test", description: "Test" }] }),
    );

    try {
      await mod.describeTool("github_test");
      assert(calls.length > 0, "With malformed config, tool should route to HTTP");
    } finally {
      restore();
    }
  } finally {
    restoreCfg();
    await Deno.remove(cfgPath).catch(() => {});
  }
});

// ── transportFor routing tests ───────────────────────────────────────────────

Deno.test({
  name: "transportFor: tool matching localServers key prefix → stdio",
  sanitizeResources: false,
  sanitizeOps: false,
  async fn() {
    const cfgPath = `${testBaseDir}_test_cfg_prefix.json`;
    await Deno.writeTextFile(
      cfgPath,
      JSON.stringify({
        remoteTools: [],
        localServers: {
          docker: { command: "cat", args: [], tools: [] },
        },
      }),
    );
    const restoreCfg = setConfigPath(cfgPath);

    try {
      const mod = await import(`./mcp.ts?_tf=prefix_${Date.now()}`);

      let fetchCalled = false;
      const savedFetch = globalThis.fetch;
      globalThis.fetch = (() => {
        fetchCalled = true;
        return Promise.resolve(new Response("", { status: 500 }));
      }) as typeof fetch;

      try {
        // "docker_run" prefix "docker" matches a localServers key
        await mod.describeTool("docker_run");
      } catch {
        // Expected failure
      }

      globalThis.fetch = savedFetch;
      assertEquals(fetchCalled, false, "Tool prefix matching localServers key should use stdio");
    } finally {
      globalThis.dispatchEvent(new Event("unload"));
      restoreCfg();
      await Deno.remove(cfgPath).catch(() => {});
    }
  },
});

Deno.test("transportFor: LOCAL_TOOLS match → local HTTP URL", async () => {
  const restoreCfg = setConfigPath("/nonexistent_routing_test_local");

  try {
    const mod = await import(`./mcp.ts?_tf=localhttp_${Date.now()}`);
    const { calls, restore } = stubFetch((_url, body) =>
      jsonRpcOk(body.id as number, { tools: [{ name: "filesystem_read", description: "Read" }] }),
    );

    try {
      await mod.describeTool("filesystem_read");
      assertEquals(
        calls[0].url,
        "http://local-server/mcp",
        "LOCAL_TOOLS prefix should route to LOCAL_TOOLS_URL",
      );
    } finally {
      restore();
    }
  } finally {
    restoreCfg();
  }
});

Deno.test("transportFor: no match → remote HTTP URL", async () => {
  const restoreCfg = setConfigPath("/nonexistent_routing_test_remote");

  try {
    const mod = await import(`./mcp.ts?_tf=remote_${Date.now()}`);
    const { calls, restore } = stubFetch((_url, body) =>
      jsonRpcOk(body.id as number, { tools: [{ name: "linear_create", description: "Create" }] }),
    );

    try {
      await mod.describeTool("linear_create");
      assertEquals(
        calls[0].url,
        "http://test-server/mcp",
        "Unmatched tool should route to remote TOOLS_SERVER_URL",
      );
    } finally {
      restore();
    }
  } finally {
    restoreCfg();
  }
});

// ── StdioTransport integration tests ─────────────────────────────────────────

Deno.test({
  name: "StdioTransport: handshake and tools/list via echo server",
  sanitizeResources: false,
  sanitizeOps: false,
  async fn() {
    await withEchoServer(async (mod) => {
      // Stub fetch for the remote HTTP leg of listTools()
      const savedFetch = globalThis.fetch;
      globalThis.fetch = ((_input: string | URL | Request, _init?: RequestInit) =>
        Promise.resolve(
          new Response(
            JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              result: { tools: [{ name: "remote_tool", description: "Remote" }] },
            }),
            { status: 200 },
          ),
        )) as typeof fetch;

      try {
        const tools = await mod.listTools();

        // Remote tools present
        assert("remote" in tools, "Should include remote tools");
        assertEquals(tools["remote"][0].name, "remote_tool");

        // Local stdio tools from echo server present
        assert("echosvr" in tools, "Should include echosvr tools");
        const echoNames = tools["echosvr"].map((t: { name: string }) => t.name).sort();
        assertEquals(echoNames, ["echosvr_greet", "echosvr_ping"]);
      } finally {
        globalThis.fetch = savedFetch;
      }
    }, "handshake");
  },
});

Deno.test({
  name: "StdioTransport: callTool routes to stdio and returns parsed result",
  sanitizeResources: false,
  sanitizeOps: false,
  async fn() {
    await withEchoServer(async (mod) => {
      const result = await mod.callTool("echosvr_ping", { data: "hello" });
      assertEquals(result.echoed, true);
      assertEquals(result.tool, "echosvr_ping");
    }, "calltool");
  },
});

Deno.test({
  name: "StdioTransport: describeTool via stdio returns tool info",
  sanitizeResources: false,
  sanitizeOps: false,
  async fn() {
    await withEchoServer(async (mod) => {
      const info = await mod.describeTool("echosvr_ping");
      assertEquals(info.name, "echosvr_ping");
      assertEquals(info.description, "Echo ping");
    }, "describe");
  },
});

// ── Process cleanup test ─────────────────────────────────────────────────────

Deno.test({
  name: "StdioTransport: unload event cleans up, subsequent call re-spawns",
  sanitizeResources: false,
  sanitizeOps: false,
  async fn() {
    await Deno.writeTextFile(echoServerPath, ECHO_SERVER_CODE);
    await Deno.writeTextFile(
      echoConfigPath,
      JSON.stringify({
        remoteTools: [],
        localServers: {
          echosvr: {
            command: Deno.execPath(),
            args: ["run", "--no-check", echoServerPath],
            tools: ["echosvr_ping"],
          },
        },
      }),
    );

    const restoreCfg = setConfigPath(echoConfigPath);
    try {
      const mod = await import(`./mcp.ts?_cleanup=${Date.now()}`);

      // First call — spawns a child process
      const r1 = await mod.callTool("echosvr_ping", {});
      assertEquals(r1.echoed, true);

      // Dispatch unload — should kill child and clear transport cache
      globalThis.dispatchEvent(new Event("unload"));
      await new Promise((r) => setTimeout(r, 200));

      // Second call — must spawn a NEW child process and still succeed
      const r2 = await mod.callTool("echosvr_ping", {});
      assertEquals(r2.echoed, true, "Should work after cleanup + re-spawn");

      // Final cleanup
      globalThis.dispatchEvent(new Event("unload"));
      await new Promise((r) => setTimeout(r, 100));
    } finally {
      restoreCfg();
      await Deno.remove(echoServerPath).catch(() => {});
      await Deno.remove(echoConfigPath).catch(() => {});
    }
  },
});
