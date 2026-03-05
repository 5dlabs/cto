import type { LinearConversationState } from "./types.js";
import type { LinearClient, LinearIssue } from "./linear-client.js";

export interface IssueAllocation {
  conversation: LinearConversationState;
  isNew: boolean;
}

/**
 * Manages the mapping between agent conversations and Linear issues.
 *
 * Analogous to discord-bridge's RoomManager, but instead of pre-allocated channels,
 * maps conversations to Linear issues (existing or newly created).
 *
 * Strategy:
 * 1. If the NATS message metadata contains a linearIssueId, use that issue
 * 2. Fallback: create a new issue in the default project for ad-hoc conversations
 */
export class IssueManager {
  private conversations = new Map<string, LinearConversationState>();
  /** Maps agent name -> conversation ID they're currently in */
  private agentToConversation = new Map<string, string>();

  constructor(
    private linearClient: LinearClient,
    private teamId: string,
    private defaultProjectId: string | undefined,
    private logger: { info: Function; warn: Function; error: Function },
  ) {}

  /**
   * Find or create a conversation for a message from `sender` to `recipient`.
   * If issueId is provided (from message metadata), use that issue directly.
   */
  async allocate(
    sender: string,
    recipient: string | undefined,
    issueId?: string,
  ): Promise<IssueAllocation | null> {
    if (!recipient) return null;

    // Check if either agent is already in a conversation
    const senderConvId = this.agentToConversation.get(sender);
    const recipientConvId = this.agentToConversation.get(recipient);

    // Both in the same conversation
    if (senderConvId && senderConvId === recipientConvId) {
      const conv = this.conversations.get(senderConvId)!;
      conv.lastActivity = new Date();
      conv.messageCount++;
      return { conversation: conv, isNew: false };
    }

    // One of them is in a conversation -- add the other
    if (senderConvId && !recipientConvId) {
      const conv = this.conversations.get(senderConvId)!;
      conv.participants.add(recipient);
      conv.lastActivity = new Date();
      conv.messageCount++;
      const newId = this.buildConversationId(conv.participants);
      if (newId !== conv.id) {
        this.conversations.delete(conv.id);
        conv.id = newId;
        this.conversations.set(newId, conv);
      }
      this.agentToConversation.set(recipient, newId);
      for (const p of conv.participants) {
        this.agentToConversation.set(p, newId);
      }
      return { conversation: conv, isNew: true };
    }

    if (recipientConvId && !senderConvId) {
      const conv = this.conversations.get(recipientConvId)!;
      conv.participants.add(sender);
      conv.lastActivity = new Date();
      conv.messageCount++;
      const newId = this.buildConversationId(conv.participants);
      if (newId !== conv.id) {
        this.conversations.delete(conv.id);
        conv.id = newId;
        this.conversations.set(newId, conv);
      }
      this.agentToConversation.set(sender, newId);
      for (const p of conv.participants) {
        this.agentToConversation.set(p, newId);
      }
      return { conversation: conv, isNew: true };
    }

    // Both in different conversations -- merge into sender's
    if (senderConvId && recipientConvId && senderConvId !== recipientConvId) {
      const senderConv = this.conversations.get(senderConvId)!;
      const recipientConv = this.conversations.get(recipientConvId)!;

      for (const p of recipientConv.participants) {
        senderConv.participants.add(p);
      }
      senderConv.lastActivity = new Date();
      senderConv.messageCount += recipientConv.messageCount + 1;

      this.conversations.delete(recipientConvId);

      const newId = this.buildConversationId(senderConv.participants);
      this.conversations.delete(senderConvId);
      senderConv.id = newId;
      this.conversations.set(newId, senderConv);
      for (const p of senderConv.participants) {
        this.agentToConversation.set(p, newId);
      }

      return { conversation: senderConv, isNew: true };
    }

    // Neither is in a conversation -- create a new one
    const participants = new Set([sender, recipient]);
    const convId = this.buildConversationId(participants);

    // Determine which Linear issue to use
    let linearIssueId: string;
    let linearProjectId: string | undefined;

    if (issueId) {
      // Use the issue ID from message metadata
      linearIssueId = issueId;
      this.logger.info(`Using provided issue ${issueId} for conversation ${convId}`);
    } else {
      // Create a new issue for this ad-hoc conversation
      try {
        const issue = await this.createConversationIssue(sender, recipient);
        linearIssueId = issue.id;
        linearProjectId = this.defaultProjectId;
        this.logger.info(
          `Created issue ${issue.identifier} for conversation ${convId}`,
        );
      } catch (err) {
        this.logger.error(`Failed to create issue for conversation ${convId}:`, err);
        return null;
      }
    }

    const conv: LinearConversationState = {
      id: convId,
      participants,
      linearIssueId,
      linearProjectId,
      lastActivity: new Date(),
      messageCount: 1,
    };

    this.conversations.set(convId, conv);
    this.agentToConversation.set(sender, convId);
    this.agentToConversation.set(recipient, convId);

    return { conversation: conv, isNew: true };
  }

  /**
   * End a conversation and clean up mappings.
   */
  endConversation(conversationId: string): LinearConversationState | null {
    const conv = this.conversations.get(conversationId);
    if (!conv) return null;

    for (const p of conv.participants) {
      if (this.agentToConversation.get(p) === conversationId) {
        this.agentToConversation.delete(p);
      }
    }
    this.conversations.delete(conversationId);

    return conv;
  }

  /**
   * Find conversations that have been inactive longer than the timeout.
   */
  getStaleConversations(timeoutMs: number): LinearConversationState[] {
    const cutoff = Date.now() - timeoutMs;
    const stale: LinearConversationState[] = [];
    for (const conv of this.conversations.values()) {
      if (conv.lastActivity.getTime() < cutoff) {
        stale.push(conv);
      }
    }
    return stale;
  }

  /** Find the conversation a given agent is currently in */
  getConversationForAgent(agent: string): LinearConversationState | undefined {
    const convId = this.agentToConversation.get(agent);
    return convId ? this.conversations.get(convId) : undefined;
  }

  private async createConversationIssue(sender: string, recipient: string): Promise<LinearIssue> {
    return this.linearClient.createIssue({
      title: `Agent conversation: ${sender} ↔ ${recipient}`,
      description: `Auto-created by linear-bridge for agent conversation tracking.\n\nParticipants: ${sender}, ${recipient}`,
      teamId: this.teamId,
      projectId: this.defaultProjectId,
    });
  }

  private buildConversationId(participants: Set<string>): string {
    return [...participants].sort().join("-");
  }
}
