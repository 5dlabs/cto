/** Agent message format (transport-agnostic, received via HTTP POST) */
export interface AgentMessage {
  from: string;
  to?: string;
  subject: string;
  message: string;
  priority: "normal" | "urgent";
  timestamp: string;
  replyTo?: string;
  type?: "message";
  role?: string;
  /** Optional metadata (model, provider, step, coordinator, etc.) */
  metadata?: Record<string, string>;
}

/** Tracks an active conversation between agents */
export interface ConversationState {
  /** Sorted participant names joined by "-" */
  id: string;
  participants: Set<string>;
  roomIndex: number;
  channelId: string;
  lastActivity: Date;
  messageCount: number;
}

/** Tracks a single pre-allocated Discord channel */
export interface RoomState {
  index: number;
  channelId: string;
  conversationId: string | null;
}

/** End-conversation signal marker */
export const END_CONVERSATION_SIGNAL = "[END_CONVERSATION]";

/** Number of pre-allocated bot conversation channels */
export const NUM_ROOMS = 5;

/** Default channel name when a room is available */
export function availableChannelName(index: number): string {
  return `room-${index}-available`;
}
