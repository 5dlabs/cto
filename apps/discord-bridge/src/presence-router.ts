import fs from "node:fs";
import path from "node:path";
import type { DiscordHandle } from "./discord-client.js";
import type { PresenceFabric } from "./presence-fabric.js";
import {
  validatePresenceInbound,
  validatePresenceOutbound,
  validatePresenceRoute,
  type PresenceInbound,
  type PresenceOutbound,
  type PresenceRoute,
} from "./presence-types.js";

export interface PresenceRouter {
  registerRoute(payload: unknown): Promise<PresenceRoute>;
  deleteRoute(routeId: string): Promise<boolean>;
  listRoutes(): PresenceRoute[];
  routeInbound(payload: unknown): Promise<{ accepted: true; route: PresenceRoute; workerStatus: number }>;
  handleOutbound(payload: unknown): Promise<{ accepted: true; message_id?: string }>;
}

function nowIso(): string {
  return new Date().toISOString();
}

function normalizeWorkerEndpoint(workerUrl: string): string {
  const trimmed = workerUrl.replace(/\/+$/, "");
  return trimmed.endsWith("/presence/inbound") ? trimmed : `${trimmed}/presence/inbound`;
}

function routeScore(route: PresenceRoute, event: PresenceInbound): number {
  let score = 0;
  if (route.runtime !== event.runtime) return -1;
  if (route.agent_id !== event.agent_id) return -1;

  if (route.coderun_id && event.coderun_id && route.coderun_id === event.coderun_id) score += 100;
  if (route.project_id && event.project_id && route.project_id === event.project_id) score += 20;
  if (route.task_id && event.task_id && route.task_id === event.task_id) score += 20;

  const routeDiscord = route.discord;
  if (routeDiscord?.guild_id && routeDiscord.guild_id !== event.discord.guild_id) return -1;
  if (routeDiscord?.channel_id && routeDiscord.channel_id !== event.discord.channel_id) return -1;
  if (routeDiscord?.thread_id && routeDiscord.thread_id !== event.discord.thread_id) return -1;

  if (routeDiscord?.channel_id) score += 30;
  if (routeDiscord?.thread_id) score += 40;
  return score;
}

function readRoutes(routeStorePath: string | undefined, logger: { warn: Function }): PresenceRoute[] {
  if (!routeStorePath || !fs.existsSync(routeStorePath)) {
    return [];
  }
  try {
    const parsed = JSON.parse(fs.readFileSync(routeStorePath, "utf8"));
    if (Array.isArray(parsed)) {
      return parsed.filter((route): route is PresenceRoute => {
        const result = validatePresenceRoute(route);
        return result.ok;
      });
    }
  } catch (err) {
    logger.warn(`Failed to load presence routes from ${routeStorePath}: ${err}`);
  }
  return [];
}

function writeRoutes(routeStorePath: string | undefined, routes: PresenceRoute[], logger: { warn: Function }): void {
  if (!routeStorePath) {
    return;
  }
  try {
    fs.mkdirSync(path.dirname(routeStorePath), { recursive: true });
    const tmpPath = `${routeStorePath}.tmp`;
    fs.writeFileSync(tmpPath, JSON.stringify(routes, null, 2));
    fs.renameSync(tmpPath, routeStorePath);
  } catch (err) {
    logger.warn(`Failed to persist presence routes to ${routeStorePath}: ${err}`);
  }
}

export function createPresenceRouter(
  discord: DiscordHandle,
  logger: { info: Function; warn: Function; error: Function },
  routeStorePath?: string,
  fabric?: PresenceFabric,
  workerSharedToken?: string,
): PresenceRouter {
  const routes = new Map<string, PresenceRoute>();
  for (const route of readRoutes(routeStorePath, logger)) {
    routes.set(route.route_id, route);
  }

  function persist(): void {
    writeRoutes(routeStorePath, [...routes.values()], logger);
  }

  function findRoute(event: PresenceInbound): PresenceRoute | undefined {
    const candidates = [...routes.values()]
      .map((route) => ({ route, score: routeScore(route, event) }))
      .filter((entry) => entry.score > 0)
      .sort((a, b) => b.score - a.score);
    if (candidates.length > 1 && candidates[0]?.score === candidates[1]?.score) {
      throw new Error(
        `Ambiguous presence route for runtime=${event.runtime} agent=${event.agent_id} coderun=${event.coderun_id ?? "none"}`,
      );
    }
    return candidates[0]?.route;
  }

  async function postToWorker(route: PresenceRoute, event: PresenceInbound): Promise<number> {
    const endpoint = normalizeWorkerEndpoint(route.worker_url);
    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(workerSharedToken ? { Authorization: `Bearer ${workerSharedToken}` } : {}),
      },
      body: JSON.stringify({ ...event, session_key: route.session_key }),
      signal: AbortSignal.timeout(10_000),
    });
    if (!response.ok) {
      const body = await response.text().catch(() => "");
      throw new Error(`Worker ${endpoint} returned HTTP ${response.status}${body ? `: ${body}` : ""}`);
    }
    return response.status;
  }

  async function applyOutbound(intent: PresenceOutbound): Promise<{ message_id?: string }> {
    if (intent.op === "send") {
      const targetId = intent.target.thread_id ?? intent.target.channel_id;
      const messageId = await discord.postMessage(targetId, intent.content);
      return { message_id: messageId };
    }
    if (intent.op === "edit") {
      const targetId = intent.target.thread_id ?? intent.target.channel_id;
      await discord.editPlainMessage(targetId, intent.message_id, intent.content);
      return { message_id: intent.message_id };
    }
    if (intent.op === "react") {
      const targetId = intent.target.thread_id ?? intent.target.channel_id;
      await discord.addReaction(targetId, intent.message_id, intent.emoji);
      return { message_id: intent.message_id };
    }
    if (intent.op === "typing") {
      if (intent.active) {
        const targetId = intent.target.thread_id ?? intent.target.channel_id;
        await discord.sendTyping(targetId);
      }
      return {};
    }
    logger.info(`Worker status: ${intent.state}${intent.detail ? ` (${intent.detail})` : ""}`);
    if (intent.target && intent.message_id) {
      const emoji = intent.state === "failed" ? "❌" : intent.state === "done" ? "✅" : intent.state === "running" ? "👀" : undefined;
      if (emoji) {
        const targetId = intent.target.thread_id ?? intent.target.channel_id;
        await discord.addReaction(targetId, intent.message_id, emoji);
      }
    }
    return { message_id: intent.message_id };
  }

  return {
    async registerRoute(payload): Promise<PresenceRoute> {
      const result = validatePresenceRoute(payload);
      if (!result.ok) {
        throw new Error(result.error);
      }
      const existing = routes.get(result.value.route_id);
      const timestamp = nowIso();
      const route: PresenceRoute = {
        ...existing,
        ...result.value,
        created_at: existing?.created_at ?? timestamp,
        updated_at: timestamp,
      };
      routes.set(route.route_id, route);
      persist();
      fabric?.publishRoute(route);
      logger.info(`Registered presence route ${route.route_id} -> ${route.worker_url}`);
      return route;
    },

    async deleteRoute(routeId): Promise<boolean> {
      const deleted = routes.delete(routeId);
      if (deleted) {
        persist();
        logger.info(`Deleted presence route ${routeId}`);
      }
      return deleted;
    },

    listRoutes(): PresenceRoute[] {
      return [...routes.values()];
    },

    async routeInbound(payload): Promise<{ accepted: true; route: PresenceRoute; workerStatus: number }> {
      const result = validatePresenceInbound(payload);
      if (!result.ok) {
        throw new Error(result.error);
      }
      const route = findRoute(result.value);
      if (!route) {
        throw new Error(
          `No presence route for runtime=${result.value.runtime} agent=${result.value.agent_id} coderun=${result.value.coderun_id ?? "none"}`,
        );
      }
      fabric?.publishInbound(result.value, route);
      const workerStatus = await postToWorker(route, result.value);
      return { accepted: true, route, workerStatus };
    },

    async handleOutbound(payload): Promise<{ accepted: true; message_id?: string }> {
      const result = validatePresenceOutbound(payload);
      if (!result.ok) {
        throw new Error(result.error);
      }
      fabric?.publishOutbound(result.value);
      const applied = await applyOutbound(result.value);
      return { accepted: true, ...applied };
    },
  };
}
