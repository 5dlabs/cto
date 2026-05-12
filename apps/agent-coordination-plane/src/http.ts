import http, { type IncomingMessage, type Server, type ServerResponse } from "node:http";
import { URL } from "node:url";
import type { AgentAddress, AgentEnvelope, AgentGroup, AgentIdentity, AgentMessage, CreateEnvelopeOptions } from "./types.js";
import {
  InMemoryCoordinationPlane,
  type DeliveryResult,
  type InboxMessage,
  type RegisteredAgent,
} from "./store.js";

export interface CoordinationHttpServiceOptions {
  plane?: InMemoryCoordinationPlane;
  sharedToken?: string;
  maxBodyBytes?: number;
  now?: () => Date | string | number;
}

interface ErrorBody {
  error: {
    code: string;
    message: string;
  };
}

type JsonBody = Record<string, unknown> | unknown[];

const DEFAULT_MAX_BODY_BYTES = 256 * 1024;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function optionalString(value: unknown, field: string): string | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (!isString(value)) {
    throw new Error(`${field} must be a non-empty string when present`);
  }
  return value;
}

function optionalNumber(value: unknown, field: string): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${field} must be a finite number when present`);
  }
  return value;
}

function optionalStringMap(value: unknown, field: string): Record<string, string> | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (!isRecord(value)) {
    throw new Error(`${field} must be an object when present`);
  }
  const result: Record<string, string> = {};
  for (const [key, entry] of Object.entries(value)) {
    if (typeof entry !== "string") {
      throw new Error(`${field}.${key} must be a string`);
    }
    result[key] = entry;
  }
  return result;
}

function parseAgentIdentity(value: unknown, field = "identity"): AgentIdentity {
  if (!isRecord(value)) {
    throw new Error(`${field} must be an object`);
  }
  if (!isString(value.agentId)) {
    throw new Error(`${field}.agentId must be a non-empty string`);
  }
  if (!isString(value.role)) {
    throw new Error(`${field}.role must be a non-empty string`);
  }
  if (!isString(value.runtime)) {
    throw new Error(`${field}.runtime must be a non-empty string`);
  }
  return {
    agentId: value.agentId,
    role: value.role,
    runtime: value.runtime as AgentIdentity["runtime"],
    projectId: optionalString(value.projectId, `${field}.projectId`),
    taskId: optionalString(value.taskId, `${field}.taskId`),
    sessionId: optionalString(value.sessionId, `${field}.sessionId`),
    podName: optionalString(value.podName, `${field}.podName`),
    model: optionalString(value.model, `${field}.model`),
    metadata: optionalStringMap(value.metadata, `${field}.metadata`),
  };
}

function parseAgentGroup(value: unknown): AgentGroup {
  if (!isRecord(value)) {
    throw new Error("group must be an object");
  }
  if (!isString(value.groupId)) {
    throw new Error("group.groupId must be a non-empty string");
  }
  if (!isString(value.name)) {
    throw new Error("group.name must be a non-empty string");
  }
  if (!Array.isArray(value.members)) {
    throw new Error("group.members must be an array");
  }
  return {
    groupId: value.groupId,
    name: value.name,
    members: value.members.map((member, index) => parseAgentIdentity(member, `group.members.${index}`)),
    projectId: optionalString(value.projectId, "group.projectId"),
    taskId: optionalString(value.taskId, "group.taskId"),
    role: optionalString(value.role, "group.role"),
    metadata: optionalStringMap(value.metadata, "group.metadata"),
  };
}

function parseAddress(value: unknown): AgentAddress {
  if (!isRecord(value)) {
    throw new Error("address must be an object");
  }
  switch (value.kind) {
    case "agent":
      if (!isString(value.agentId)) {
        throw new Error("address.agentId must be a non-empty string");
      }
      return {
        kind: "agent",
        agentId: value.agentId,
        projectId: optionalString(value.projectId, "address.projectId"),
        taskId: optionalString(value.taskId, "address.taskId"),
      };
    case "role":
      if (!isString(value.role)) {
        throw new Error("address.role must be a non-empty string");
      }
      return {
        kind: "role",
        role: value.role,
        projectId: optionalString(value.projectId, "address.projectId"),
        taskId: optionalString(value.taskId, "address.taskId"),
      };
    case "task":
      if (!isString(value.projectId) || !isString(value.taskId)) {
        throw new Error("address.projectId and address.taskId must be non-empty strings");
      }
      return { kind: "task", projectId: value.projectId, taskId: value.taskId };
    case "project":
      if (!isString(value.projectId)) {
        throw new Error("address.projectId must be a non-empty string");
      }
      return { kind: "project", projectId: value.projectId };
    case "group":
      if (!isString(value.groupId)) {
        throw new Error("address.groupId must be a non-empty string");
      }
      return { kind: "group", groupId: value.groupId };
    case "broadcast":
      return { kind: "broadcast" };
    default:
      throw new Error("address.kind must be agent, role, task, project, group, or broadcast");
  }
}

function parseMessage(value: unknown): AgentMessage<unknown> {
  if (!isRecord(value)) {
    throw new Error("message must be an object");
  }
  if (!isString(value.messageId)) {
    throw new Error("message.messageId must be a non-empty string");
  }
  if (!isString(value.kind)) {
    throw new Error("message.kind must be a non-empty string");
  }
  if (!isString(value.createdAt)) {
    throw new Error("message.createdAt must be a non-empty string");
  }
  return {
    messageId: value.messageId,
    kind: value.kind as AgentMessage["kind"],
    from: parseAgentIdentity(value.from, "message.from"),
    to: parseAddress(value.to),
    body: value.body,
    createdAt: value.createdAt,
    priority: optionalString(value.priority, "message.priority") as AgentMessage["priority"],
    replyToMessageId: optionalString(value.replyToMessageId, "message.replyToMessageId"),
    correlationId: optionalString(value.correlationId, "message.correlationId"),
    metadata: optionalStringMap(value.metadata, "message.metadata"),
  };
}

function parseCreateEnvelopeOptions(value: Record<string, unknown>): CreateEnvelopeOptions {
  const trace = value.trace;
  if (trace !== undefined && !isRecord(trace)) {
    throw new Error("trace must be an object when present");
  }
  return {
    envelopeId: optionalString(value.envelopeId, "envelopeId"),
    subject: optionalString(value.subject, "subject"),
    sentAt: optionalString(value.sentAt, "sentAt"),
    replyTo: optionalString(value.replyTo, "replyTo"),
    ttlMs: optionalNumber(value.ttlMs, "ttlMs"),
    attempt: optionalNumber(value.attempt, "attempt"),
    trace: trace as CreateEnvelopeOptions["trace"],
  };
}

function parseEnvelope(value: unknown): AgentEnvelope<unknown> {
  if (!isRecord(value)) {
    throw new Error("envelope must be an object");
  }
  return value as unknown as AgentEnvelope<unknown>;
}

async function readJson(req: IncomingMessage, maxBodyBytes: number): Promise<unknown> {
  const chunks: Buffer[] = [];
  let total = 0;
  for await (const chunk of req) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    total += buffer.byteLength;
    if (total > maxBodyBytes) {
      throw Object.assign(new Error("request body too large"), { statusCode: 413, code: "body_too_large" });
    }
    chunks.push(buffer);
  }
  if (chunks.length === 0) {
    return {};
  }
  try {
    return JSON.parse(Buffer.concat(chunks).toString("utf8")) as unknown;
  } catch {
    throw Object.assign(new Error("request body must be valid JSON"), { statusCode: 400, code: "invalid_json" });
  }
}

function sendJson(res: ServerResponse, statusCode: number, body: JsonBody | ErrorBody): void {
  const payload = JSON.stringify(body);
  res.writeHead(statusCode, {
    "content-type": "application/json; charset=utf-8",
    "content-length": Buffer.byteLength(payload),
  });
  res.end(payload);
}

function sendNoContent(res: ServerResponse): void {
  res.writeHead(204);
  res.end();
}

function bearerToken(req: IncomingMessage): string | undefined {
  const header = req.headers.authorization;
  if (header === undefined) {
    return undefined;
  }
  const value = Array.isArray(header) ? header[0] : header;
  const match = /^Bearer\s+(.+)$/i.exec(value);
  return match?.[1];
}

function methodNotAllowed(res: ServerResponse): void {
  sendJson(res, 405, { error: { code: "method_not_allowed", message: "method not allowed" } });
}

function badRequest(res: ServerResponse, message: string): void {
  sendJson(res, 400, { error: { code: "bad_request", message } });
}

export class CoordinationHttpService {
  readonly plane: InMemoryCoordinationPlane;
  private readonly sharedToken?: string;
  private readonly maxBodyBytes: number;
  private readonly clock: () => Date | string | number;

  constructor(options: CoordinationHttpServiceOptions = {}) {
    this.plane = options.plane ?? new InMemoryCoordinationPlane({ now: options.now });
    this.sharedToken = options.sharedToken;
    this.maxBodyBytes = options.maxBodyBytes ?? DEFAULT_MAX_BODY_BYTES;
    this.clock = options.now ?? (() => new Date());
  }

  createServer(): Server {
    return http.createServer((req, res) => {
      void this.handle(req, res).catch((error: unknown) => {
        const statusCode = typeof (error as { statusCode?: unknown }).statusCode === "number" ? (error as { statusCode: number }).statusCode : 500;
        const code = typeof (error as { code?: unknown }).code === "string" ? (error as { code: string }).code : statusCode === 500 ? "internal_error" : "bad_request";
        const message = error instanceof Error ? error.message : "request failed";
        sendJson(res, statusCode, { error: { code, message } });
      });
    });
  }

  async handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const url = new URL(req.url ?? "/", "http://coordination.local");
    if (url.pathname === "/healthz") {
      if (req.method !== "GET") {
        methodNotAllowed(res);
        return;
      }
      sendJson(res, 200, { ok: true });
      return;
    }

    if (!this.isAuthorized(req)) {
      sendJson(res, 401, { error: { code: "unauthorized", message: "missing or invalid bearer token" } });
      return;
    }

    const segments = url.pathname.split("/").filter(Boolean);
    if (segments[0] !== "v1") {
      sendJson(res, 404, { error: { code: "not_found", message: "unknown route" } });
      return;
    }

    if (segments[1] === "agents") {
      await this.handleAgents(req, res, segments);
      return;
    }
    if (segments[1] === "groups") {
      await this.handleGroups(req, res, segments);
      return;
    }
    if (segments[1] === "lookup") {
      await this.handleLookup(req, res);
      return;
    }
    if (segments[1] === "messages") {
      await this.handleMessages(req, res, segments);
      return;
    }
    if (segments[1] === "inbox") {
      await this.handleInbox(req, res, segments, url);
      return;
    }
    if (segments[1] === "dead-letters") {
      await this.handleDeadLetters(req, res, segments);
      return;
    }

    sendJson(res, 404, { error: { code: "not_found", message: "unknown route" } });
  }

  private isAuthorized(req: IncomingMessage): boolean {
    if (this.sharedToken === undefined || this.sharedToken.length === 0) {
      return true;
    }
    return bearerToken(req) === this.sharedToken;
  }

  private async handleAgents(req: IncomingMessage, res: ServerResponse, segments: string[]): Promise<void> {
    if (segments.length === 2 && req.method === "POST") {
      const body = await readJson(req, this.maxBodyBytes);
      if (!isRecord(body)) {
        badRequest(res, "body must be an object");
        return;
      }
      const registered = this.plane.registerAgent(parseAgentIdentity(body.identity), {
        now: body.now as string | number | Date | undefined,
        ttlMs: optionalNumber(body.ttlMs, "ttlMs"),
        expiresAt: body.expiresAt as string | number | Date | undefined,
        metadata: optionalStringMap(body.metadata, "metadata"),
      });
      sendJson(res, 201, registered as unknown as JsonBody);
      return;
    }
    if (segments.length === 3 && req.method === "DELETE") {
      sendJson(res, 200, { deleted: this.plane.unregisterAgent(decodeURIComponent(segments[2])) });
      return;
    }
    methodNotAllowed(res);
  }

  private async handleGroups(req: IncomingMessage, res: ServerResponse, segments: string[]): Promise<void> {
    if (segments.length === 2 && req.method === "POST") {
      const body = await readJson(req, this.maxBodyBytes);
      if (!isRecord(body)) {
        badRequest(res, "body must be an object");
        return;
      }
      const group = this.plane.registerGroup(parseAgentGroup(body.group), {
        metadata: optionalStringMap(body.metadata, "metadata"),
      });
      sendJson(res, 201, group as unknown as JsonBody);
      return;
    }
    if (segments.length === 3 && req.method === "DELETE") {
      sendJson(res, 200, { deleted: this.plane.unregisterGroup(decodeURIComponent(segments[2])) });
      return;
    }
    methodNotAllowed(res);
  }

  private async handleLookup(req: IncomingMessage, res: ServerResponse): Promise<void> {
    if (req.method !== "POST") {
      methodNotAllowed(res);
      return;
    }
    const body = await readJson(req, this.maxBodyBytes);
    if (!isRecord(body)) {
      badRequest(res, "body must be an object");
      return;
    }
    const agents = this.plane.lookup(parseAddress(body.address), (body.now as string | number | Date | undefined) ?? this.clock());
    sendJson(res, 200, { agents: agents as unknown as JsonBody });
  }

  private async handleMessages(req: IncomingMessage, res: ServerResponse, segments: string[]): Promise<void> {
    if (segments.length === 2 && req.method === "POST") {
      const body = await readJson(req, this.maxBodyBytes);
      if (!isRecord(body)) {
        badRequest(res, "body must be an object");
        return;
      }
      const result = this.plane.sendMessage(parseMessage(body.message), {
        ...parseCreateEnvelopeOptions(body),
        now: (body.now as string | number | Date | undefined) ?? this.clock(),
      });
      sendJson(res, 202, deliveryResultBody(result));
      return;
    }
    if (segments.length === 3 && segments[2] === "envelopes" && req.method === "POST") {
      const body = await readJson(req, this.maxBodyBytes);
      if (!isRecord(body)) {
        badRequest(res, "body must be an object");
        return;
      }
      const result = this.plane.sendEnvelope(parseEnvelope(body.envelope), {
        now: (body.now as string | number | Date | undefined) ?? this.clock(),
      });
      sendJson(res, 202, deliveryResultBody(result));
      return;
    }
    methodNotAllowed(res);
  }

  private async handleInbox(req: IncomingMessage, res: ServerResponse, segments: string[], url: URL): Promise<void> {
    if (segments.length < 3) {
      sendJson(res, 404, { error: { code: "not_found", message: "agent id required" } });
      return;
    }
    const agentId = decodeURIComponent(segments[2]);
    if (segments.length === 3 && req.method === "GET") {
      const limitParam = url.searchParams.get("limit");
      const includeExpiredParam = url.searchParams.get("includeExpired");
      const limit = limitParam === null ? undefined : Number.parseInt(limitParam, 10);
      if (limit !== undefined && (!Number.isInteger(limit) || limit < 0)) {
        badRequest(res, "limit must be a non-negative integer");
        return;
      }
      const includeExpired = includeExpiredParam === null ? undefined : includeExpiredParam === "true";
      const messages = this.plane.readInbox(agentId, {
        now: url.searchParams.get("now") ?? this.clock(),
        limit,
        includeExpired,
      });
      sendJson(res, 200, { messages: messages as unknown as JsonBody });
      return;
    }
    if (segments.length === 5 && segments[3] === "acks" && req.method === "POST") {
      sendJson(res, 200, { acked: this.plane.ack(agentId, decodeURIComponent(segments[4])) });
      return;
    }
    if (segments.length === 5 && segments[3] === "failures" && req.method === "POST") {
      const body = await readJson(req, this.maxBodyBytes);
      if (!isRecord(body)) {
        badRequest(res, "body must be an object");
        return;
      }
      const result = this.plane.failDelivery(agentId, decodeURIComponent(segments[4]), {
        now: (body.now as string | number | Date | undefined) ?? this.clock(),
        reason: optionalString(body.reason, "reason"),
      });
      if (result === undefined) {
        sendJson(res, 404, { error: { code: "not_found", message: "delivery not found" } });
        return;
      }
      sendJson(res, 200, result as unknown as JsonBody);
      return;
    }
    methodNotAllowed(res);
  }

  private async handleDeadLetters(req: IncomingMessage, res: ServerResponse, segments: string[]): Promise<void> {
    if (segments.length !== 3 || req.method !== "GET") {
      methodNotAllowed(res);
      return;
    }
    sendJson(res, 200, { messages: this.plane.readDeadLetters(decodeURIComponent(segments[2])) as unknown as JsonBody });
  }
}

function deliveryResultBody(result: DeliveryResult<unknown>): JsonBody {
  return {
    envelope: result.envelope,
    recipients: result.recipients,
    deliveries: result.deliveries,
  };
}

export function createCoordinationHttpServer(options: CoordinationHttpServiceOptions = {}): Server {
  return new CoordinationHttpService(options).createServer();
}

export function startCoordinationHttpServer(options: CoordinationHttpServiceOptions = {}): Server {
  const port = Number.parseInt(process.env.PORT ?? "8080", 10);
  const host = process.env.HOST ?? "0.0.0.0";
  const sharedToken = options.sharedToken ?? process.env.COORDINATION_SHARED_TOKEN;
  const server = createCoordinationHttpServer({ ...options, sharedToken });
  server.listen(port, host);
  return server;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  startCoordinationHttpServer();
}

export type { DeliveryResult, InboxMessage, RegisteredAgent };
