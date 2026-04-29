export interface BridgeConfig {
  /** Discord bot token */
  discordToken: string;
  /** Discord guild (server) ID */
  guildId: string;
  /** Inactivity timeout in ms before a conversation room is freed (default: 1 hour) */
  inactivityTimeoutMs: number;
  /** Category name for bot conversation channels */
  categoryName: string;
  /** HTTP server port for notification API (default: 3200) */
  httpPort: number;
  /** Linear bridge URL for cross-cancel callbacks (default: http://linear-bridge.bots.svc:3100) */
  linearBridgeUrl: string;
  /** Optional fixed channel ID for deliberation traffic (bypasses room allocator) */
  deliberationChannelId?: string;
  /** Optional JSON file for persisted runtime route mappings */
  presenceRouteStorePath?: string;
  /** Optional NATS URL for presence event fanout */
  natsUrl?: string;
  /** Shared bearer token required for /presence/* endpoints */
  presenceSharedToken?: string;
}

function defaultWorkspaceRoot(): string {
  return process.env.WORKSPACE?.trim() || process.cwd();
}

export function loadConfig(): BridgeConfig {
  const discordToken = process.env.DISCORD_BRIDGE_TOKEN;
  if (!discordToken) {
    throw new Error("DISCORD_BRIDGE_TOKEN environment variable is required");
  }

  return {
    discordToken,
    guildId: process.env.GUILD_ID ?? "1409006087331512342",
    inactivityTimeoutMs: parseInt(process.env.INACTIVITY_TIMEOUT_MS ?? "3600000", 10),
    categoryName: process.env.CATEGORY_NAME ?? "Bot Conversations",
    httpPort: parseInt(process.env.HTTP_PORT ?? "3200", 10),
    linearBridgeUrl: process.env.LINEAR_BRIDGE_URL ?? "http://linear-bridge.bots.svc:3100",
    deliberationChannelId: process.env.DISCORD_DELIBERATION_CHANNEL_ID?.trim() || undefined,
    presenceRouteStorePath:
      process.env.PRESENCE_ROUTE_STORE_PATH?.trim() || `${defaultWorkspaceRoot()}/.intake/presence-routes.json`,
    natsUrl: process.env.NATS_URL?.trim() || undefined,
    presenceSharedToken: process.env.PRESENCE_SHARED_TOKEN?.trim() || undefined,
  };
}
