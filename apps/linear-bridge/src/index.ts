import { loadConfig } from "./config.js";
import { createLinearClient } from "./linear-client.js";
import { IssueManager } from "./issue-manager.js";
import { createBridge } from "./bridge.js";
import { createHttpServer } from "./http-server.js";
import { createAgentSessionManager } from "./agent-session-manager.js";
import { createElicitationHandler } from "./elicitation-handler.js";
import { createRunRegistry } from "./run-registry.js";
import { LokiActivityStream } from "./loki-activity-stream.js";

const logger = {
  info: (...args: unknown[]) => console.log(`[linear-bridge]`, ...args),
  warn: (...args: unknown[]) => console.warn(`[linear-bridge]`, ...args),
  error: (...args: unknown[]) => console.error(`[linear-bridge]`, ...args),
};

async function main(): Promise<void> {
  const config = loadConfig();
  logger.info("Starting Linear bridge...");
  logger.info(`  Team ID: ${config.linearTeamId}`);
  logger.info(`  HTTP port: ${config.httpPort}`);
  logger.info(`  Discord bridge: ${config.discordBridgeUrl}`);
  logger.info(`  Inactivity timeout: ${config.inactivityTimeoutMs}ms`);
  logger.info(`  Agent sessions: ${config.agentSessionsEnabled ? "enabled" : "disabled"}`);
  logger.info(`  ACP activity stream: ${config.acpActivityEnabled ? "enabled" : "disabled"}`);
  if (config.acpActivityEnabled) {
    logger.info(`  Loki URL: ${config.lokiUrl}`);
    logger.info(`  Loki org: ${config.lokiOrgId}`);
    logger.info(`  Loki poll interval: ${config.lokiPollIntervalMs}ms`);
  }
  if (config.defaultProjectId) {
    logger.info(`  Default project: ${config.defaultProjectId}`);
  }

  // 1. Initialize Linear client
  const linearClient = createLinearClient(config.linearApiKey, logger);

  // 2. Set up issue manager
  const issueManager = new IssueManager(
    linearClient,
    config.linearTeamId,
    config.defaultProjectId,
    logger,
  );

  // 3. Set up agent session manager + run registry
  const sessionManager = createAgentSessionManager();
  const runRegistry = createRunRegistry();

  // 4. Create the bridge orchestrator
  const bridge = createBridge(config, linearClient, issueManager, logger);

  // 5. Create elicitation handler (HTTP-based cross-cancel to Discord bridge)
  let elicitHandler: ReturnType<typeof createElicitationHandler> | undefined;
  if (config.agentSessionsEnabled) {
    elicitHandler = createElicitationHandler(
      linearClient,
      sessionManager,
      runRegistry,
      config.discordBridgeUrl,
      logger,
    );
  }

  // 6. Wire up Loki activity stream
  let lokiStream: LokiActivityStream | undefined;
  if (config.acpActivityEnabled) {
    lokiStream = new LokiActivityStream(
      {
        lokiUrl: config.lokiUrl,
        lokiOrgId: config.lokiOrgId,
        pollIntervalMs: config.lokiPollIntervalMs,
      },
      linearClient,
      runRegistry,
      logger,
    );
  }

  // 7. Start HTTP server (webhooks + notifications + run management)
  const httpServer = createHttpServer(
    config.httpPort,
    config.linearWebhookSecret,
    bridge,
    elicitHandler,
    runRegistry,
    (event) => {
      elicitHandler?.handleWebhookEvent(event).catch((err) => {
        logger.error("Failed to handle webhook event:", err);
      });
    },
    logger,
  );
  await httpServer.start();

  // Start Loki activity stream after server is up
  lokiStream?.start();

  // 9. Schedule GC (every 60s)
  let gcInterval: ReturnType<typeof setInterval> | undefined;
  if (config.agentSessionsEnabled) {
    gcInterval = setInterval(() => {
      const sessionRemoved = sessionManager.gc(config.inactivityTimeoutMs);
      const runRemoved = runRegistry.gc(config.inactivityTimeoutMs);
      if (sessionRemoved > 0 || runRemoved > 0) {
        logger.info(`GC: removed ${sessionRemoved} stale session(s), ${runRemoved} stale run(s)`);
      }
    }, 60_000);
  }

  logger.info("Linear bridge is running");

  // Graceful shutdown
  const shutdown = async () => {
    logger.info("Shutting down...");
    if (gcInterval) clearInterval(gcInterval);
    lokiStream?.stop();
    bridge.stop();
    elicitHandler?.destroy();
    await httpServer.stop();
    process.exit(0);
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

main().catch((err) => {
  logger.error("Fatal error:", err);
  process.exit(1);
});
