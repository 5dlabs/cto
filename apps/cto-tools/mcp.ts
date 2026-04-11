// mcp.ts — MCP tool-calling runtime for CTO agent SDK (Deno-compatible)
// Lets AI agents call MCP tools via code execution instead of native tool calls.

// ─── Types ───────────────────────────────────────────────────────────────────

export interface ToolInfo {
  name: string;
  description?: string;
  inputSchema?: Record<string, unknown>;
}

export interface ToolsByServer {
  [serverPrefix: string]: ToolInfo[];
}

interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: number;
  method: string;
  params?: Record<string, unknown>;
}

interface JsonRpcResponse<T = unknown> {
  jsonrpc: "2.0";
  id: number;
  result?: T;
  error?: { code: number; message: string; data?: unknown };
}

interface ToolsListResult {
  tools: Array<{ name: string; description?: string; inputSchema?: Record<string, unknown> }>;
}

interface ToolCallResult {
  content: Array<{ type: string; text: string }>;
}

// ─── Errors ──────────────────────────────────────────────────────────────────

/** Structured error from an MCP tool call or the JSON-RPC transport. */
export class ToolError extends Error {
  readonly code: number;
  readonly data?: unknown;

  constructor(code: number, message: string, data?: unknown) {
    super(message);
    this.name = "ToolError";
    this.code = code;
    this.data = data;
  }
}

export const ErrorCodes = {
  TOOL_NOT_FOUND: -32601,
  POLICY_DENIED: -32403,
  SERVER_ERROR: -32000,
} as const;

// ─── Config ──────────────────────────────────────────────────────────────────

const env = (key: string, fallback: string): string =>
  Deno.env.get(key) ?? fallback;

const TOOLS_SERVER_URL = env(
  "TOOLS_SERVER_URL",
  "http://cto-tools.cto.svc.cluster.local:3000/mcp",
);
const LOCAL_TOOLS_URL = env("LOCAL_TOOLS_URL", "http://localhost:3001/mcp");

const localPrefixes: Set<string> = new Set(
  env("LOCAL_TOOLS", "filesystem,memory")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean),
);

const agentId = Deno.env.get("CTO_AGENT_ID") ?? "";
const agentPrewarm = Deno.env.get("CTO_AGENT_PREWARM") ?? "";

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Extract the server prefix from a tool name (`github_search_code` → `github`). */
function serverPrefix(toolName: string): string {
  const idx = toolName.indexOf("_");
  return idx > 0 ? toolName.slice(0, idx) : toolName;
}

/** Route a tool call to the local sidecar or the remote MCP server. */
function endpointFor(toolName: string): string {
  return localPrefixes.has(serverPrefix(toolName))
    ? LOCAL_TOOLS_URL
    : TOOLS_SERVER_URL;
}

let rpcId = 0;

const RETRY_DELAYS_MS = [1_000, 2_000, 4_000];
const MAX_RETRIES = RETRY_DELAYS_MS.length;

function isRetryable(err: unknown): boolean {
  if (err instanceof Response) return err.status === 503;
  if (err instanceof TypeError) return true; // fetch network / connection refused
  return false;
}

async function sleep(ms: number): Promise<void> {
  await new Promise((r) => setTimeout(r, ms));
}

// ─── JSON-RPC transport ──────────────────────────────────────────────────────

async function rpc<T = unknown>(
  url: string,
  method: string,
  params?: Record<string, unknown>,
): Promise<T> {
  const body: JsonRpcRequest = {
    jsonrpc: "2.0",
    id: ++rpcId,
    method,
    ...(params !== undefined && { params }),
  };

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (agentId) headers["X-Agent-Id"] = agentId;
  if (agentPrewarm) headers["X-Agent-Prewarm"] = agentPrewarm;

  let lastError: unknown;

  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    try {
      const res = await fetch(url, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
      });

      if (res.status === 503) {
        lastError = res;
        if (attempt < MAX_RETRIES) {
          await sleep(RETRY_DELAYS_MS[attempt]);
          continue;
        }
        throw new ToolError(
          ErrorCodes.SERVER_ERROR,
          `MCP server returned 503 after ${MAX_RETRIES + 1} attempts`,
        );
      }

      const json: JsonRpcResponse<T> = await res.json();

      if (json.error) {
        throw new ToolError(json.error.code, json.error.message, json.error.data);
      }

      return json.result as T;
    } catch (err) {
      if (err instanceof ToolError) throw err;

      lastError = err;
      if (attempt < MAX_RETRIES && isRetryable(err)) {
        await sleep(RETRY_DELAYS_MS[attempt]);
        continue;
      }

      if (err instanceof Error) {
        throw new ToolError(ErrorCodes.SERVER_ERROR, err.message);
      }
      throw new ToolError(ErrorCodes.SERVER_ERROR, String(err));
    }
  }

  // Unreachable, but satisfies the type checker.
  throw new ToolError(
    ErrorCodes.SERVER_ERROR,
    `RPC failed after retries: ${lastError}`,
  );
}

// ─── Public API ──────────────────────────────────────────────────────────────

/**
 * List all tools available on the MCP server, grouped by server prefix.
 *
 * @returns An object keyed by server prefix (e.g. `github`, `linear`) whose
 *   values are arrays of `ToolInfo`.
 */
export async function listTools(): Promise<ToolsByServer> {
  const { tools } = await rpc<ToolsListResult>(
    TOOLS_SERVER_URL,
    "tools/list",
  );

  const grouped: ToolsByServer = {};
  for (const tool of tools) {
    const prefix = serverPrefix(tool.name);
    (grouped[prefix] ??= []).push({
      name: tool.name,
      description: tool.description,
      inputSchema: tool.inputSchema,
    });
  }
  return grouped;
}

/**
 * Retrieve the schema and description for a single tool by its full name.
 *
 * @param name Fully-qualified tool name (e.g. `github_search_code`).
 * @throws {ToolError} with code `-32601` if the tool is not found.
 */
export async function describeTool(name: string): Promise<ToolInfo> {
  const { tools } = await rpc<ToolsListResult>(
    endpointFor(name),
    "tools/list",
  );

  const tool = tools.find((t) => t.name === name);
  if (!tool) {
    throw new ToolError(
      ErrorCodes.TOOL_NOT_FOUND,
      `Tool not found: ${name}`,
    );
  }
  return {
    name: tool.name,
    description: tool.description,
    inputSchema: tool.inputSchema,
  };
}

/**
 * Invoke an MCP tool by name and return the parsed result.
 *
 * The call is routed to either the local sidecar or the remote MCP server
 * based on the tool's server prefix and the `LOCAL_TOOLS` configuration.
 *
 * @typeParam T The expected shape of the first text content item, parsed as JSON.
 * @param name Fully-qualified tool name.
 * @param args Tool arguments matching the tool's input schema.
 * @returns The parsed JSON value from the first `text` content block.
 */
export async function callTool<T = unknown>(
  name: string,
  args: Record<string, unknown>,
): Promise<T> {
  const result = await rpc<ToolCallResult>(
    endpointFor(name),
    "tools/call",
    { name, arguments: args },
  );

  const text = result.content?.find((c) => c.type === "text")?.text;
  if (text === undefined) {
    throw new ToolError(
      ErrorCodes.SERVER_ERROR,
      `Tool ${name} returned no text content`,
    );
  }

  try {
    return JSON.parse(text) as T;
  } catch {
    // If the response isn't JSON, return the raw string cast to T.
    return text as unknown as T;
  }
}

/**
 * Request an elevated capability for a tool the agent does not currently have.
 *
 * Sends a `tools_request_capability` call through the MCP server. On success
 * the response contains the granted tool's schema; on deny a `ToolError` with
 * code `-32403` is thrown by the RPC layer.
 *
 * @param toolName The tool the agent wants access to.
 * @param reason   Human-readable justification for the escalation.
 * @returns The granted tool's info (name, description, schema).
 */
export async function escalate(
  toolName: string,
  reason: string,
): Promise<ToolInfo> {
  const result = await callTool<ToolInfo>(
    "tools_request_capability",
    { tool_name: toolName, reason },
  );
  return result;
}
