import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { mkdir, readFile, rename, writeFile, appendFile, access } from "node:fs/promises";
import { constants } from "node:fs";
import { dirname, join } from "node:path";

export const MORGAN_SIDECAR_SERVICE = "morgan-agent-sidecar" as const;
export const MORGAN_SIDECAR_VERSION = "0.1.0" as const;
export const DEFAULT_MCP_PORT = 4000;
export const DEFAULT_HEALTH_PORT = 4001;

export interface MorganToolDescriptor {
  name: string;
  description: string;
  input_schema: Record<string, unknown>;
}

export interface MorganSidecarOptions {
  workspaceDir?: string;
  eventLog?: string;
  commandLog?: string;
  statusFile?: string;
  coderunId?: string;
  agentId?: string;
  sessionId?: string;
  providerMode?: string;
  mcpPort?: number;
  now?: () => Date;
}

export interface MorganStreamPaths {
  workspaceDir: string;
  eventLog: string;
  commandLog: string;
  statusFile: string;
}

export interface MorganEvent {
  seq: number;
  ts: string;
  source: "morgan-agent-sidecar";
  session_id: string;
  coderun_id: string;
  agent_id: string;
  type: string;
  payload: Record<string, unknown>;
}

export interface MorganCommandInput {
  source: string;
  session_id: string;
  type: string;
  payload?: Record<string, unknown>;
  now?: () => Date;
}

export interface MorganHealth {
  ok: boolean;
  service: typeof MORGAN_SIDECAR_SERVICE;
  version: typeof MORGAN_SIDECAR_VERSION;
  mcp: { port: number; ready: boolean };
  workspace: {
    root: string;
    event_log_writable: boolean;
    command_log_readable: boolean;
    status_file_writable: boolean;
  };
  providers: {
    mode: string;
    livekit: "configured" | "missing_optional";
    lemonslice: "configured" | "missing_optional";
    recall: "configured" | "missing_optional";
  };
}

interface MorganStatusSnapshot {
  session_id: string;
  coderun_id: string;
  agent_id: string;
  status: string;
  state: string;
  active_provider: string;
  fallback_used: boolean;
  surface: Record<string, unknown> | null;
  last_event_seq: number;
  last_error: string | null;
  updated_at: string;
}

const TOOL_NAMES = [
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
] as const;

export function listMorganTools(): MorganToolDescriptor[] {
  return TOOL_NAMES.map((name) => ({
    name,
    description: descriptionForTool(name),
    input_schema: { type: "object", additionalProperties: true },
  }));
}

function descriptionForTool(name: string): string {
  switch (name) {
    case "meet_join":
      return "Morgan Meet compatibility alias for starting a stub Morgan session";
    case "meet_leave":
      return "Morgan Meet compatibility alias for stopping the current Morgan session";
    case "meet_get_status":
      return "Morgan Meet compatibility alias for reading Morgan session status";
    case "meet_stream_audio":
      return "Controlled stub for future audio artifact ingestion";
    default:
      return `Morgan sidecar tool ${name}`;
  }
}

export async function createMorganSidecar(options: MorganSidecarOptions = {}): Promise<MorganSidecar> {
  const sidecar = new MorganSidecar(options);
  await sidecar.initialize();
  return sidecar;
}

export class MorganSidecar {
  readonly paths: MorganStreamPaths;
  private readonly coderunId: string;
  private readonly agentId: string;
  private readonly sessionId: string;
  private readonly providerMode: string;
  private readonly mcpPort: number;
  private readonly now: () => Date;
  private seq = 0;
  private status: MorganStatusSnapshot;

  constructor(options: MorganSidecarOptions = {}) {
    const workspaceDir = options.workspaceDir ?? process.env.MORGAN_STREAM_DIR ?? "/workspace";
    this.paths = {
      workspaceDir,
      eventLog: options.eventLog ?? process.env.MORGAN_EVENT_LOG ?? join(workspaceDir, "morgan-events.jsonl"),
      commandLog: options.commandLog ?? process.env.MORGAN_COMMAND_LOG ?? join(workspaceDir, "morgan-commands.jsonl"),
      statusFile: options.statusFile ?? process.env.MORGAN_STATUS_FILE ?? join(workspaceDir, "morgan-status.json"),
    };
    this.coderunId = options.coderunId ?? process.env.CODERUN_ID ?? "local-coderun";
    this.agentId = options.agentId ?? process.env.MORGAN_AGENT_ID ?? "morgan";
    this.sessionId = options.sessionId ?? process.env.MORGAN_SESSION_ID ?? `morgan_${this.coderunId}_local`;
    this.providerMode = options.providerMode ?? process.env.MORGAN_PROVIDER_MODE ?? "stub";
    this.mcpPort = options.mcpPort ?? Number(process.env.MORGAN_MCP_PORT ?? DEFAULT_MCP_PORT);
    this.now = options.now ?? (() => new Date());
    this.status = this.createStatus("ready", "idle", null, 0, null);
  }

  async initialize(): Promise<void> {
    await mkdir(dirname(this.paths.eventLog), { recursive: true });
    await mkdir(dirname(this.paths.commandLog), { recursive: true });
    await mkdir(dirname(this.paths.statusFile), { recursive: true });
    await appendFile(this.paths.eventLog, "");
    await appendFile(this.paths.commandLog, "");
    await this.writeStatus(this.status);
    await this.appendEvent("sidecar_ready", {
      tools: TOOL_NAMES,
      workspace: {
        events: this.paths.eventLog,
        commands: this.paths.commandLog,
        status: this.paths.statusFile,
      },
    });
  }

  async health(): Promise<MorganHealth> {
    const [eventWritable, commandReadable, statusWritable] = await Promise.all([
      canAccess(this.paths.eventLog, constants.W_OK),
      canAccess(this.paths.commandLog, constants.R_OK),
      canAccess(this.paths.statusFile, constants.W_OK),
    ]);
    return {
      ok: eventWritable && commandReadable && statusWritable,
      service: MORGAN_SIDECAR_SERVICE,
      version: MORGAN_SIDECAR_VERSION,
      mcp: { port: this.mcpPort, ready: true },
      workspace: {
        root: this.paths.workspaceDir,
        event_log_writable: eventWritable,
        command_log_readable: commandReadable,
        status_file_writable: statusWritable,
      },
      providers: {
        mode: this.providerMode,
        livekit: configured(process.env.LIVEKIT_URL),
        lemonslice: configured(process.env.LEMONSLICE_API_KEY),
        recall: configured(process.env.RECALL_API_KEY),
      },
    };
  }

  async callTool(name: string, input: Record<string, unknown> = {}): Promise<Record<string, unknown>> {
    switch (name) {
      case "morgan_session_start":
      case "meet_join":
        return this.startSession(input);
      case "morgan_session_stop":
      case "meet_leave":
        return this.stopSession(input);
      case "morgan_session_status":
      case "meet_get_status":
        return this.readStatus();
      case "morgan_say":
        return this.say(input);
      case "morgan_set_state":
        return this.setState(input);
      case "morgan_events_tail":
        return { events: await tailMorganEvents(this.paths.eventLog, Number(input.limit ?? 20)) };
      case "meet_stream_audio":
        return {
          status: "not_implemented",
          provider_mode: this.providerMode,
          reason: "audio streaming is intentionally stubbed until provider integration is wired",
        };
      default:
        throw new Error(`Unknown Morgan tool: ${name}`);
    }
  }

  private async startSession(input: Record<string, unknown>): Promise<Record<string, unknown>> {
    const surface = recordOrNull(input.surface);
    this.status = this.createStatus("starting", "idle", surface, this.seq, null);
    await this.appendEvent("session_started", {
      surface,
      mode: stringOrDefault(input.mode, "symbolic"),
      provider_mode: this.providerMode,
    });
    this.status = this.createStatus("starting", "idle", surface, this.seq, null);
    await this.writeStatus(this.status);
    return {
      session_id: this.sessionId,
      status: "starting",
      surface,
      active_provider: this.providerMode,
      workspace: {
        events: this.paths.eventLog,
        commands: this.paths.commandLog,
        status: this.paths.statusFile,
      },
    };
  }

  private async stopSession(input: Record<string, unknown>): Promise<Record<string, unknown>> {
    await this.appendEvent("session_stopped", { reason: stringOrDefault(input.reason, "requested") });
    this.status = this.createStatus("stopped", "idle", this.status.surface, this.seq, null);
    await this.writeStatus(this.status);
    return { session_id: this.sessionId, status: "stopped" };
  }

  private async say(input: Record<string, unknown>): Promise<Record<string, unknown>> {
    const text = stringOrDefault(input.text, "");
    await this.appendEvent("turn_queued", { text, interrupt: input.interrupt === true });
    this.status = this.createStatus(this.status.status, "idle", this.status.surface, this.seq, null);
    await this.writeStatus(this.status);
    return { session_id: this.sessionId, status: "queued", text_length: text.length };
  }

  private async setState(input: Record<string, unknown>): Promise<Record<string, unknown>> {
    const state = stringOrDefault(input.state, "idle");
    await this.appendEvent("state", { state });
    this.status = this.createStatus(this.status.status, state, this.status.surface, this.seq, null);
    await this.writeStatus(this.status);
    return { session_id: this.sessionId, status: this.status.status, state };
  }

  private async readStatus(): Promise<Record<string, unknown>> {
    return JSON.parse(await readFile(this.paths.statusFile, "utf8")) as Record<string, unknown>;
  }

  private async appendEvent(type: string, payload: Record<string, unknown>): Promise<MorganEvent> {
    const event: MorganEvent = {
      seq: ++this.seq,
      ts: this.now().toISOString(),
      source: MORGAN_SIDECAR_SERVICE,
      session_id: this.sessionId,
      coderun_id: this.coderunId,
      agent_id: this.agentId,
      type,
      payload,
    };
    await appendFile(this.paths.eventLog, `${JSON.stringify(event)}\n`);
    return event;
  }

  private createStatus(
    status: string,
    state: string,
    surface: Record<string, unknown> | null,
    lastEventSeq: number,
    lastError: string | null,
  ): MorganStatusSnapshot {
    return {
      session_id: this.sessionId,
      coderun_id: this.coderunId,
      agent_id: this.agentId,
      status,
      state,
      active_provider: this.providerMode,
      fallback_used: this.providerMode === "stub",
      surface,
      last_event_seq: lastEventSeq,
      last_error: lastError,
      updated_at: this.now().toISOString(),
    };
  }

  private async writeStatus(status: MorganStatusSnapshot): Promise<void> {
    const tempPath = `${this.paths.statusFile}.tmp`;
    await writeFile(tempPath, `${JSON.stringify(status, null, 2)}\n`);
    await rename(tempPath, this.paths.statusFile);
  }
}

export async function appendMorganCommand(commandLog: string, input: MorganCommandInput): Promise<void> {
  await mkdir(dirname(commandLog), { recursive: true });
  let seq = 1;
  try {
    const existing = await readFile(commandLog, "utf8");
    seq = existing.split("\n").filter(Boolean).length + 1;
  } catch {
    seq = 1;
  }
  const command = {
    seq,
    ts: (input.now ?? (() => new Date()))().toISOString(),
    source: input.source,
    session_id: input.session_id,
    command_id: `cmd_${String(seq).padStart(6, "0")}`,
    type: input.type,
    payload: input.payload ?? {},
  };
  await appendFile(commandLog, `${JSON.stringify(command)}\n`);
}

export async function tailMorganEvents(eventLog: string, limit = 20): Promise<MorganEvent[]> {
  const text = await readFile(eventLog, "utf8");
  return text
    .split("\n")
    .filter(Boolean)
    .slice(-Math.max(0, limit))
    .map((line) => JSON.parse(line) as MorganEvent);
}

async function canAccess(path: string, mode: number): Promise<boolean> {
  try {
    await access(path, mode);
    return true;
  } catch {
    return false;
  }
}

function configured(value: string | undefined): "configured" | "missing_optional" {
  return value && value.length > 0 ? "configured" : "missing_optional";
}

function stringOrDefault(value: unknown, fallback: string): string {
  return typeof value === "string" ? value : fallback;
}

function recordOrNull(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
}

async function readRequestBody(request: IncomingMessage): Promise<Record<string, unknown>> {
  const chunks: Buffer[] = [];
  for await (const chunk of request) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  if (chunks.length === 0) {
    return {};
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8")) as Record<string, unknown>;
}

function writeJson(response: ServerResponse, statusCode: number, body: unknown): void {
  response.writeHead(statusCode, { "content-type": "application/json" });
  response.end(`${JSON.stringify(body)}\n`);
}

export async function startServer(
  sidecar?: MorganSidecar,
  port = Number(process.env.HTTP_PORT ?? DEFAULT_HEALTH_PORT),
) {
  const activeSidecar = sidecar ?? (await createMorganSidecar());
  const server = createServer(async (request, response) => {
    try {
      if (request.method === "GET" && request.url === "/healthz") {
        writeJson(response, 200, await activeSidecar.health());
        return;
      }
      if (request.method === "GET" && request.url === "/mcp/tools") {
        writeJson(response, 200, { tools: listMorganTools() });
        return;
      }
      if (request.method === "POST" && request.url?.startsWith("/mcp/call/")) {
        const toolName = decodeURIComponent(request.url.slice("/mcp/call/".length));
        const input = await readRequestBody(request);
        writeJson(response, 200, await activeSidecar.callTool(toolName, input));
        return;
      }
      writeJson(response, 404, { error: "not_found" });
    } catch (error) {
      writeJson(response, 500, { error: error instanceof Error ? error.message : "unknown" });
    }
  });
  await new Promise<void>((resolve) => server.listen(port, resolve));
  return server;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  startServer().then((server) => {
    const address = server.address();
    console.log(`[morgan-agent-sidecar] listening`, address);
  });
}
