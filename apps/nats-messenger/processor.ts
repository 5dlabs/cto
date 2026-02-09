import type { AgentMessage, ProcessedMessage, MessagePriority } from "./types";

/**
 * Convert a raw NATS AgentMessage into a ProcessedMessage suitable for
 * injection into the agent's session via enqueueSystemEvent().
 */
export function processInboundMessage(
  subject: string,
  msg: AgentMessage,
  selfName: string,
): ProcessedMessage {
  const priority: MessagePriority = msg.priority ?? "normal";

  // Build human-readable event text the agent will see
  const lines: string[] = [];
  lines.push(`[nats] Message from **${msg.from}**`);
  if (msg.to && msg.to !== selfName) {
    lines.push(`  (broadcast on ${subject})`);
  }
  if (priority === "urgent") {
    lines.push(`  Priority: URGENT`);
  }
  lines.push(`  ${msg.message}`);
  if (msg.replyTo) {
    lines.push(`  [Reply expected - use nats(action="reply", to="${msg.from}", message="your reply")]`);
  }

  // Session key — route to the agent's primary session
  const sessionKey = `nats:${selfName}`;

  return {
    sessionKey,
    eventText: lines.join("\n"),
    priority,
    raw: msg,
  };
}
