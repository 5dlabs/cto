/**
 * HTTP Server — Linear Bridge
 *
 * Full HTTP router replacing the old webhook-only server.
 * Handles Linear webhooks, run registration, notifications, and elicitations.
 *
 * Endpoints:
 *   POST /webhooks/linear              — Linear Agent Session webhooks
 *   POST /runs/:runId/callback         — Dynamic per-run callback (Linear webhook target)
 *   POST /runs/:runId/register         — Register run → {agent, sessionKey, issueId}
 *   DELETE /runs/:runId                — Deregister run
 *   POST /notify                       — Agent message → Linear comment
 *   POST /elicitation                  — Elicitation request → Linear select signal
 *   POST /elicitation/cancel           — Cancel (answered on Discord)
 *   GET  /health                       — Health check
 */

import { createServer, type Server, type IncomingMessage, type ServerResponse } from 'node:http';
import { createHmac, timingSafeEqual } from 'node:crypto';
import type { Bridge } from './bridge.js';
import type { ElicitationHandler } from './elicitation-handler.js';
import type { RunRegistry } from './run-registry.js';
import type { AgentMessage } from './types.js';
import type { ElicitationRequest, ElicitationCancel, DesignReviewRequest } from './elicitation-types.js';

export interface AgentSessionWebhookEvent {
  action: 'created' | 'prompted' | 'updated';
  type: 'AgentSession';
  data: {
    id: string;
    // Legacy shortcut used by older webhook payloads for select choices.
    promptedValue?: string;
    // Per Linear docs, prompted webhook carries user message in agentActivity.body.
    agentActivity?: {
      id?: string;
      body?: string;
      signal?: string;
      signalMetadata?: unknown;
      userId?: string;
      [key: string]: unknown;
    };
    body?: string;
    userId?: string;
    [key: string]: unknown;
  };
  createdAt: string;
}

export type WebhookEventHandler = (event: AgentSessionWebhookEvent) => void;

export interface HttpServer {
  start(): Promise<void>;
  stop(): Promise<void>;
}

export function createHttpServer(
  port: number,
  webhookSecret: string | undefined,
  bridge: Bridge,
  elicitHandler: ElicitationHandler | undefined,
  runRegistry: RunRegistry,
  onWebhookEvent: WebhookEventHandler,
  logger: { info: Function; warn: Function; error: Function },
): HttpServer {
  let server: Server | undefined;

  function verifySignature(body: Buffer, signature: string | undefined): boolean {
    if (!webhookSecret) return true;
    if (!signature) return false;
    const expected = createHmac('sha256', webhookSecret).update(body).digest('hex');
    const sig = signature.replace(/^sha256=/, '');
    if (expected.length !== sig.length) return false;
    return timingSafeEqual(Buffer.from(expected, 'hex'), Buffer.from(sig, 'hex'));
  }

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

  /** Extract path parameters: /runs/:runId/... */
  function matchRunPath(url: string): { runId: string; rest: string } | null {
    const match = url.match(/^\/runs\/([^/]+)(\/.*)?$/);
    if (!match) return null;
    return { runId: match[1], rest: match[2] ?? '' };
  }

  async function handleRequest(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const method = req.method?.toUpperCase();
    const rawUrl = req.url ?? '/';
    const parsedUrl = new URL(rawUrl, 'http://localhost');
    const url = parsedUrl.pathname;
    const statusMatch = url.match(/^\/elicitation\/status\/([^/]+)$/);

    // Health check
    if (method === 'GET' && url === '/health') {
      json(res, 200, { status: 'ok', service: 'linear-bridge', runs: runRegistry.size() });
      return;
    }

    if (method === 'GET' && url === '/history/decisions') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const sessionId = parsedUrl.searchParams.get('session_id') ?? undefined;
      const limitRaw = parsedUrl.searchParams.get('limit');
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
      const limitRaw = parsedUrl.searchParams.get('limit');
      const status = parsedUrl.searchParams.get('status') ?? undefined;
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
      const limitRaw = parsedUrl.searchParams.get('limit');
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
      const elicitationId = parsedUrl.searchParams.get('elicitation_id');
      if (!elicitationId) {
        json(res, 400, { error: 'Missing query param: elicitation_id' });
        return;
      }
      const bridge = parsedUrl.searchParams.get('bridge') ?? undefined;
      const audit = elicitHandler.getDecisionAudit(elicitationId, bridge);
      json(res, 200, { audit });
      return;
    }

    if (method === 'GET' && url === '/history/design') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const sessionId = parsedUrl.searchParams.get('session_id') ?? undefined;
      const limitRaw = parsedUrl.searchParams.get('limit');
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
      json(res, 200, elicitHandler.getStatus(elicitationId));
      return;
    }

    // DELETE /runs/:runId
    const runMatch = matchRunPath(url);
    if (method === 'DELETE' && runMatch && runMatch.rest === '') {
      runRegistry.deregister(runMatch.runId);
      json(res, 200, { deregistered: true, runId: runMatch.runId });
      return;
    }

    if (method !== 'POST') {
      json(res, 405, { error: 'Method not allowed' });
      return;
    }

    const body = await readBody(req);

    // Linear webhook — needs signature verification before JSON parse
    if (url === '/webhooks/linear') {
      const signature = req.headers['linear-signature'] as string | undefined
        ?? req.headers['x-linear-signature'] as string | undefined;

      if (!verifySignature(body, signature)) {
        logger.warn('Webhook signature verification failed');
        json(res, 401, { error: 'Unauthorized' });
        return;
      }

      let event: AgentSessionWebhookEvent;
      try {
        event = JSON.parse(body.toString('utf-8'));
      } catch {
        json(res, 400, { error: 'Bad request' });
        return;
      }

      // Must respond 200 within 10 seconds per Linear docs
      json(res, 200, { ok: true });

      try {
        onWebhookEvent(event);
      } catch (err) {
        logger.error('Webhook event handler error:', err);
      }
      return;
    }

    // All other POST endpoints need JSON body
    let payload: unknown;
    try {
      payload = JSON.parse(body.toString('utf-8'));
    } catch {
      json(res, 400, { error: 'Invalid JSON' });
      return;
    }

    // POST /runs/:runId/register
    if (runMatch && runMatch.rest === '/register') {
      const data = payload as {
        agent?: string;
        sessionKey?: string;
        issueId?: string;
        linearSessionId?: string;
        resumeToken?: string;
      };
      runRegistry.register(runMatch.runId, {
        agentPod: data.agent ?? '',
        sessionKey: data.sessionKey ?? '',
        issueId: data.issueId ?? '',
        linearSessionId: data.linearSessionId,
        resumeToken: data.resumeToken,
      });
      json(res, 200, { registered: true, runId: runMatch.runId });
      return;
    }

    // POST /runs/:runId/callback — Dynamic callback for Linear webhooks per run
    if (runMatch && runMatch.rest === '/callback') {
      const run = runRegistry.lookup(runMatch.runId);
      if (!run) {
        json(res, 404, { error: `Run ${runMatch.runId} not found` });
        return;
      }

      // This is used by Linear webhook → Lobster resume
      json(res, 200, { received: true, runId: runMatch.runId });

      // If there's a resume token and the payload has response data, trigger Lobster resume
      if (run.resumeToken) {
        const callbackData = payload as Record<string, unknown>;
        logger.info(`Run ${runMatch.runId}: callback received, resumeToken present`);
        // The elicitation handler will process this via the run registry
        elicitHandler?.handleRunCallback(runMatch.runId, callbackData).catch((err) => {
          logger.error(`Failed to handle run callback: ${err}`);
        });
      }
      return;
    }

    // POST /notify
    if (url === '/notify') {
      const msg = payload as AgentMessage;
      if (!msg.from || !msg.message) {
        json(res, 400, { error: 'Missing required fields: from, message' });
        return;
      }
      msg.priority = msg.priority ?? 'normal';
      msg.timestamp = msg.timestamp ?? new Date().toISOString();
      msg.subject = msg.subject ?? `agent.${msg.to ?? 'broadcast'}.inbox`;

      bridge.handleMessage(msg.subject, msg);
      json(res, 200, { received: true });
      return;
    }

    // POST /elicitation
    if (url === '/elicitation') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const request = payload as ElicitationRequest;
      if (!request.elicitation_id || !request.question) {
        json(res, 400, { error: 'Missing required fields: elicitation_id, question' });
        return;
      }
      json(res, 200, { received: true, elicitation_id: request.elicitation_id });
      elicitHandler.handleRequest(request).catch((err) => {
        logger.error(`Failed to handle elicitation request: ${err}`);
      });
      return;
    }

    // POST /elicitation/cancel
    if (url === '/elicitation/cancel') {
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

    // POST /design-review — Design variant selection via Linear
    if (url === '/design-review') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const review = payload as DesignReviewRequest;
      if (!review.review_id || !review.variants?.length) {
        json(res, 400, { error: 'Missing required fields: review_id, variants' });
        return;
      }
      json(res, 200, { received: true, review_id: review.review_id });
      elicitHandler.handleDesignReview(review).catch((err) => {
        logger.error(`Failed to handle design review: ${err}`);
      });
      return;
    }

    // POST /history/design-snapshot
    if (url === '/history/design-snapshot') {
      if (!elicitHandler) {
        json(res, 503, { error: 'Elicitation handler not initialized' });
        return;
      }
      const bodyObj = payload as Record<string, unknown>;
      const sessionId = bodyObj['session_id'];
      if (typeof sessionId !== 'string' || !sessionId) {
        json(res, 400, { error: 'Missing required field: session_id' });
        return;
      }
      elicitHandler.recordDesignSnapshot(bodyObj);
      json(res, 200, { received: true, session_id: sessionId });
      return;
    }

    json(res, 404, { error: 'Not found' });
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
