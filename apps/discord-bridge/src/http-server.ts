/**
 * HTTP Notification API — Discord Bridge
 *
 * Lightweight HTTP server (port 3200) that receives notifications and
 * elicitation requests from workflow steps via HTTP POST, replacing NATS.
 *
 * Endpoints:
 *   POST /notify              — Agent message notification
 *   POST /elicitation         — Elicitation request (renders buttons/select menus)
 *   POST /elicitation/cancel  — Cancel pending elicitation (answered elsewhere)
 *   GET  /health              — Health check
 */

import { createServer, type Server, type IncomingMessage, type ServerResponse } from 'node:http';
import type { Bridge } from './bridge.js';
import type { DiscordHandle } from './discord-client.js';
import type { DiscordElicitationHandler } from './elicitation-handler.js';
import type { AgentMessage } from './types.js';
import type { ElicitationRequest, ElicitationCancel, DesignReviewRequest } from './elicitation-types.js';

export interface HttpServer {
  start(): Promise<void>;
  stop(): Promise<void>;
}

export function createHttpServer(
  port: number,
  bridge: Bridge,
  elicitHandler: DiscordElicitationHandler | undefined,
  logger: { info: Function; warn: Function; error: Function },
  defaultChannelId?: string,
  discord?: DiscordHandle,
): HttpServer {
  let server: Server | undefined;

  async function readBody(req: IncomingMessage): Promise<Buffer> {
    const chunks: Buffer[] = [];
    for await (const chunk of req) {
      chunks.push(typeof chunk === 'string' ? Buffer.from(chunk) : chunk);
    }
    return Buffer.concat(chunks);
  }

  function json(res: ServerResponse, status: number, data: unknown): void {
    res.writeHead(status, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(data));
  }

  async function handleRequest(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const method = req.method?.toUpperCase();
    const rawUrl = req.url ?? "/";
    const parsedUrl = new URL(rawUrl, "http://localhost");
    const url = parsedUrl.pathname;
    const statusMatch = url?.match(/^\/elicitation\/status\/([^/]+)$/);

    // Health check
    if (method === 'GET' && url === '/health') {
      json(res, 200, { status: 'ok', service: 'discord-bridge' });
      return;
    }

    if (method === 'GET' && url === '/history/decisions') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const sessionId = parsedUrl.searchParams.get("session_id") ?? undefined;
      const limitRaw = parsedUrl.searchParams.get("limit");
      const limit = limitRaw ? Number.parseInt(limitRaw, 10) : 100;
      const rows = elicitHandler.getDecisionHistory(sessionId, Number.isFinite(limit) ? Math.max(1, Math.min(limit, 1000)) : 100);
      json(res, 200, { decisions: rows });
      return;
    }

    if (method === 'GET' && url === '/history/sessions') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const limitRaw = parsedUrl.searchParams.get("limit");
      const status = parsedUrl.searchParams.get("status") ?? undefined;
      const limit = limitRaw ? Number.parseInt(limitRaw, 10) : 200;
      const rows = elicitHandler.getSessionHistory(Number.isFinite(limit) ? Math.max(1, Math.min(limit, 1000)) : 200, status);
      json(res, 200, { sessions: rows });
      return;
    }

    if (method === 'GET' && url === '/history/waiting') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const limitRaw = parsedUrl.searchParams.get("limit");
      const limit = limitRaw ? Number.parseInt(limitRaw, 10) : 200;
      const rows = elicitHandler.getWaitingSessions(Number.isFinite(limit) ? Math.max(1, Math.min(limit, 1000)) : 200);
      json(res, 200, { waiting: rows });
      return;
    }

    if (method === 'GET' && url === '/history/decision-audit') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const elicitationId = parsedUrl.searchParams.get("elicitation_id");
      if (!elicitationId) {
        json(res, 400, { error: 'Missing query param: elicitation_id' });
        return;
      }
      const bridge = parsedUrl.searchParams.get("bridge") ?? undefined;
      const audit = elicitHandler.getDecisionAudit(elicitationId, bridge);
      json(res, 200, { audit });
      return;
    }

    if (method === 'GET' && url === '/history/design') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const sessionId = parsedUrl.searchParams.get("session_id") ?? undefined;
      const limitRaw = parsedUrl.searchParams.get("limit");
      const limit = limitRaw ? Number.parseInt(limitRaw, 10) : 50;
      const rows = elicitHandler.getDesignHistory(sessionId, Number.isFinite(limit) ? Math.max(1, Math.min(limit, 1000)) : 50);
      json(res, 200, { design: rows });
      return;
    }

    if (method === 'GET' && statusMatch) {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const elicitationId = decodeURIComponent(statusMatch[1] ?? '');
      if (!elicitationId) {
        json(res, 400, { error: 'Missing elicitation id' });
        return;
      }
      const status = elicitHandler.getStatus(elicitationId);
      json(res, 200, status);
      return;
    }

    if (method !== 'POST') {
      json(res, 405, { error: 'Method not allowed' });
      return;
    }

    const body = await readBody(req);
    let payload: unknown;
    try {
      payload = JSON.parse(body.toString('utf-8'));
    } catch {
      json(res, 400, { error: 'Invalid JSON' });
      return;
    }

    switch (url) {
      case '/notify': {
        const msg = payload as AgentMessage;
        if (!msg.from || !msg.message) {
          json(res, 400, { error: 'Missing required fields: from, message' });
          return;
        }
        // Fill defaults
        msg.priority = msg.priority ?? 'normal';
        msg.timestamp = msg.timestamp ?? new Date().toISOString();
        msg.subject = msg.subject ?? `agent.${msg.to ?? 'broadcast'}.inbox`;

        bridge.handleMessage(msg.subject, msg);
        json(res, 200, { received: true });
        return;
      }

      case '/elicitation': {
        if (!elicitHandler) {
          json(res, 503, { error: 'Elicitation handler not initialized' });
          return;
        }
        const request = payload as ElicitationRequest;
        if (!request.elicitation_id || !request.question) {
          json(res, 400, { error: 'Missing required fields: elicitation_id, question' });
          return;
        }
        try {
          if (request.session_id && defaultChannelId && discord) {
            const sessionThreadId = await discord.getOrCreateSessionThread(
              defaultChannelId,
              request.session_id,
            );
            // Keep deliberation chatter in the session thread, but render the
            // interactive pros/cons decision card in the main intake channel.
            request.discord_channel_id = defaultChannelId;
            request.metadata = {
              ...(request.metadata ?? {}),
              main_channel_id: defaultChannelId,
              session_thread_id: sessionThreadId,
            };
          } else if (!request.discord_channel_id && defaultChannelId) {
            request.discord_channel_id = defaultChannelId;
          }
        } catch (err) {
          logger.warn(`Session thread lookup for elicitation failed; using main channel card: ${err}`);
          if (defaultChannelId) request.discord_channel_id = defaultChannelId;
        }
        // Handle async — respond immediately
        json(res, 200, { received: true, elicitation_id: request.elicitation_id });
        elicitHandler.handleRequest(request).catch((err) => {
          logger.error(`Failed to handle elicitation request: ${err}`);
        });
        return;
      }

      case '/elicitation/cancel': {
        if (!elicitHandler) {
          json(res, 503, { error: 'Elicitation handler not initialized' });
          return;
        }
        const cancel = payload as ElicitationCancel;
        if (!cancel.elicitation_id) {
          json(res, 400, { error: 'Missing required field: elicitation_id' });
          return;
        }
        json(res, 200, { received: true });
        elicitHandler.handleCancel(cancel).catch((err) => {
          logger.error(`Failed to handle elicitation cancel: ${err}`);
        });
        return;
      }

      case '/design-review': {
        if (!elicitHandler) {
          json(res, 503, { error: 'Elicitation handler not initialized' });
          return;
        }
        const review = payload as DesignReviewRequest;
        if (!review.review_id || !review.variants?.length) {
          json(res, 400, { error: 'Missing required fields: review_id, variants' });
          return;
        }
        if (!review.discord_channel_id && defaultChannelId) {
          review.discord_channel_id = defaultChannelId;
        }
        json(res, 200, { received: true, review_id: review.review_id });
        elicitHandler.handleDesignReview(review).catch((err) => {
          logger.error(`Failed to handle design review: ${err}`);
        });
        return;
      }

      case '/history/design-snapshot': {
        if (!elicitHandler) {
          json(res, 503, { error: 'Elicitation handler not initialized' });
          return;
        }
        const bodyObj = payload as Record<string, unknown>;
        const sessionId = bodyObj["session_id"];
        if (typeof sessionId !== "string" || !sessionId) {
          json(res, 400, { error: 'Missing required field: session_id' });
          return;
        }
        elicitHandler.recordDesignSnapshot(bodyObj);
        json(res, 200, { received: true, session_id: sessionId });
        return;
      }

      default:
        json(res, 404, { error: 'Not found' });
    }
  }

  return {
    start() {
      return new Promise<void>((resolve, reject) => {
        server = createServer((req, res) => {
          handleRequest(req, res).catch((err) => {
            logger.error('HTTP request handler error:', err);
            if (!res.headersSent) {
              json(res, 500, { error: 'Internal server error' });
            }
          });
        });
        server.on('error', reject);
        server.listen(port, () => {
          logger.info(`HTTP server listening on port ${port}`);
          resolve();
        });
      });
    },

    stop() {
      return new Promise<void>((resolve) => {
        if (!server) {
          resolve();
          return;
        }
        server.close(() => {
          logger.info('HTTP server stopped');
          resolve();
        });
      });
    },
  };
}
