import type { BridgeConfig } from "./config.js";
import type { LinearClient } from "./linear-client.js";
import type { AgentMessage } from "./types.js";
import { END_CONVERSATION_SIGNAL } from "./types.js";
import { IssueManager } from "./issue-manager.js";

export interface Bridge {
  /** Handle an incoming agent message (via HTTP POST) */
  handleMessage(subject: string, msg: AgentMessage): void;
  /** Stop the bridge (clears timers) */
  stop(): void;
}

/**
 * Orchestrator: receives agent messages (via HTTP), finds/creates conversations
 * via IssueManager, and posts formatted markdown comments to Linear issues.
 */
export function createBridge(
  config: BridgeConfig,
  linearClient: LinearClient,
  issueManager: IssueManager,
  logger: { info: Function; warn: Function; error: Function },
): Bridge {
  // Periodic check for stale conversations
  const gcInterval = setInterval(async () => {
    const stale = issueManager.getStaleConversations(config.inactivityTimeoutMs);
    for (const conv of stale) {
      logger.info(
        `Conversation "${conv.id}" timed out (inactive for ${config.inactivityTimeoutMs}ms)`,
      );
      await postSummaryComment(conv.linearIssueId, conv);
      issueManager.endConversation(conv.id);
    }
  }, 60_000);

  async function postSummaryComment(
    issueId: string,
    conv: { id: string; participants: Set<string>; messageCount: number; lastActivity: Date },
  ): Promise<void> {
    const participants = [...conv.participants].join(", ");
    const body = `---\n**Conversation ended** — ${conv.messageCount} messages between ${participants}\nLast activity: ${conv.lastActivity.toISOString()}`;
    try {
      await linearClient.createComment(issueId, body);
    } catch (err) {
      logger.error(`Failed to post summary comment to issue ${issueId}:`, err);
    }
  }

  /**
   * Extract the target agent name from a NATS subject.
   * e.g. "agent.forge.inbox" -> "forge"
   */
  function extractRecipient(subject: string, msg: AgentMessage): string | undefined {
    if (msg.to) return msg.to;
    const parts = subject.split(".");
    if (parts.length >= 3 && parts[0] === "agent" && parts[2] === "inbox") {
      return parts[1];
    }
    return undefined;
  }

  /**
   * Format an agent message as a rich markdown comment for Linear.
   */
  function formatComment(msg: AgentMessage, recipient: string | undefined): string {
    const priorityIcon = msg.priority === "urgent" ? "🔴 urgent" : "🟢 normal";
    const arrow = recipient ? ` → ${recipient}` : "";

    const headerParts = [`**${msg.from}**${arrow}`, priorityIcon];

    const model = msg.metadata?.model;
    const provider = msg.metadata?.provider;
    if (model) headerParts.push(provider ? `${model} (${provider})` : model);

    const coordinator = msg.metadata?.coordinator;
    if (coordinator) headerParts.push(coordinator);

    const step = msg.metadata?.step;
    if (step) headerParts.push(step);

    headerParts.push(msg.timestamp);

    return `${headerParts.join(" | ")}\n\n${msg.message}`;
  }

  return {
    handleMessage(subject: string, msg: AgentMessage): void {
      const recipient = extractRecipient(subject, msg);

      // Check for end-conversation signal
      if (msg.message.includes(END_CONVERSATION_SIGNAL)) {
        const conv = issueManager.getConversationForAgent(msg.from);
        if (conv) {
          // Post the end message, then a summary, then free the conversation
          const comment = formatComment(msg, recipient);
          linearClient.createComment(conv.linearIssueId, comment).catch((err) => {
            logger.error(`Failed to post end comment:`, err);
          });
          postSummaryComment(conv.linearIssueId, conv).then(() => {
            logger.info(`Conversation "${conv.id}" ended by ${msg.from}`);
            issueManager.endConversation(conv.id);
          });
        }
        return;
      }

      // Extract optional issue ID from message metadata
      const metadataIssueId = msg.metadata?.linearIssueId;

      issueManager
        .allocate(msg.from, recipient, metadataIssueId)
        .then((allocation) => {
          if (!allocation) {
            if (recipient) {
              logger.warn(`Could not allocate issue for ${msg.from} -> ${recipient}`);
            }
            return;
          }

          const { conversation } = allocation;
          const comment = formatComment(msg, recipient);

          linearClient.createComment(conversation.linearIssueId, comment).catch((err) => {
            logger.error(
              `Failed to post comment to issue ${conversation.linearIssueId}:`,
              err,
            );
          });
        })
        .catch((err) => {
          logger.error(`Failed to allocate issue for ${msg.from}:`, err);
        });
    },

    stop(): void {
      clearInterval(gcInterval);
    },
  };
}
