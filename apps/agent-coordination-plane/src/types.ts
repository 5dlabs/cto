export type AgentRuntime = "hermes" | "openclaw" | "external" | "unknown";

export type AgentRole =
  | "coordinator"
  | "planner"
  | "coder"
  | "reviewer"
  | "operator"
  | "researcher"
  | "general"
  | (string & {});

export type MessagePriority = "low" | "normal" | "high" | "urgent";

export type AgentMessageKind =
  | "command"
  | "request"
  | "response"
  | "event"
  | "status"
  | "discovery";

export interface AgentIdentity {
  agentId: string;
  role: AgentRole;
  runtime: AgentRuntime;
  projectId?: string;
  taskId?: string;
  sessionId?: string;
  podName?: string;
  model?: string;
  metadata?: Record<string, string>;
}

export interface AgentGroup {
  groupId: string;
  name: string;
  members: AgentIdentity[];
  projectId?: string;
  taskId?: string;
  role?: AgentRole;
  metadata?: Record<string, string>;
}

export type AgentAddress =
  | { kind: "agent"; agentId: string; projectId?: string; taskId?: string }
  | { kind: "role"; role: AgentRole; projectId?: string; taskId?: string }
  | { kind: "task"; projectId: string; taskId: string }
  | { kind: "project"; projectId: string }
  | { kind: "group"; groupId: string }
  | { kind: "broadcast" };

export interface AgentMessage<TBody = unknown> {
  messageId: string;
  kind: AgentMessageKind;
  from: AgentIdentity;
  to: AgentAddress;
  body: TBody;
  createdAt: string;
  priority?: MessagePriority;
  replyToMessageId?: string;
  correlationId?: string;
  metadata?: Record<string, string>;
}

export interface AgentEnvelope<TBody = unknown> {
  schema: "cto.agent.envelope.v1";
  envelopeId: string;
  subject: string;
  message: AgentMessage<TBody>;
  sentAt: string;
  replyTo?: string;
  ttlMs?: number;
  attempt?: number;
  trace?: {
    traceId?: string;
    spanId?: string;
    parentSpanId?: string;
  };
}

export interface CreateEnvelopeOptions {
  envelopeId?: string;
  subject?: string;
  sentAt?: string;
  replyTo?: string;
  ttlMs?: number;
  attempt?: number;
  trace?: AgentEnvelope["trace"];
}
