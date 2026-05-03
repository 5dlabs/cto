export type PresenceRuntime = "openclaw" | "hermes" | "hosted";
export type PresenceEventType = "message" | "interaction" | "thread" | "lifecycle";
export type PresenceChatType = "dm" | "group" | "thread";
export type PresenceStatusState = "started" | "running" | "blocked" | "done" | "failed";

export interface PresenceAttachment {
  url: string;
  content_type?: string;
  filename?: string;
}

export interface PresenceDiscordContext {
  account_id: string;
  guild_id?: string;
  channel_id: string;
  thread_id?: string;
  message_id?: string;
  user_id?: string;
  user_name?: string;
  chat_type?: PresenceChatType;
  parent_channel_id?: string;
  mentioned_agent_ids?: string[];
}

export interface PresenceDiscordEvent {
  schema: "cto.presence.v1";
  event_type: PresenceEventType;
  agent_id?: string;
  project_id?: string;
  task_id?: string;
  coderun_id?: string;
  discord: PresenceDiscordContext;
  text?: string;
  attachments?: PresenceAttachment[];
  metadata?: Record<string, string>;
}

export interface PresenceInbound extends PresenceDiscordEvent {
  runtime: PresenceRuntime;
  agent_id: string;
  session_key?: string;
}

export interface DiscordTarget {
  account_id?: string;
  channel_id: string;
  thread_id?: string;
}

export type PresenceOutbound =
  | { op: "send"; target: DiscordTarget; content: string }
  | { op: "edit"; target: DiscordTarget; message_id: string; content: string }
  | { op: "react"; target: DiscordTarget; message_id: string; emoji: string }
  | { op: "typing"; target: DiscordTarget; active: boolean }
  | { op: "status"; state: PresenceStatusState; detail?: string; target?: DiscordTarget; message_id?: string };

export interface PresenceRoute {
  route_id: string;
  runtime: PresenceRuntime;
  agent_id: string;
  worker_url: string;
  project_id?: string;
  task_id?: string;
  coderun_id?: string;
  discord?: Partial<PresenceDiscordContext>;
  session_key?: string;
  metadata?: Record<string, string>;
  created_at?: string;
  updated_at?: string;
}

type ValidationError = { ok: false; error: string };
export type ValidationResult<T> =
  | { ok: true; value: T }
  | ValidationError;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(obj: Record<string, unknown>, key: string): string | undefined {
  const value = obj[key];
  return typeof value === "string" && value.trim() ? value : undefined;
}

function optionalStringField(obj: Record<string, unknown>, key: string): string | undefined {
  const value = obj[key];
  return typeof value === "string" && value.trim() ? value : undefined;
}

function stringMapField(obj: Record<string, unknown>, key: string): ValidationResult<Record<string, string> | undefined> {
  const value = obj[key];
  if (value === undefined) {
    return { ok: true, value: undefined };
  }
  if (!isRecord(value) || !Object.values(value).every((item) => typeof item === "string")) {
    return { ok: false, error: `${key} must be a string map` };
  }
  return { ok: true, value: value as Record<string, string> };
}

function isPresenceRuntime(value: unknown): value is PresenceRuntime {
  return value === "openclaw" || value === "hermes" || value === "hosted";
}

function isPresenceEventType(value: unknown): value is PresenceEventType {
  return value === "message" || value === "interaction" || value === "thread" || value === "lifecycle";
}

function validateDiscordContext(value: unknown): ValidationResult<PresenceDiscordContext> {
  if (!isRecord(value)) {
    return { ok: false, error: "discord must be an object" };
  }
  const accountId = stringField(value, "account_id");
  const channelId = stringField(value, "channel_id");
  if (!accountId) return { ok: false, error: "discord.account_id is required" };
  if (!channelId) return { ok: false, error: "discord.channel_id is required" };

  const mentioned = value.mentioned_agent_ids;
  if (mentioned !== undefined && (!Array.isArray(mentioned) || !mentioned.every((v) => typeof v === "string"))) {
    return { ok: false, error: "discord.mentioned_agent_ids must be a string array" };
  }

  const chatType = value.chat_type;
  if (chatType !== undefined && chatType !== "dm" && chatType !== "group" && chatType !== "thread") {
    return { ok: false, error: "discord.chat_type must be dm, group, or thread" };
  }

  return {
    ok: true,
    value: {
      account_id: accountId,
      channel_id: channelId,
      guild_id: optionalStringField(value, "guild_id"),
      thread_id: optionalStringField(value, "thread_id"),
      message_id: optionalStringField(value, "message_id"),
      user_id: optionalStringField(value, "user_id"),
      user_name: optionalStringField(value, "user_name"),
      chat_type: chatType as PresenceChatType | undefined,
      parent_channel_id: optionalStringField(value, "parent_channel_id"),
      mentioned_agent_ids: mentioned as string[] | undefined,
    },
  };
}

export function validatePresenceDiscordEvent(payload: unknown): ValidationResult<PresenceDiscordEvent> {
  if (!isRecord(payload)) {
    return { ok: false, error: "payload must be an object" };
  }
  if (payload.schema !== "cto.presence.v1") {
    return { ok: false, error: "schema must be cto.presence.v1" };
  }
  if (!isPresenceEventType(payload.event_type)) {
    return { ok: false, error: "event_type must be message, interaction, thread, or lifecycle" };
  }
  const discord = validateDiscordContext(payload.discord);
  if (discord.ok === false) {
    return { ok: false, error: discord.error };
  }

  const attachments = payload.attachments;
  if (
    attachments !== undefined &&
    (!Array.isArray(attachments) || !attachments.every((item) => isRecord(item) && typeof item.url === "string"))
  ) {
    return { ok: false, error: "attachments must be an array of objects with url" };
  }

  const metadata = stringMapField(payload, "metadata");
  if (metadata.ok === false) {
    return { ok: false, error: metadata.error };
  }

  return {
    ok: true,
    value: {
      schema: "cto.presence.v1",
      event_type: payload.event_type,
      agent_id: optionalStringField(payload, "agent_id"),
      project_id: optionalStringField(payload, "project_id"),
      task_id: optionalStringField(payload, "task_id"),
      coderun_id: optionalStringField(payload, "coderun_id"),
      discord: discord.value,
      text: optionalStringField(payload, "text"),
      attachments: attachments as PresenceAttachment[] | undefined,
      metadata: metadata.value,
    },
  };
}

export function validatePresenceInbound(payload: unknown): ValidationResult<PresenceInbound> {
  if (!isRecord(payload)) {
    return { ok: false, error: "payload must be an object" };
  }
  if (payload.schema !== "cto.presence.v1") {
    return { ok: false, error: "schema must be cto.presence.v1" };
  }
  if (!isPresenceEventType(payload.event_type)) {
    return { ok: false, error: "event_type must be message, interaction, thread, or lifecycle" };
  }
  if (!isPresenceRuntime(payload.runtime)) {
    return { ok: false, error: "runtime must be openclaw, hermes, or hosted" };
  }
  const agentId = stringField(payload, "agent_id");
  if (!agentId) {
    return { ok: false, error: "agent_id is required" };
  }
  const discord = validateDiscordContext(payload.discord);
  if (discord.ok === false) {
    return { ok: false, error: discord.error };
  }

  const attachments = payload.attachments;
  if (
    attachments !== undefined &&
    (!Array.isArray(attachments) || !attachments.every((item) => isRecord(item) && typeof item.url === "string"))
  ) {
    return { ok: false, error: "attachments must be an array of objects with url" };
  }

  const metadata = stringMapField(payload, "metadata");
  if (metadata.ok === false) {
    return { ok: false, error: metadata.error };
  }

  return {
    ok: true,
    value: {
      schema: "cto.presence.v1",
      event_type: payload.event_type,
      runtime: payload.runtime,
      agent_id: agentId,
      project_id: optionalStringField(payload, "project_id"),
      task_id: optionalStringField(payload, "task_id"),
      coderun_id: optionalStringField(payload, "coderun_id"),
      discord: discord.value,
      text: optionalStringField(payload, "text"),
      attachments: attachments as PresenceAttachment[] | undefined,
      metadata: metadata.value,
    },
  };
}

export function validatePresenceRoute(payload: unknown): ValidationResult<PresenceRoute> {
  if (!isRecord(payload)) {
    return { ok: false, error: "payload must be an object" };
  }
  if (!isPresenceRuntime(payload.runtime)) {
    return { ok: false, error: "runtime must be openclaw, hermes, or hosted" };
  }
  const agentId = stringField(payload, "agent_id");
  const workerUrl = stringField(payload, "worker_url");
  if (!agentId) return { ok: false, error: "agent_id is required" };
  if (!workerUrl) return { ok: false, error: "worker_url is required" };
  const routeId =
    optionalStringField(payload, "route_id") ??
    optionalStringField(payload, "coderun_id") ??
    `${payload.runtime}:${agentId}:${Date.now()}`;

  const metadata = stringMapField(payload, "metadata");
  if (metadata.ok === false) {
    return { ok: false, error: metadata.error };
  }

  return {
    ok: true,
    value: {
      route_id: routeId,
      runtime: payload.runtime,
      agent_id: agentId,
      worker_url: workerUrl,
      project_id: optionalStringField(payload, "project_id"),
      task_id: optionalStringField(payload, "task_id"),
      coderun_id: optionalStringField(payload, "coderun_id"),
      discord: isRecord(payload.discord) ? (payload.discord as Partial<PresenceDiscordContext>) : undefined,
      session_key: optionalStringField(payload, "session_key"),
      metadata: metadata.value,
    },
  };
}

export function validatePresenceOutbound(payload: unknown): ValidationResult<PresenceOutbound> {
  if (!isRecord(payload)) {
    return { ok: false, error: "payload must be an object" };
  }
  const op = payload.op;
  if (op === "send" || op === "edit") {
    if (!isRecord(payload.target) || typeof payload.target.channel_id !== "string") {
      return { ok: false, error: "target.channel_id is required" };
    }
    if (typeof payload.content !== "string") {
      return { ok: false, error: "content is required" };
    }
    if (op === "edit" && typeof payload.message_id !== "string") {
      return { ok: false, error: "message_id is required for edit" };
    }
    return { ok: true, value: payload as PresenceOutbound };
  }
  if (op === "react") {
    if (!isRecord(payload.target) || typeof payload.target.channel_id !== "string") {
      return { ok: false, error: "target.channel_id is required" };
    }
    if (typeof payload.message_id !== "string" || typeof payload.emoji !== "string") {
      return { ok: false, error: "message_id and emoji are required for react" };
    }
    return { ok: true, value: payload as PresenceOutbound };
  }
  if (op === "typing") {
    if (!isRecord(payload.target) || typeof payload.target.channel_id !== "string") {
      return { ok: false, error: "target.channel_id is required" };
    }
    return { ok: true, value: payload as PresenceOutbound };
  }
  if (op === "status") {
    const state = payload.state;
    if (state !== "started" && state !== "running" && state !== "blocked" && state !== "done" && state !== "failed") {
      return { ok: false, error: "state must be started, running, blocked, done, or failed" };
    }
    return { ok: true, value: payload as PresenceOutbound };
  }
  return { ok: false, error: "op must be send, edit, react, typing, or status" };
}
