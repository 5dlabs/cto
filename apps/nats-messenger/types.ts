/** Plugin configuration from openclaw.json */
export interface NatsConfig {
  enabled: boolean;
  url: string;
  agentName: string;
  subjects: string[];
  reconnectWaitMs?: number;
  maxReconnectAttempts?: number;
}

/** Priority levels for inter-agent messages */
export type MessagePriority = "normal" | "urgent";

/** Wire format for messages published to NATS */
export interface AgentMessage {
  from: string;
  to?: string;
  subject: string;
  message: string;
  priority: MessagePriority;
  timestamp: string;
  replyTo?: string;
}

/** Parsed inbound message ready for injection into a session */
export interface ProcessedMessage {
  sessionKey: string;
  eventText: string;
  priority: MessagePriority;
  raw: AgentMessage;
}
