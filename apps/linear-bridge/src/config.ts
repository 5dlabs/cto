export interface BridgeConfig {
  /** Linear API key (lin_api_*) */
  linearApiKey: string;
  /** Default Linear team ID for issue creation */
  linearTeamId: string;
  /** Inactivity timeout in ms before a conversation is GC'd (default: 1 hour) */
  inactivityTimeoutMs: number;
  /** Optional default project ID — if set, ad-hoc issues are created in this project */
  defaultProjectId?: string;
  /** Port for HTTP server (default: 3100) */
  httpPort: number;
  /** HMAC-SHA256 secret for verifying Linear webhook signatures */
  linearWebhookSecret?: string;
  /** Enable Agent Session API features (default: true) */
  agentSessionsEnabled: boolean;
  /** Discord bridge URL for cross-cancel callbacks (default: http://discord-bridge.bots.svc:3200) */
  discordBridgeUrl: string;
}

export function loadConfig(): BridgeConfig {
  const linearApiKey = process.env.LINEAR_API_KEY;
  if (!linearApiKey) {
    throw new Error("LINEAR_API_KEY environment variable is required");
  }

  const linearTeamId = process.env.LINEAR_TEAM_ID;
  if (!linearTeamId) {
    throw new Error("LINEAR_TEAM_ID environment variable is required");
  }

  return {
    linearApiKey,
    linearTeamId,
    inactivityTimeoutMs: parseInt(process.env.INACTIVITY_TIMEOUT_MS ?? "3600000", 10),
    defaultProjectId: process.env.LINEAR_DEFAULT_PROJECT_ID || undefined,
    httpPort: parseInt(process.env.WEBHOOK_PORT ?? "3100", 10),
    linearWebhookSecret: process.env.LINEAR_WEBHOOK_SECRET || undefined,
    agentSessionsEnabled: process.env.AGENT_SESSIONS_ENABLED !== "false",
    discordBridgeUrl: process.env.DISCORD_BRIDGE_URL ?? "http://discord-bridge.bots.svc:3200",
  };
}
