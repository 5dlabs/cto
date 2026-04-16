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

// ─── Client Config ───────────────────────────────────────────────────────────

interface LocalServerConfig {
  command: string;
  args: string[];
  tools: string[];
  workingDirectory?: string;
  env?: Record<string, string>;
}

interface ClientConfig {
  remoteTools: string[];
  localServers: Record<string, LocalServerConfig>;
}

function loadClientConfig(): ClientConfig | null {
  const configPath =
    Deno.env.get("CLIENT_CONFIG_PATH") ?? "/task-files/client-config.json";
  try {
    const text = Deno.readTextFileSync(configPath);
    return JSON.parse(text) as ClientConfig;
  } catch {
    return null;
  }
}

const clientConfig = loadClientConfig();

// ─── Config ──────────────────────────────────────────────────────────────────

const envVar = (key: string, fallback: string): string =>
  Deno.env.get(key) ?? fallback;

const TOOLS_SERVER_URL = envVar(
  "TOOLS_SERVER_URL",
  "http://cto-tools.cto.svc.cluster.local:3000/mcp",
);
const LOCAL_TOOLS_URL = envVar("LOCAL_TOOLS_URL", "http://localhost:3001/mcp");

const localPrefixes: Set<string> = new Set(
  envVar("LOCAL_TOOLS", "filesystem,memory")
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

/** Strip the server prefix from a namespaced tool name (e.g. "memory_search_nodes" → "search_nodes"). */
function stripPrefix(toolName: string, server: string): string {
  return toolName.startsWith(server + "_") ? toolName.slice(server.length + 1) : toolName;
}

/**
 * Find which local server (from clientConfig) owns a given tool name.
 * Returns the server key or null if the tool isn't local.
 */
function localServerFor(toolName: string): string | null {
  if (!clientConfig) return null;
  for (const [key, cfg] of Object.entries(clientConfig.localServers)) {
    if (cfg.tools?.includes(toolName)) return key;
  }
  // Fallback: check if the server prefix matches a localServers key
  const prefix = serverPrefix(toolName);
  if (clientConfig.localServers[prefix]) return prefix;
  return null;
}

type TransportRoute =
  | { kind: "stdio"; server: string }
  | { kind: "http"; url: string };

/**
 * Route a tool call to local stdio, local HTTP sidecar, or remote MCP server.
 */
function transportFor(toolName: string): TransportRoute {
  const server = localServerFor(toolName);
  if (server) return { kind: "stdio", server };
  // Legacy env-var routing
  if (localPrefixes.has(serverPrefix(toolName))) {
    return { kind: "http", url: LOCAL_TOOLS_URL };
  }
  return { kind: "http", url: TOOLS_SERVER_URL };
}

/** Route a tool call to the local sidecar or the remote MCP server (HTTP-only legacy). */
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

// ─── Stdio Transport ────────────────────────────────────────────────────────

class StdioTransport {
  #process: Deno.ChildProcess | null = null;
  #stdin: WritableStreamDefaultWriter<Uint8Array> | null = null;
  #reader: ReadableStreamDefaultReader<string> | null = null;
  #buffer = "";
  #initialized = false;
  #initPromise: Promise<void> | null = null;
  #msgId = 0;
  readonly #serverName: string;
  readonly #config: LocalServerConfig;

  constructor(serverName: string, config: LocalServerConfig) {
    this.#serverName = serverName;
    this.#config = config;
  }

  /** Spawn the child process if not already running. */
  #spawn(): void {
    if (this.#process) return;

    const cwd = this.#config.workingDirectory ?? Deno.cwd();

    const childEnv: Record<string, string> = {};
    // Inherit parent PATH
    const parentPath = Deno.env.get("PATH");
    if (parentPath) childEnv["PATH"] = parentPath;
    // Match Rust client: set WORKING_DIRECTORY and PROJECT_DIR
    childEnv["WORKING_DIRECTORY"] = cwd;
    childEnv["PROJECT_DIR"] = cwd;
    // Merge server-specific env
    if (this.#config.env) {
      Object.assign(childEnv, this.#config.env);
    }

    const cmd = new Deno.Command(this.#config.command, {
      args: this.#config.args,
      cwd,
      stdin: "piped",
      stdout: "piped",
      stderr: "piped",
      env: childEnv,
    });

    this.#process = cmd.spawn();

    // Drain stderr so the child process doesn't block
    this.#process.stderr.pipeTo(
      new WritableStream({ write(_chunk) { /* discard */ } }),
    ).catch(() => { /* ignore */ });

    this.#stdin = this.#process.stdin.getWriter();

    // Build a line reader from stdout
    const decoder = new TextDecoderStream();
    const readable = this.#process.stdout.pipeThrough(decoder);
    this.#reader = readable.getReader();
  }

  /** Read one newline-delimited JSON line from stdout. */
  async #readLine(): Promise<string> {
    for (;;) {
      const nlIdx = this.#buffer.indexOf("\n");
      if (nlIdx !== -1) {
        const line = this.#buffer.slice(0, nlIdx);
        this.#buffer = this.#buffer.slice(nlIdx + 1);
        if (line.trim().length > 0) return line;
        continue;
      }
      const { value, done } = await this.#reader!.read();
      if (done) {
        throw new ToolError(
          ErrorCodes.SERVER_ERROR,
          `Stdio stream closed for server '${this.#serverName}'`,
        );
      }
      this.#buffer += value;
    }
  }

  /** Read lines until we find a JSON-RPC response matching the given id. */
  async #readResponse<T>(id: number): Promise<JsonRpcResponse<T>> {
    for (;;) {
      const line = await this.#readLine();
      let msg: Record<string, unknown>;
      try {
        msg = JSON.parse(line);
      } catch {
        // Skip non-JSON lines (e.g. logging output)
        continue;
      }
      // Skip notifications (no id field)
      if (!("id" in msg)) continue;
      if (msg.id === id) return msg as unknown as JsonRpcResponse<T>;
      // Mismatched id — skip (could be out-of-order notification)
    }
  }

  /** Write a JSON-RPC message to stdin (newline-delimited). */
  async #write(msg: Record<string, unknown>): Promise<void> {
    const line = JSON.stringify(msg) + "\n";
    const encoded = new TextEncoder().encode(line);
    await this.#stdin!.write(encoded);
  }

  /** Perform the MCP initialize handshake (once). */
  async #initialize(): Promise<void> {
    if (this.#initialized) return;
    if (this.#initPromise) {
      await this.#initPromise;
      return;
    }
    this.#initPromise = this.#doInitialize();
    try {
      await this.#initPromise;
    } catch (e) {
      this.#initPromise = null;
      throw e;
    }
  }

  async #doInitialize(): Promise<void> {
    this.#spawn();

    // Step 1: Send initialize request
    const initId = ++this.#msgId;
    await this.#write({
      jsonrpc: "2.0",
      id: initId,
      method: "initialize",
      params: {
        protocolVersion: "2024-11-05",
        capabilities: { tools: {} },
        clientInfo: { name: "cto-tools", version: "1.0.0" },
      },
    });

    // Step 2: Read initialize response
    const initResp = await this.#readResponse(initId);
    if (initResp.error) {
      throw new ToolError(
        initResp.error.code,
        `Initialize failed for '${this.#serverName}': ${initResp.error.message}`,
        initResp.error.data,
      );
    }

    // Step 3: Send initialized notification (no id, no response expected)
    await this.#write({
      jsonrpc: "2.0",
      method: "notifications/initialized",
    });

    this.#initialized = true;
  }

  /** Send a JSON-RPC request and return the result. */
  async request<T>(
    method: string,
    params?: Record<string, unknown>,
  ): Promise<T> {
    await this.#initialize();

    const id = ++this.#msgId;
    const msg: Record<string, unknown> = { jsonrpc: "2.0", id, method };
    if (params !== undefined) msg.params = params;
    await this.#write(msg);

    const resp = await this.#readResponse<T>(id);
    if (resp.error) {
      throw new ToolError(resp.error.code, resp.error.message, resp.error.data);
    }
    return resp.result as T;
  }

  /** Terminate the child process. */
  shutdown(): void {
    try {
      this.#process?.kill();
    } catch { /* already dead */ }
    this.#process = null;
    this.#stdin = null;
    this.#reader = null;
    this.#buffer = "";
    this.#initialized = false;
    this.#initPromise = null;
  }
}

// Per-server transport cache
const stdioTransports = new Map<string, StdioTransport>();

function getStdioTransport(serverName: string): StdioTransport {
  let t = stdioTransports.get(serverName);
  if (t) return t;

  const cfg = clientConfig?.localServers[serverName];
  if (!cfg) {
    throw new ToolError(
      ErrorCodes.SERVER_ERROR,
      `No local server config for '${serverName}'`,
    );
  }
  t = new StdioTransport(serverName, cfg);
  stdioTransports.set(serverName, t);
  return t;
}

// Clean up child processes on module unload
globalThis.addEventListener("unload", () => {
  for (const t of stdioTransports.values()) t.shutdown();
  stdioTransports.clear();
});

// ─── Stdio JSON-RPC transport ───────────────────────────────────────────────

async function rpcStdio<T = unknown>(
  serverName: string,
  method: string,
  params?: Record<string, unknown>,
): Promise<T> {
  const transport = getStdioTransport(serverName);
  return transport.request<T>(method, params);
}

// ─── HTTP JSON-RPC transport ────────────────────────────────────────────────

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
 * Includes tools from both remote and local stdio servers.
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

  // Fetch tools from each local stdio server
  if (clientConfig) {
    for (const serverName of Object.keys(clientConfig.localServers)) {
      try {
        const result = await rpcStdio<ToolsListResult>(
          serverName,
          "tools/list",
        );
        for (const tool of result.tools) {
          const prefixedName = `${serverName}_${tool.name}`;
          (grouped[serverName] ??= []).push({
            name: prefixedName,
            description: tool.description,
            inputSchema: tool.inputSchema,
          });
        }
      } catch {
        // If a local server fails to respond, skip it gracefully
      }
    }
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
  const route = transportFor(name);

  const { tools } = route.kind === "stdio"
    ? await rpcStdio<ToolsListResult>(route.server, "tools/list")
    : await rpc<ToolsListResult>(route.url, "tools/list");

  const lookupName = route.kind === "stdio"
    ? stripPrefix(name, route.server)
    : name;
  const tool = tools.find((t) => t.name === lookupName);
  if (!tool) {
    throw new ToolError(
      ErrorCodes.TOOL_NOT_FOUND,
      `Tool not found: ${name}`,
    );
  }
  return {
    name,
    description: tool.description,
    inputSchema: tool.inputSchema,
  };
}

/**
 * Invoke an MCP tool by name and return the parsed result.
 *
 * The call is routed to a local stdio server (if configured in client-config),
 * the local HTTP sidecar, or the remote MCP server based on routing rules.
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
  const route = transportFor(name);

  const result = route.kind === "stdio"
    ? await rpcStdio<ToolCallResult>(
        route.server,
        "tools/call",
        { name: stripPrefix(name, route.server), arguments: args },
      )
    : await rpc<ToolCallResult>(
        route.url,
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
