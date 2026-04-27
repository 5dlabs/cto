import type { NatsConfig } from "./types";

/** Validate and return config with defaults applied */
export function resolveConfig(raw: Partial<NatsConfig>): NatsConfig {
  if (!raw.url) throw new Error("nats-messenger: config.url is required");
  if (!raw.agentName) throw new Error("nats-messenger: config.agentName is required");

  return {
    enabled: raw.enabled ?? true,
    url: raw.url,
    agentName: raw.agentName,
    subjects: raw.subjects ?? [
      `agent.${raw.agentName}.inbox`,
      "agent.all.broadcast",
    ],
    reconnectWaitMs: raw.reconnectWaitMs ?? 2000,
    maxReconnectAttempts: raw.maxReconnectAttempts ?? -1,
    agentRole: raw.agentRole ?? "general",
    roster: raw.roster ?? [],
    maxPingPongTurns: raw.maxPingPongTurns ?? 10,
    pingPongWindowMs: raw.pingPongWindowMs ?? 300000,
  };
}
