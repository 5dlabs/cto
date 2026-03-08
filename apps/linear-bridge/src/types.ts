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
  /** Optional metadata from Lobster workflows (e.g., linearIssueId) */
  metadata?: Record<string, string>;
}

/** Tracks an active conversation synced to a Linear issue */
export interface LinearConversationState {
  /** Sorted participant names joined by "-" */
  id: string;
  participants: Set<string>;
  /** The Linear issue ID to post comments to */
  linearIssueId: string;
  /** The Linear project ID for context */
  linearProjectId?: string;
  lastActivity: Date;
  messageCount: number;
}

/** Maps a conversation to a Linear issue */
export interface IssueMapping {
  conversationId: string;
  linearIssueId: string;
  linearProjectId?: string;
  createdAt: Date;
}

/** End-conversation signal marker */
export const END_CONVERSATION_SIGNAL = "[END_CONVERSATION]";
