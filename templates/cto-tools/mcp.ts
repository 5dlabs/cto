/**
 * CTO Tools MCP Runtime — lightweight typed bridge for agents running in Deno.
 *
 * Agents import `callTool`, `listTools`, or `describeTool` and the runtime
 * handles JSON-RPC framing, connection to the tools HTTP proxy, and error
 * propagation.
 *
 * Usage:
 *   import { callTool, listTools } from "/.cto-tools/mcp.ts";
 *   const result = await callTool("github_search_code", { q: "auth" });
 *
 * Environment:
 *   TOOLS_SERVER_URL — base URL of the cto-tools HTTP proxy
 *                      (default: http://cto-tools.cto.svc.cluster.local:3000/mcp)
 *   CTO_AGENT_ID     — agent identity for session tracking (default: "unknown")
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface ToolSchema {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

export interface ToolResult {
  content: Array<{ type: string; text: string }>;
  isError?: boolean;
  structuredContent?: Record<string, unknown>;
}

interface JsonRpcResponse {
  jsonrpc: string;
  id: number;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const TOOLS_URL =
  Deno.env.get("TOOLS_SERVER_URL") ??
  "http://cto-tools.cto.svc.cluster.local:3000/mcp";

const AGENT_ID = Deno.env.get("CTO_AGENT_ID") ?? "unknown";

let _nextId = 1;

// ---------------------------------------------------------------------------
// JSON-RPC transport
// ---------------------------------------------------------------------------

async function rpc(method: string, params?: Record<string, unknown>): Promise<unknown> {
  const body = JSON.stringify({
    jsonrpc: "2.0",
    id: _nextId++,
    method,
    ...(params !== undefined && { params }),
  });

  const res = await fetch(TOOLS_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Agent-Id": AGENT_ID,
    },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`tools server returned ${res.status}: ${text}`);
  }

  const json: JsonRpcResponse = await res.json();

  if (json.error) {
    throw new Error(
      `JSON-RPC error ${json.error.code}: ${json.error.message}`
    );
  }

  return json.result;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * List all available tools from the catalog.
 */
export async function listTools(): Promise<ToolSchema[]> {
  const result = (await rpc("tools/list")) as { tools: ToolSchema[] };
  return result.tools;
}

/**
 * Get the schema for a single tool by name.
 * Returns `undefined` if the tool is not found.
 */
export async function describeTool(
  name: string,
): Promise<ToolSchema | undefined> {
  const tools = await listTools();
  return tools.find((t) => t.name === name);
}

/**
 * Call an MCP tool by its fully-prefixed name.
 *
 * @param name - Tool name (e.g. "github_search_code")
 * @param args - Arguments matching the tool's input schema
 * @returns The tool's response content
 * @throws On JSON-RPC errors or HTTP failures
 */
export async function callTool(
  name: string,
  args: Record<string, unknown> = {},
): Promise<ToolResult> {
  const result = (await rpc("tools/call", {
    name,
    arguments: args,
  })) as ToolResult;

  if (result.isError) {
    const msg = result.content?.[0]?.text ?? "unknown tool error";
    throw new Error(`Tool ${name} returned error: ${msg}`);
  }

  return result;
}

/**
 * Request access to a tool outside the prewarm set.
 *
 * @param toolName - Fully-prefixed tool name
 * @param reason - Human-readable reason for the request
 * @returns The escalation response (grant or deny)
 */
export async function escalate(
  toolName: string,
  reason: string,
): Promise<ToolResult> {
  return callTool("tools_request_capability", {
    tool_name: toolName,
    reason,
  });
}
