/** Plugin configuration from openclaw.json */
export interface NatsConfig {
  enabled: boolean;
  url: string;
  agentName: string;
  subjects: string[];
  reconnectWaitMs?: number;
  maxReconnectAttempts?: number;
  agentRole?: string;
  roster?: RosterEntry[];
  maxPingPongTurns?: number;
  pingPongWindowMs?: number;
}

/** Priority levels for inter-agent messages */
export type MessagePriority = "normal" | "urgent";

/** Discriminator for message types (backward-compat: undefined = "message") */
export type AgentMessageType = "message" | "discovery_ping" | "discovery_pong";

/** Wire format for messages published to NATS */
export interface AgentMessage {
  from: string;
  to?: string;
  subject: string;
  message: string;
  priority: MessagePriority;
  timestamp: string;
  replyTo?: string;
  type?: AgentMessageType;
  role?: string;
}

/** An entry in the static agent roster */
export interface RosterEntry {
  id: string;
  role: string;
}

/** Ping-pong guard state for a single peer */
export interface PingPongState {
  count: number;
  lastReset: number;
}

/** Parsed inbound message ready for injection into a session */
export interface ProcessedMessage {
  sessionKey: string;
  eventText: string;
  priority: MessagePriority;
  raw: AgentMessage;
}
