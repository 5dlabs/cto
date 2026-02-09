import type { ConversationState, RoomState } from "./types.js";
import { NUM_ROOMS } from "./types.js";

export interface RoomAllocation {
  conversation: ConversationState;
  isNew: boolean;
}

/**
 * Manages the mapping between agent conversations and pre-allocated Discord rooms.
 *
 * Conversation detection: when a message arrives from A to B, check if either A or B
 * is already in an active conversation. If so, add the other participant to that
 * conversation. Otherwise, create a new conversation.
 */
export class RoomManager {
  private rooms: RoomState[] = [];
  private conversations = new Map<string, ConversationState>();
  /** Maps agent name -> conversation ID they're currently in */
  private agentToConversation = new Map<string, string>();

  initialize(channelIds: string[]): void {
    this.rooms = channelIds.map((channelId, index) => ({
      index,
      channelId,
      conversationId: null,
    }));
  }

  /**
   * Find or create a conversation for a message from `sender` to `recipient`.
   * Returns the conversation and whether a room rename is needed.
   */
  allocate(sender: string, recipient: string | undefined): RoomAllocation | null {
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
      // Update sender mapping if ID changed
      for (const p of conv.participants) {
        this.agentToConversation.set(p, newId);
      }
      return { conversation: conv, isNew: true }; // isNew triggers rename
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

    // Both in different conversations -- merge into the sender's conversation
    if (senderConvId && recipientConvId && senderConvId !== recipientConvId) {
      const senderConv = this.conversations.get(senderConvId)!;
      const recipientConv = this.conversations.get(recipientConvId)!;

      // Merge recipient's conversation into sender's
      for (const p of recipientConv.participants) {
        senderConv.participants.add(p);
      }
      senderConv.lastActivity = new Date();
      senderConv.messageCount += recipientConv.messageCount + 1;

      // Free the recipient's room
      const recipientRoom = this.rooms.find(r => r.conversationId === recipientConvId);
      if (recipientRoom) {
        recipientRoom.conversationId = null;
      }
      this.conversations.delete(recipientConvId);

      // Update conversation ID and mappings
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
    const freeRoom = this.rooms.find(r => r.conversationId === null);
    if (!freeRoom) {
      return null; // All rooms occupied
    }

    const participants = new Set([sender, recipient]);
    const convId = this.buildConversationId(participants);
    const conv: ConversationState = {
      id: convId,
      participants,
      roomIndex: freeRoom.index,
      channelId: freeRoom.channelId,
      lastActivity: new Date(),
      messageCount: 1,
    };

    freeRoom.conversationId = convId;
    this.conversations.set(convId, conv);
    this.agentToConversation.set(sender, convId);
    this.agentToConversation.set(recipient, convId);

    return { conversation: conv, isNew: true };
  }

  /**
   * End a conversation and free its room.
   * Returns the freed room index and channel ID, or null if not found.
   */
  endConversation(conversationId: string): { roomIndex: number; channelId: string } | null {
    const conv = this.conversations.get(conversationId);
    if (!conv) return null;

    const room = this.rooms.find(r => r.conversationId === conversationId);
    if (room) {
      room.conversationId = null;
    }

    for (const p of conv.participants) {
      if (this.agentToConversation.get(p) === conversationId) {
        this.agentToConversation.delete(p);
      }
    }
    this.conversations.delete(conversationId);

    return room ? { roomIndex: room.index, channelId: room.channelId } : null;
  }

  /**
   * Find conversations that have been inactive longer than the timeout.
   */
  getStaleConversations(timeoutMs: number): ConversationState[] {
    const cutoff = Date.now() - timeoutMs;
    const stale: ConversationState[] = [];
    for (const conv of this.conversations.values()) {
      if (conv.lastActivity.getTime() < cutoff) {
        stale.push(conv);
      }
    }
    return stale;
  }

  /** Find the conversation a given agent is currently in */
  getConversationForAgent(agent: string): ConversationState | undefined {
    const convId = this.agentToConversation.get(agent);
    return convId ? this.conversations.get(convId) : undefined;
  }

  /** Get all channel IDs for rooms */
  getRoomChannelIds(): string[] {
    return this.rooms.map(r => r.channelId);
  }

  get roomCount(): number {
    return this.rooms.length;
  }

  private buildConversationId(participants: Set<string>): string {
    return [...participants].sort().join("-");
  }
}
