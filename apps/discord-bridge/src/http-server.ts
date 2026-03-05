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
import type { DiscordElicitationHandler } from './elicitation-handler.js';
import type { AgentMessage } from './types.js';
import type { ElicitationRequest, ElicitationCancel } from './elicitation-types.js';

export interface HttpServer {
  start(): Promise<void>;
  stop(): Promise<void>;
}

export function createHttpServer(
  port: number,
  bridge: Bridge,
  elicitHandler: DiscordElicitationHandler | undefined,
  logger: { info: Function; warn: Function; error: Function },
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
    const url = req.url;

    // Health check
    if (method === 'GET' && url === '/health') {
      json(res, 200, { status: 'ok', service: 'discord-bridge' });
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
