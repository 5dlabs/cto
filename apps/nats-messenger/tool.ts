import type { NatsClientHandle } from "./client";
import type { AgentMessage, MessagePriority, RosterEntry } from "./types";
import type { PingPongCheck } from "./service";

/**
 * Build the dynamic tool description including roster and usage guidance.
 */
function buildDescription(agentName: string, roster: RosterEntry[]): string {
  const lines: string[] = [
    "Send messages to other agents via NATS (cross-pod inter-agent messaging).",
    "",
    "WHEN TO USE: Always use nats() for messaging other agents. sessions_send() only works",
    "in-process (same pod) and will fail for cross-pod communication. Every agent runs in its",
    "own pod, so nats() is the correct choice for all inter-agent messages.",
    "",
    "Actions:",
    '  publish  — Fire-and-forget message. Use `to` for direct or `subject` for custom routing.',
    '  request  — Send and wait for a reply. Returns the response message.',
    '  discover — Find which agents are currently online. No other params needed.',
    "",
    "Parameters:",
    '  action   — "publish", "request", or "discover"',
    "  to       — Target agent name (resolves to agent.<name>.inbox)",
    "  subject  — Raw NATS subject (overrides `to`)",
    "  message  — Message body text (required for publish/request)",
    '  priority — "normal" (default) or "urgent" (wakes recipient immediately)',
    "  timeoutMs — Timeout for request/discover (default 10000/3000)",
    "",
    "Examples:",
    '  nats(action="publish", to="forge", message="deploy the staging build")',
    '  nats(action="request", to="planner", message="what tasks are pending?")',
    '  nats(action="publish", subject="agent.all.broadcast", message="status check")',
    '  nats(action="discover")',
  ];

  if (roster.length > 0) {
    lines.push("");
    lines.push("Known Agents:");
    for (const entry of roster) {
      const marker = entry.id === agentName ? " (you)" : "";
      lines.push(`  ${entry.id}${marker} — ${entry.role}`);
    }
  }

  return lines.join("\n");
}

/**
 * Create the `nats` tool that agents can invoke to send messages.
 */
export function createNatsTool(
  agentName: string,
  getClient: () => NatsClientHandle | null,
  roster: RosterEntry[],
  checkPingPong: (peer: string) => PingPongCheck,
  recordPingPong: (peer: string) => void,
) {
  return {
    id: "nats",
    name: "nats",
    description: buildDescription(agentName, roster),
    parameters: {
      type: "object",
      properties: {
        action: {
          type: "string",
          enum: ["publish", "request", "discover"],
          description: "publish (fire-and-forget), request (wait for reply), or discover (find online agents)",
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
          description: "Timeout in ms for request/discover action (default: 10000/3000)",
        },
      },
      required: ["action"],
    },

    async execute(
      _toolCallId: string,
      params: {
        action: "publish" | "request" | "discover";
        to?: string;
        subject?: string;
        message?: string;
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

      // --- discover action ---
      if (params.action === "discover") {
        try {
          const peers = await client.discoverPeers(params.timeoutMs ?? 3000);
          if (peers.length === 0) {
            return result("No agents responded to discovery ping. They may be offline or starting up.");
          }
          const lines = peers.map(
            (p) => `  ${p.from} — ${p.role ?? "unknown role"}`,
          );
          return result(`Online agents (${peers.length}):\n${lines.join("\n")}`);
        } catch (err) {
          return result(`Discovery failed: ${err}`);
        }
      }

      // --- publish / request require message ---
      if (!params.message) {
        return result(
          'Error: "message" parameter is required for publish and request actions.',
        );
      }

      const resolvedSubject =
        params.subject ?? (params.to ? `agent.${params.to}.inbox` : null);

      if (!resolvedSubject) {
        return result(
          'Error: Either "to" (agent name) or "subject" (raw NATS subject) is required.',
        );
      }

      // Outbound ping-pong guard
      const peer = params.to ?? resolvedSubject;
      const check = checkPingPong(peer);
      if (!check.allowed) {
        return result(
          `Ping-pong limit reached for ${peer} (${check.count}/${check.limit} messages in window). ` +
          `Wait for the window to reset or message a different agent.`,
        );
      }
      recordPingPong(peer);

      const msg: AgentMessage = {
        from: agentName,
        to: params.to,
        subject: resolvedSubject,
        message: params.message,
        priority: params.priority ?? "normal",
        timestamp: new Date().toISOString(),
        type: "message",
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
        `Error: Unknown action "${params.action}". Use "publish", "request", or "discover".`,
      );
    },
  };
}
