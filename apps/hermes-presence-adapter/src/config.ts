import type { AdapterConfig } from "./types.js";

function optional(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed.replace(/\/+$/, "") : undefined;
}

export function loadConfig(): AdapterConfig {
  const port = Number.parseInt(process.env.HTTP_PORT ?? "3305", 10);
  const podIp = process.env.POD_IP?.trim();
  const workerUrl = process.env.PRESENCE_WORKER_URL?.trim() || (podIp ? `http://${podIp}:${port}` : undefined);
  const routeId = process.env.PRESENCE_ROUTE_ID?.trim() || process.env.CODERUN_ID?.trim();
  const agentId = process.env.AGENT_ID?.trim();
  const discordAccountId = process.env.DISCORD_ACCOUNT_ID?.trim();
  const discordChannelId = process.env.PRESENCE_DISCORD_CHANNEL_ID?.trim() || process.env.DISCORD_CHANNEL_ID?.trim();

  return {
    port,
    hermesApiUrl: optional(process.env.HERMES_API_URL),
    hermesInputUrl: optional(process.env.HERMES_INPUT_URL),
    inboxPath: process.env.HERMES_INBOX_PATH?.trim() || "/workspace/presence-inbox.jsonl",
    presenceRouterUrl: optional(process.env.PRESENCE_ROUTER_URL),
    presenceSharedToken: process.env.PRESENCE_SHARED_TOKEN?.trim() || undefined,
    route: routeId && agentId && workerUrl
      ? {
          route_id: routeId,
          runtime: "hermes",
          agent_id: agentId,
          project_id: process.env.PROJECT_ID?.trim() || undefined,
          task_id: process.env.TASK_ID?.trim() || undefined,
          coderun_id: process.env.CODERUN_ID?.trim() || routeId,
          worker_url: workerUrl,
          session_key: process.env.PRESENCE_SESSION_KEY?.trim() || undefined,
          discord: discordAccountId
            ? {
                account_id: discordAccountId,
                guild_id: process.env.DISCORD_GUILD_ID?.trim() || undefined,
                channel_id: discordChannelId || undefined,
                thread_id: process.env.DISCORD_THREAD_ID?.trim() || undefined,
              }
            : undefined,
          metadata: {
            source: "hermes-presence-adapter",
          },
        }
      : undefined,
  };
}
