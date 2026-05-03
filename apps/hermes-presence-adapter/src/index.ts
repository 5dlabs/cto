import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { pathToFileURL } from "node:url";
import { loadConfig } from "./config.js";
import { appendInbox, deletePresenceRoute, postHermesInput, postPresenceStatus, registerPresenceRoute, startHermesRun } from "./hermes-client.js";
import type { AdapterConfig, PresenceInbound } from "./types.js";

const logger = {
  info: (...args: unknown[]) => console.log("[hermes-presence-adapter]", ...args),
  warn: (...args: unknown[]) => console.warn("[hermes-presence-adapter]", ...args),
  error: (...args: unknown[]) => console.error("[hermes-presence-adapter]", ...args),
};

function json(res: ServerResponse, status: number, body: unknown): void {
  res.writeHead(status, { "Content-Type": "application/json" });
  res.end(JSON.stringify(body));
}

function authorizeInbound(req: IncomingMessage, token: string | undefined): boolean {
  if (!token) {
    return true;
  }
  const bearer = req.headers.authorization?.match(/^Bearer\s+(.+)$/i)?.[1];
  const headerToken = Array.isArray(req.headers["x-presence-token"])
    ? req.headers["x-presence-token"][0]
    : req.headers["x-presence-token"];
  return bearer === token || headerToken === token;
}

async function readJson(req: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(typeof chunk === "string" ? Buffer.from(chunk) : chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

function isStringMap(value: unknown): value is Record<string, string> {
  return Boolean(
    value &&
      typeof value === "object" &&
      !Array.isArray(value) &&
      Object.values(value).every((item) => typeof item === "string"),
  );
}

function validateInbound(payload: unknown): PresenceInbound {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    throw new Error("payload must be an object");
  }
  const event = payload as Partial<PresenceInbound>;
  if (event.schema !== "cto.presence.v1") throw new Error("schema must be cto.presence.v1");
  if (event.runtime !== "hermes") throw new Error("runtime must be hermes");
  if (!event.agent_id) throw new Error("agent_id is required");
  if (!event.discord?.account_id || !event.discord.channel_id) {
    throw new Error("discord.account_id and discord.channel_id are required");
  }
  if (event.metadata !== undefined && !isStringMap(event.metadata)) {
    throw new Error("metadata must be a string map");
  }
  return event as PresenceInbound;
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function createAdapterServer(config: AdapterConfig): ReturnType<typeof createServer> {
  return createServer((req, res) => {
    void (async () => {
      const method = req.method?.toUpperCase();
      const url = new URL(req.url ?? "/", "http://localhost");
      if (method === "GET" && url.pathname === "/health") {
        json(res, 200, { status: "ok", service: "hermes-presence-adapter" });
        return;
      }
      if (method !== "POST" || url.pathname !== "/presence/inbound") {
        json(res, 404, { error: "Not found" });
        return;
      }
      if (!authorizeInbound(req, config.presenceSharedToken)) {
        json(res, 401, { error: "Unauthorized" });
        return;
      }

      let event: PresenceInbound;
      try {
        event = validateInbound(await readJson(req));
      } catch (err) {
        json(res, 400, { error: String(err instanceof Error ? err.message : err) });
        return;
      }

      try {
        await postPresenceStatus(config.presenceRouterUrl, config.presenceSharedToken, event, "started").catch((err) => {
          logger.warn(`Presence status post failed: ${err}`);
        });
        const run = config.hermesInputUrl
          ? await postHermesInput(config.hermesInputUrl, event).catch((err) => {
              logger.warn(`Hermes input endpoint unavailable; falling back to inbox: ${err}`);
              return appendInbox(config.inboxPath, event);
            })
          : config.hermesApiUrl
            ? await startHermesRun(config.hermesApiUrl, event)
            : await appendInbox(config.inboxPath, event);
        await postPresenceStatus(config.presenceRouterUrl, config.presenceSharedToken, event, "running", run.id ?? run.run_id).catch((err) => {
          logger.warn(`Presence status post failed: ${err}`);
        });
        json(res, 202, { accepted: true, run });
      } catch (err) {
        const message = String(err instanceof Error ? err.message : err);
        await postPresenceStatus(config.presenceRouterUrl, config.presenceSharedToken, event, "failed", message).catch(() => undefined);
        json(res, 502, { error: message });
      }
    })().catch((err) => {
      logger.error("Request failed:", err);
      if (!res.headersSent) json(res, 500, { error: "Internal server error" });
    });
  });
}

async function main(): Promise<void> {
  const config = loadConfig();
  if (config.hermesInputUrl) logger.info(`Hermes input endpoint: ${config.hermesInputUrl}`);
  if (config.hermesApiUrl) logger.info(`Hermes API: ${config.hermesApiUrl}`);
  logger.info(`Inbox path: ${config.inboxPath}`);
  if (config.presenceRouterUrl) logger.info(`Presence router: ${config.presenceRouterUrl}`);
  let shuttingDown = false;

  const server = createAdapterServer(config);

  await new Promise<void>((resolve) => server.listen(config.port, resolve));
  logger.info(`HTTP server listening on ${config.port}`);

  const route = config.route;
  if (route) {
    void (async () => {
      while (!shuttingDown) {
        try {
          await registerPresenceRoute(config.presenceRouterUrl, config.presenceSharedToken, route);
          logger.info(`Registered presence route ${route.route_id} -> ${route.worker_url}`);
          return;
        } catch (err) {
          logger.warn(`Presence route registration failed; retrying: ${err}`);
          await delay(5_000);
        }
      }
    })();
  }

  const shutdown = () => {
    if (shuttingDown) {
      return;
    }
    shuttingDown = true;
    void (async () => {
      await deletePresenceRoute(config.presenceRouterUrl, config.presenceSharedToken, config.route?.route_id).catch((err) => {
        logger.warn(`Presence route cleanup failed: ${err}`);
      });
      server.close(() => process.exit(0));
    })();
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  main().catch((err) => {
    logger.error("Fatal error:", err);
    process.exit(1);
  });
}
