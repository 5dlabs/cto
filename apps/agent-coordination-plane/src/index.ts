export type {
  AgentAddress,
  AgentEnvelope,
  AgentGroup,
  AgentIdentity,
  AgentMessage,
  AgentMessageKind,
  AgentRole,
  AgentRuntime,
  CreateEnvelopeOptions,
  MessagePriority,
} from "./types.js";

import type { AgentAddress, AgentEnvelope, AgentMessage, CreateEnvelopeOptions } from "./types.js";

export const AGENT_ENVELOPE_SCHEMA = "cto.agent.envelope.v1" as const;
export const AGENT_SUBJECT_PREFIX = "cto.agent.v1" as const;

export interface ValidationIssue {
  path: string;
  message: string;
}

export type ValidationResult<T> = { ok: true; value: T } | { ok: false; issues: ValidationIssue[] };

function issue(path: string, message: string): ValidationIssue {
  return { path, message };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isStringMap(value: unknown): value is Record<string, string> {
  return isRecord(value) && Object.values(value).every((entry) => typeof entry === "string");
}

function validateString(value: unknown, path: string, issues: ValidationIssue[]): value is string {
  if (typeof value === "string" && value.length > 0) {
    return true;
  }
  issues.push(issue(path, "must be a non-empty string"));
  return false;
}

function validateOptionalStringMap(value: unknown, path: string, issues: ValidationIssue[]): void {
  if (value === undefined) {
    return;
  }
  if (!isStringMap(value)) {
    issues.push(issue(path, "must be an object whose values are strings"));
  }
}

export function safeSubjectSegment(value: string | undefined, fallback = "unknown"): string {
  return (value ?? fallback)
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "") || fallback;
}

export function subjectForAddress(address: AgentAddress): string {
  switch (address.kind) {
    case "agent":
      return [AGENT_SUBJECT_PREFIX, "agent", safeSubjectSegment(address.agentId), "inbox"].join(".");
    case "role":
      return [AGENT_SUBJECT_PREFIX, "role", safeSubjectSegment(address.role), "inbox"].join(".");
    case "task":
      return [
        AGENT_SUBJECT_PREFIX,
        "project",
        safeSubjectSegment(address.projectId),
        "task",
        safeSubjectSegment(address.taskId),
        "inbox",
      ].join(".");
    case "project":
      return [AGENT_SUBJECT_PREFIX, "project", safeSubjectSegment(address.projectId), "inbox"].join(".");
    case "group":
      return [AGENT_SUBJECT_PREFIX, "group", safeSubjectSegment(address.groupId), "inbox"].join(".");
    case "broadcast":
      return [AGENT_SUBJECT_PREFIX, "broadcast"].join(".");
  }
}

export function addressKey(address: AgentAddress): string {
  switch (address.kind) {
    case "agent":
      return [
        "agent",
        safeSubjectSegment(address.projectId, "any-project"),
        safeSubjectSegment(address.taskId, "any-task"),
        safeSubjectSegment(address.agentId),
      ].join(":");
    case "role":
      return [
        "role",
        safeSubjectSegment(address.projectId, "any-project"),
        safeSubjectSegment(address.taskId, "any-task"),
        safeSubjectSegment(address.role),
      ].join(":");
    case "task":
      return ["task", safeSubjectSegment(address.projectId), safeSubjectSegment(address.taskId)].join(":");
    case "project":
      return ["project", safeSubjectSegment(address.projectId)].join(":");
    case "group":
      return ["group", safeSubjectSegment(address.groupId)].join(":");
    case "broadcast":
      return "broadcast";
  }
}

export function createEnvelope<TBody>(
  message: AgentMessage<TBody>,
  options: CreateEnvelopeOptions = {},
): AgentEnvelope<TBody> {
  return {
    schema: AGENT_ENVELOPE_SCHEMA,
    envelopeId: options.envelopeId ?? message.messageId,
    subject: options.subject ?? subjectForAddress(message.to),
    message,
    sentAt: options.sentAt ?? new Date().toISOString(),
    replyTo: options.replyTo,
    ttlMs: options.ttlMs,
    attempt: options.attempt,
    trace: options.trace,
  };
}

export function validateAgentEnvelope(value: unknown): ValidationResult<AgentEnvelope> {
  const issues: ValidationIssue[] = [];

  if (!isRecord(value)) {
    return { ok: false, issues: [issue("$", "must be an object")] };
  }

  if (value.schema !== AGENT_ENVELOPE_SCHEMA) {
    issues.push(issue("schema", `must be ${AGENT_ENVELOPE_SCHEMA}`));
  }
  validateString(value.envelopeId, "envelopeId", issues);
  validateString(value.subject, "subject", issues);
  validateString(value.sentAt, "sentAt", issues);
  if (value.replyTo !== undefined && typeof value.replyTo !== "string") {
    issues.push(issue("replyTo", "must be a string when present"));
  }
  if (value.ttlMs !== undefined && (typeof value.ttlMs !== "number" || value.ttlMs < 0)) {
    issues.push(issue("ttlMs", "must be a non-negative number when present"));
  }
  if (value.attempt !== undefined && (typeof value.attempt !== "number" || value.attempt < 0)) {
    issues.push(issue("attempt", "must be a non-negative number when present"));
  }

  const message = value.message;
  if (!isRecord(message)) {
    issues.push(issue("message", "must be an object"));
  } else {
    validateString(message.messageId, "message.messageId", issues);
    validateString(message.kind, "message.kind", issues);
    validateString(message.createdAt, "message.createdAt", issues);
    if (!isRecord(message.from)) {
      issues.push(issue("message.from", "must be an object"));
    } else {
      validateString(message.from.agentId, "message.from.agentId", issues);
      validateString(message.from.role, "message.from.role", issues);
      validateString(message.from.runtime, "message.from.runtime", issues);
      validateOptionalStringMap(message.from.metadata, "message.from.metadata", issues);
    }
    if (!isRecord(message.to)) {
      issues.push(issue("message.to", "must be an object"));
    } else {
      validateString(message.to.kind, "message.to.kind", issues);
    }
    validateOptionalStringMap(message.metadata, "message.metadata", issues);
  }

  if (issues.length > 0) {
    return { ok: false, issues };
  }
  return { ok: true, value: value as unknown as AgentEnvelope };
}
