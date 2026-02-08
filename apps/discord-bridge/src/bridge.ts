import { EmbedBuilder } from "discord.js";
import type { BridgeConfig } from "./config.js";
import type { DiscordHandle } from "./discord-client.js";
import type { AgentMessage } from "./types.js";
import { END_CONVERSATION_SIGNAL, availableChannelName } from "./types.js";
import { RoomManager } from "./room-manager.js";

export interface Bridge {
  /** Handle an incoming NATS agent message */
  handleMessage(subject: string, msg: AgentMessage): void;
  /** Stop the bridge (clears timers) */
  stop(): void;
}

/**
 * Orchestrator: receives NATS messages, finds/creates conversations via RoomManager,
 * and posts rich embeds to the appropriate Discord channel.
 */
export function createBridge(
  config: BridgeConfig,
  discord: DiscordHandle,
  roomManager: RoomManager,
  logger: { info: Function; warn: Function; error: Function },
): Bridge {
  // Periodic check for stale conversations
  const gcInterval = setInterval(() => {
    const stale = roomManager.getStaleConversations(config.inactivityTimeoutMs);
    for (const conv of stale) {
      logger.info(`Conversation "${conv.id}" timed out (inactive for ${config.inactivityTimeoutMs}ms)`);
      freeConversation(conv.id);
    }
  }, 60_000); // Check every minute

  function freeConversation(conversationId: string): void {
    const freed = roomManager.endConversation(conversationId);
    if (freed) {
      const name = availableChannelName(freed.roomIndex);
      discord.renameChannel(freed.channelId, name).catch(() => {});
      logger.info(`Freed room ${freed.roomIndex}, renamed to "${name}"`);
    }
  }

  /**
   * Extract the target agent name from a NATS subject.
   * e.g. "agent.forge.inbox" -> "forge"
   */
  function extractRecipient(subject: string, msg: AgentMessage): string | undefined {
    if (msg.to) return msg.to;
    const parts = subject.split(".");
    // agent.<name>.inbox pattern
    if (parts.length >= 3 && parts[0] === "agent" && parts[2] === "inbox") {
      return parts[1];
    }
    return undefined;
  }

  /**
   * Build a Discord channel name from participant names.
   * Sorts and joins with "-", truncated to 100 chars (Discord limit).
   */
  function channelName(participants: Set<string>): string {
    const name = [...participants].sort().join("-");
    return name.slice(0, 100);
  }

  function buildEmbed(msg: AgentMessage, recipient: string | undefined, roomLabel: string): EmbedBuilder {
    const embed = new EmbedBuilder()
      .setAuthor({ name: msg.from })
      .setDescription(msg.message)
      .setTimestamp(new Date(msg.timestamp));

    if (msg.priority === "urgent") {
      embed.setColor(0xed4245); // red
    } else {
      embed.setColor(0x5865f2); // blurple/blue
    }

    const footerParts: string[] = [];
    if (recipient) footerParts.push(`To: ${recipient}`);
    footerParts.push(roomLabel);
    embed.setFooter({ text: footerParts.join(" | ") });

    return embed;
  }

  return {
    handleMessage(subject: string, msg: AgentMessage): void {
      const recipient = extractRecipient(subject, msg);

      // Check for end-conversation signal
      if (msg.message.includes(END_CONVERSATION_SIGNAL)) {
        const conv = roomManager.getConversationForAgent(msg.from);
        if (conv) {
          // Post the end message first, then free the room
          const embed = buildEmbed(msg, recipient, `room-${conv.roomIndex}`);
          discord.postEmbed(conv.channelId, embed).catch(() => {});
          logger.info(`Conversation "${conv.id}" ended by ${msg.from}`);
          freeConversation(conv.id);
        }
        return;
      }

      const allocation = roomManager.allocate(msg.from, recipient);

      if (!allocation) {
        // No recipient or all rooms occupied
        if (recipient) {
          logger.warn(
            `All ${roomManager.roomCount} rooms occupied, cannot relay ${msg.from} -> ${recipient}`,
          );
        }
        return;
      }

      const { conversation, isNew } = allocation;

      // Rename the channel if participants changed
      if (isNew) {
        const name = channelName(conversation.participants);
        discord.renameChannel(conversation.channelId, name).catch(() => {});
      }

      const roomLabel = `room-${conversation.roomIndex}`;
      const embed = buildEmbed(msg, recipient, roomLabel);
      discord.postEmbed(conversation.channelId, embed).catch(() => {});
    },

    stop(): void {
      clearInterval(gcInterval);
    },
  };
}
