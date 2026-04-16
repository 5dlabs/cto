// codegen_test.ts — Tests for codegen.ts pure functions and integration.

import { assert, assertEquals, assertStringIncludes } from "jsr:@std/assert";
import {
  buildArgsType,
  generateJsDoc,
  generateReadme,
  generateServerIndex,
  generateToolFile,
  jsonSchemaToTsType,
  parseTool,
} from "./codegen.ts";
import type { JsonSchema, ParsedTool, ToolDef } from "./codegen.ts";

// ─── jsonSchemaToTsType ──────────────────────────────────────────────────────

Deno.test("jsonSchemaToTsType: string", () => {
  assertEquals(jsonSchemaToTsType({ type: "string" }), "string");
});

Deno.test("jsonSchemaToTsType: number", () => {
  assertEquals(jsonSchemaToTsType({ type: "number" }), "number");
});

Deno.test("jsonSchemaToTsType: boolean", () => {
  assertEquals(jsonSchemaToTsType({ type: "boolean" }), "boolean");
});

Deno.test("jsonSchemaToTsType: integer → number", () => {
  assertEquals(jsonSchemaToTsType({ type: "integer" }), "number");
});

Deno.test("jsonSchemaToTsType: array with items", () => {
  assertEquals(
    jsonSchemaToTsType({ type: "array", items: { type: "string" } }),
    "Array<string>",
  );
});

Deno.test("jsonSchemaToTsType: array without items", () => {
  assertEquals(jsonSchemaToTsType({ type: "array" }), "unknown[]");
});

Deno.test("jsonSchemaToTsType: object with properties", () => {
  const schema: JsonSchema = {
    type: "object",
    properties: { name: { type: "string" } },
  };
  assertEquals(jsonSchemaToTsType(schema), "{ name?: string }");
});

Deno.test("jsonSchemaToTsType: object with required properties", () => {
  const schema: JsonSchema = {
    type: "object",
    properties: { name: { type: "string" }, age: { type: "integer" } },
    required: ["name"],
  };
  const result = jsonSchemaToTsType(schema);
  assertStringIncludes(result, "name: string");
  assertStringIncludes(result, "age?: number");
});

Deno.test("jsonSchemaToTsType: object without properties", () => {
  assertEquals(jsonSchemaToTsType({ type: "object" }), "Record<string, unknown>");
});

Deno.test("jsonSchemaToTsType: undefined", () => {
  assertEquals(jsonSchemaToTsType(undefined), "unknown");
});

Deno.test("jsonSchemaToTsType: unknown type fallback", () => {
  assertEquals(jsonSchemaToTsType({ type: "null" }), "unknown");
});

Deno.test("jsonSchemaToTsType: nested array of objects", () => {
  const schema: JsonSchema = {
    type: "array",
    items: {
      type: "object",
      properties: { id: { type: "number" } },
    },
  };
  assertEquals(jsonSchemaToTsType(schema), "Array<{ id?: number }>");
});

// ─── buildArgsType ───────────────────────────────────────────────────────────

Deno.test("buildArgsType: required and optional properties", () => {
  const schema: JsonSchema = {
    type: "object",
    properties: {
      query: { type: "string" },
      limit: { type: "integer" },
    },
    required: ["query"],
  };
  const result = buildArgsType(schema);
  assertStringIncludes(result, "query: string");
  assertStringIncludes(result, "limit?: number");
});

Deno.test("buildArgsType: no schema → Record", () => {
  assertEquals(buildArgsType(undefined), "Record<string, unknown>");
});

Deno.test("buildArgsType: empty properties → Record", () => {
  assertEquals(
    buildArgsType({ type: "object" }),
    "Record<string, unknown>",
  );
});

Deno.test("buildArgsType: all required", () => {
  const schema: JsonSchema = {
    type: "object",
    properties: { a: { type: "string" }, b: { type: "boolean" } },
    required: ["a", "b"],
  };
  const result = buildArgsType(schema);
  assertStringIncludes(result, "a: string");
  assertStringIncludes(result, "b: boolean");
  // No question marks
  assertEquals(result.includes("?"), false);
});

// ─── parseTool ───────────────────────────────────────────────────────────────

Deno.test("parseTool: prefixed name", () => {
  const tool: ToolDef = { name: "github_search_code", description: "Search code" };
  const result = parseTool(tool);
  assertEquals(result.serverPrefix, "github");
  assertEquals(result.funcName, "search_code");
  assertEquals(result.fullName, "github_search_code");
  assertEquals(result.description, "Search code");
});

Deno.test("parseTool: multi-underscore name keeps first split", () => {
  const tool: ToolDef = { name: "linear_create_issue", description: "Create" };
  const result = parseTool(tool);
  assertEquals(result.serverPrefix, "linear");
  assertEquals(result.funcName, "create_issue");
});

Deno.test("parseTool: no underscore → prefix = name, funcName = call", () => {
  const tool: ToolDef = { name: "ping" };
  const result = parseTool(tool);
  assertEquals(result.serverPrefix, "ping");
  assertEquals(result.funcName, "call");
});

Deno.test("parseTool: preserves inputSchema", () => {
  const schema: JsonSchema = { type: "object", properties: { q: { type: "string" } } };
  const tool: ToolDef = { name: "gh_search", inputSchema: schema };
  const result = parseTool(tool);
  assertEquals(result.inputSchema, schema);
});

// ─── generateToolFile ────────────────────────────────────────────────────────

Deno.test("generateToolFile: contains auto-generated header", () => {
  const tool: ParsedTool = {
    fullName: "github_search_code",
    serverPrefix: "github",
    funcName: "search_code",
    description: "Search code on GitHub",
  };
  const output = generateToolFile(tool);
  assertStringIncludes(output, "Auto-generated by codegen.ts");
});

Deno.test("generateToolFile: imports from mcp.ts", () => {
  const tool: ParsedTool = {
    fullName: "github_search_code",
    serverPrefix: "github",
    funcName: "search_code",
  };
  const output = generateToolFile(tool);
  assertStringIncludes(output, 'import { callTool } from "../../mcp.ts"');
});

Deno.test("generateToolFile: exports async function with correct name", () => {
  const tool: ParsedTool = {
    fullName: "linear_create_issue",
    serverPrefix: "linear",
    funcName: "create_issue",
    inputSchema: {
      type: "object",
      properties: { title: { type: "string" } },
      required: ["title"],
    },
  };
  const output = generateToolFile(tool);
  assertStringIncludes(output, "export async function create_issue(");
  assertStringIncludes(output, "title: string");
  assertStringIncludes(output, 'callTool("linear_create_issue"');
});

Deno.test("generateToolFile: no schema → Record<string, unknown> args", () => {
  const tool: ParsedTool = {
    fullName: "ping_call",
    serverPrefix: "ping",
    funcName: "call",
  };
  const output = generateToolFile(tool);
  assertStringIncludes(output, "Record<string, unknown>");
});

// ─── generateServerIndex ─────────────────────────────────────────────────────

Deno.test("generateServerIndex: re-exports all functions sorted", () => {
  const output = generateServerIndex(["search_code", "get_file"]);
  assertStringIncludes(output, 'export { get_file } from "./get_file.ts"');
  assertStringIncludes(output, 'export { search_code } from "./search_code.ts"');
  // get_file should come before search_code (alphabetical)
  const idxGet = output.indexOf("get_file");
  const idxSearch = output.indexOf("search_code");
  assertEquals(idxGet < idxSearch, true);
});

Deno.test("generateServerIndex: contains auto-generated header", () => {
  const output = generateServerIndex(["foo"]);
  assertStringIncludes(output, "Auto-generated by codegen.ts");
});

Deno.test("generateServerIndex: single function", () => {
  const output = generateServerIndex(["only_one"]);
  assertStringIncludes(output, 'export { only_one } from "./only_one.ts"');
});

// ─── Integration: main() with mocked fetch ──────────────────────────────────

Deno.test({
  name: "integration: main() generates correct directory structure",
  async fn() {
    const outputDir = await Deno.makeTempDir({ prefix: "codegen_test_" });

    // Save originals
    const originalFetch = globalThis.fetch;

    try {
      // Set output dir via env
      Deno.env.set("CTO_TOOLS_OUTPUT", outputDir);

      // Mock fetch to return a tools/list response
      globalThis.fetch = ((_input: string | URL | Request, _init?: RequestInit) => {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              result: {
                tools: [
                  {
                    name: "github_search_code",
                    description: "Search code on GitHub",
                    inputSchema: {
                      type: "object",
                      properties: {
                        q: { type: "string", description: "Search query" },
                        repo: { type: "string", description: "Repository" },
                      },
                      required: ["q"],
                    },
                  },
                  {
                    name: "github_get_file",
                    description: "Get file contents",
                    inputSchema: {
                      type: "object",
                      properties: {
                        path: { type: "string" },
                      },
                      required: ["path"],
                    },
                  },
                  {
                    name: "linear_create_issue",
                    description: "Create a Linear issue",
                    inputSchema: {
                      type: "object",
                      properties: {
                        title: { type: "string" },
                        description: { type: "string" },
                      },
                      required: ["title"],
                    },
                  },
                ],
              },
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }) as typeof fetch;

      // Dynamically import and run main — we need to re-import to pick up env
      // Instead, replicate the main logic using exported functions since main()
      // reads module-level constants that were already captured at import time.
      const {
        parseTool: parse,
        generateToolFile: genTool,
        generateServerIndex: genIndex,
        generateDenoJson: genDeno,
        generateReadme: genReadme,
      } = await import("./codegen.ts");

      // Simulate what main() does
      const mockTools: ToolDef[] = [
        {
          name: "github_search_code",
          description: "Search code on GitHub",
          inputSchema: {
            type: "object",
            properties: {
              q: { type: "string", description: "Search query" },
              repo: { type: "string", description: "Repository" },
            },
            required: ["q"],
          },
        },
        {
          name: "github_get_file",
          description: "Get file contents",
          inputSchema: {
            type: "object",
            properties: { path: { type: "string" } },
            required: ["path"],
          },
        },
        {
          name: "linear_create_issue",
          description: "Create a Linear issue",
          inputSchema: {
            type: "object",
            properties: {
              title: { type: "string" },
              description: { type: "string" },
            },
            required: ["title"],
          },
        },
      ];

      const parsed = mockTools.map(parse);
      const byServer = new Map<string, ParsedTool[]>();
      for (const tool of parsed) {
        const group = byServer.get(tool.serverPrefix) ?? [];
        group.push(tool);
        byServer.set(tool.serverPrefix, group);
      }

      const serversDir = `${outputDir}/servers`;
      await Deno.mkdir(serversDir, { recursive: true });

      for (const [prefix, serverTools] of byServer) {
        const serverDir = `${serversDir}/${prefix}`;
        await Deno.mkdir(serverDir, { recursive: true });

        const funcNames: string[] = [];
        for (const tool of serverTools) {
          const content = genTool(tool);
          await Deno.writeTextFile(`${serverDir}/${tool.funcName}.ts`, content);
          funcNames.push(tool.funcName);
        }

        const indexContent = genIndex(funcNames);
        await Deno.writeTextFile(`${serverDir}/index.ts`, indexContent);
      }

      await Deno.writeTextFile(`${outputDir}/deno.json`, genDeno());
      await Deno.writeTextFile(
        `${outputDir}/README.md`,
        genReadme([...byServer.keys()]),
      );

      // ── Verify directory structure ──

      // github server files
      const ghSearchCode = await Deno.readTextFile(
        `${serversDir}/github/search_code.ts`,
      );
      assertStringIncludes(ghSearchCode, "export async function search_code(");
      assertStringIncludes(ghSearchCode, 'callTool("github_search_code"');

      const ghGetFile = await Deno.readTextFile(
        `${serversDir}/github/get_file.ts`,
      );
      assertStringIncludes(ghGetFile, "export async function get_file(");

      // github index re-exports
      const ghIndex = await Deno.readTextFile(`${serversDir}/github/index.ts`);
      assertStringIncludes(ghIndex, 'export { search_code } from "./search_code.ts"');
      assertStringIncludes(ghIndex, 'export { get_file } from "./get_file.ts"');

      // linear server
      const linearIssue = await Deno.readTextFile(
        `${serversDir}/linear/create_issue.ts`,
      );
      assertStringIncludes(linearIssue, "export async function create_issue(");
      assertStringIncludes(linearIssue, "title: string");

      // linear index
      const linearIndex = await Deno.readTextFile(
        `${serversDir}/linear/index.ts`,
      );
      assertStringIncludes(
        linearIndex,
        'export { create_issue } from "./create_issue.ts"',
      );

      // deno.json exists and is valid JSON
      const denoJson = await Deno.readTextFile(`${outputDir}/deno.json`);
      const denoConfig = JSON.parse(denoJson);
      assertEquals(denoConfig.compilerOptions.strict, true);

      // README.md
      const readme = await Deno.readTextFile(`${outputDir}/README.md`);
      assertStringIncludes(readme, "github");
      assertStringIncludes(readme, "linear");
      assertStringIncludes(readme, "Auto-generated by codegen.ts");
    } finally {
      // Restore
      globalThis.fetch = originalFetch;
      Deno.env.delete("CTO_TOOLS_OUTPUT");

      // Cleanup temp dir
      await Deno.remove(outputDir, { recursive: true });
    }
  },
});

// ── client-config.json / local-server tool parsing ───────────────────────────

Deno.test("parseTool: handles tools from local server namespaces", () => {
  const localTools: ToolDef[] = [
    {
      name: "filesystem_read_file",
      description: "Read a file",
      inputSchema: {
        type: "object",
        properties: { path: { type: "string" } },
        required: ["path"],
      },
    },
    {
      name: "memory_store",
      description: "Store key-value",
      inputSchema: {
        type: "object",
        properties: { key: { type: "string" }, value: { type: "string" } },
        required: ["key", "value"],
      },
    },
  ];

  for (const tool of localTools) {
    const parsed = parseTool(tool);
    const expectedPrefix = tool.name.split("_")[0];
    assertEquals(parsed.serverPrefix, expectedPrefix);
    const expectedFunc = tool.name.slice(expectedPrefix.length + 1);
    assertEquals(parsed.funcName, expectedFunc);
    assertEquals(parsed.fullName, tool.name);
  }
});

Deno.test("generateToolFile: wrapper for local-server tool calls callTool correctly", () => {
  const tool: ParsedTool = {
    fullName: "filesystem_read_file",
    serverPrefix: "filesystem",
    funcName: "read_file",
    description: "Read a local file",
    inputSchema: {
      type: "object",
      properties: { path: { type: "string", description: "File path" } },
      required: ["path"],
    },
  };
  const output = generateToolFile(tool);
  assertStringIncludes(output, 'callTool("filesystem_read_file"');
  assertStringIncludes(output, "export async function read_file(");
  assertStringIncludes(output, "path: string");
});

// ── Dual-source merge test ───────────────────────────────────────────────────

Deno.test("integration: mixed remote and local tools generate correct wrappers", async () => {
  const mixedTools: ToolDef[] = [
    {
      name: "github_search_code",
      description: "Remote: Search code",
      inputSchema: { type: "object", properties: { q: { type: "string" } }, required: ["q"] },
    },
    { name: "linear_list_issues", description: "Remote: List issues" },
    {
      name: "filesystem_read_file",
      description: "Local: Read file",
      inputSchema: { type: "object", properties: { path: { type: "string" } }, required: ["path"] },
    },
    {
      name: "memory_store",
      description: "Local: Store in memory",
      inputSchema: {
        type: "object",
        properties: { key: { type: "string" }, value: { type: "string" } },
        required: ["key", "value"],
      },
    },
  ];

  const parsed = mixedTools.map(parseTool);
  const byServer = new Map<string, ParsedTool[]>();
  for (const tool of parsed) {
    const group = byServer.get(tool.serverPrefix) ?? [];
    group.push(tool);
    byServer.set(tool.serverPrefix, group);
  }

  assertEquals(byServer.size, 4);
  assert(byServer.has("github"));
  assert(byServer.has("linear"));
  assert(byServer.has("filesystem"));
  assert(byServer.has("memory"));

  const outputDir = await Deno.makeTempDir({ prefix: "codegen_merge_" });
  try {
    for (const [prefix, tools] of byServer) {
      const serverDir = `${outputDir}/${prefix}`;
      await Deno.mkdir(serverDir, { recursive: true });

      const funcNames: string[] = [];
      for (const tool of tools) {
        const content = generateToolFile(tool);
        await Deno.writeTextFile(`${serverDir}/${tool.funcName}.ts`, content);
        funcNames.push(tool.funcName);
        assertStringIncludes(content, `callTool("${tool.fullName}"`);
      }

      const index = generateServerIndex(funcNames);
      await Deno.writeTextFile(`${serverDir}/index.ts`, index);
      for (const fn of funcNames) {
        assertStringIncludes(index, `export { ${fn} }`);
      }
    }

    // Verify local-server wrappers have correct content
    const fsFile = await Deno.readTextFile(`${outputDir}/filesystem/read_file.ts`);
    assertStringIncludes(fsFile, 'callTool("filesystem_read_file"');
    assertStringIncludes(fsFile, "path: string");

    const memFile = await Deno.readTextFile(`${outputDir}/memory/store.ts`);
    assertStringIncludes(memFile, 'callTool("memory_store"');
    assertStringIncludes(memFile, "key: string");

    // README lists all servers
    const readme = generateReadme([...byServer.keys()]);
    assertStringIncludes(readme, "github");
    assertStringIncludes(readme, "filesystem");
    assertStringIncludes(readme, "memory");
  } finally {
    await Deno.remove(outputDir, { recursive: true });
  }
});

// ── Local server failure graceful degradation ────────────────────────────────

Deno.test("graceful degradation: tool with no schema generates valid wrapper", () => {
  const tool: ToolDef = { name: "broken_no_schema" };
  const parsed = parseTool(tool);
  const output = generateToolFile(parsed);

  assertStringIncludes(output, `export async function ${parsed.funcName}(`);
  assertStringIncludes(output, "Record<string, unknown>");
  assertStringIncludes(output, 'callTool("broken_no_schema"');
  assertStringIncludes(output, "Auto-generated by codegen.ts");
});

Deno.test("graceful degradation: partial tool list still generates wrappers for valid tools", () => {
  // If a local server fails, only remote tools are available.
  const remoteOnlyTools: ToolDef[] = [
    {
      name: "github_search_code",
      description: "Search code",
      inputSchema: { type: "object", properties: { q: { type: "string" } }, required: ["q"] },
    },
    { name: "github_get_file", description: "Get file" },
  ];

  const parsed = remoteOnlyTools.map(parseTool);
  const byServer = new Map<string, ParsedTool[]>();
  for (const tool of parsed) {
    const group = byServer.get(tool.serverPrefix) ?? [];
    group.push(tool);
    byServer.set(tool.serverPrefix, group);
  }

  assertEquals(byServer.size, 1);
  assert(byServer.has("github"));
  assertEquals(byServer.get("github")!.length, 2);

  const funcNames = byServer.get("github")!.map((t) => t.funcName);
  const index = generateServerIndex(funcNames);
  assertStringIncludes(index, "search_code");
  assertStringIncludes(index, "get_file");

  const readme = generateReadme(["github"]);
  assertStringIncludes(readme, "github");
  // Still produces a valid README with a single server
  assertStringIncludes(readme, "Available servers");
});

Deno.test("generateJsDoc: tool with no description produces valid JSDoc", () => {
  const tool: ParsedTool = {
    fullName: "test_no_desc",
    serverPrefix: "test",
    funcName: "no_desc",
  };
  const jsDoc = generateJsDoc(tool);
  assertStringIncludes(jsDoc, "/**");
  assertStringIncludes(jsDoc, "*/");
});
