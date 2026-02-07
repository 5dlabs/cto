import type { NatsClientHandle } from "./client";
import type { AgentMessage, MessagePriority } from "./types";

/**
 * Create the `nats` tool that agents can invoke to send messages.
 *
 * Usage:
 *   nats(action="publish", to="forge", message="ping", priority="urgent")
 *   nats(action="publish", subject="agent.all.broadcast", message="status check")
 *   nats(action="request", to="metal", message="status?", timeoutMs=10000)
 */
export function createNatsTool(
  agentName: string,
  getClient: () => NatsClientHandle | null,
) {
  return {
    id: "nats",
    name: "nats",
    description: [
      "Send messages to other agents via NATS.",
      "",
      "Actions:",
      '  publish — Fire-and-forget message. Use `to` for direct or `subject` for custom routing.',
      '  request — Send and wait for a reply. Returns the response message.',
      "",
      "Parameters:",
      '  action   — "publish" or "request"',
      "  to       — Target agent name (resolves to agent.<name>.inbox)",
      "  subject  — Raw NATS subject (overrides `to`)",
      "  message  — Message body text",
      '  priority — "normal" (default) or "urgent" (wakes recipient immediately)',
      "  timeoutMs — Timeout for request action (default 10000)",
    ].join("\n"),
    parameters: {
      type: "object",
      properties: {
        action: {
          type: "string",
          enum: ["publish", "request"],
          description: "publish (fire-and-forget) or request (wait for reply)",
        },
        to: {
          type: "string",
          description: "Target agent name (e.g. forge, metal, trader)",
        },
        subject: {
          type: "string",
          description: "Raw NATS subject (overrides to). E.g. agent.all.broadcast",
        },
        message: {
          type: "string",
          description: "Message body text",
        },
        priority: {
          type: "string",
          enum: ["normal", "urgent"],
          description: "Message priority (default: normal)",
        },
        timeoutMs: {
          type: "number",
          description: "Timeout in ms for request action (default: 10000)",
        },
      },
      required: ["action", "message"],
    },

    async execute(
      _toolCallId: string,
      params: {
        action: "publish" | "request";
        to?: string;
        subject?: string;
        message: string;
        priority?: MessagePriority;
        timeoutMs?: number;
      },
    ): Promise<{ content: { type: string; text: string }[] }> {
      const result = (text: string) => ({
        content: [{ type: "text", text }],
      });

      const client = getClient();
      if (!client || !client.isConnected()) {
        return result(
          "Error: NATS client is not connected. The nats-messenger service may not have started.",
        );
      }

      const resolvedSubject =
        params.subject ?? (params.to ? `agent.${params.to}.inbox` : null);

      if (!resolvedSubject) {
        return result(
          'Error: Either "to" (agent name) or "subject" (raw NATS subject) is required.',
        );
      }

      const msg: AgentMessage = {
        from: agentName,
        to: params.to,
        subject: resolvedSubject,
        message: params.message,
        priority: params.priority ?? "normal",
        timestamp: new Date().toISOString(),
      };

      if (params.action === "publish") {
        client.publish(resolvedSubject, msg);
        return result(
          `Published to ${resolvedSubject} (priority=${msg.priority})`,
        );
      }

      if (params.action === "request") {
        try {
          const reply = await client.request(
            resolvedSubject,
            msg,
            params.timeoutMs ?? 10000,
          );
          return result(`Reply from ${reply.from}: ${reply.message}`);
        } catch (err) {
          return result(`Request to ${resolvedSubject} failed: ${err}`);
        }
      }

      return result(
        `Error: Unknown action "${params.action}". Use "publish" or "request".`,
      );
    },
  };
}
