import { setTimeout } from 'node:timers/promises';

const STITCH_MCP_URL = 'https://stitch.googleapis.com/mcp';
const DEFAULT_TIMEOUT_MS = 30_000;
const GENERATION_TIMEOUT_MS = 180_000;
const GENERATION_MAX_RETRIES = 2;
const GENERATION_TOOLS = new Set(['generate_screen_from_text', 'generate_variants', 'edit_screens']);

function readPositiveIntEnv(name: string, fallback: number): number {
  const raw = process.env[name]?.trim();
  if (!raw) {
    return fallback;
  }

  const parsed = Number.parseInt(raw, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return fallback;
  }

  return parsed;
}

interface McpResponse {
  jsonrpc: '2.0';
  id: number;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

export class StitchDirectClient {
  private apiKey: string;
  private sessionId: string | null = null;
  private nextId = 1;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  private buildHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json, text/event-stream',
      'X-Goog-Api-Key': this.apiKey,
    };
    if (this.sessionId) {
      headers['Mcp-Session-Id'] = this.sessionId;
    }
    return headers;
  }

  private async rpc(method: string, params: Record<string, unknown>, timeoutMs?: number): Promise<unknown> {
    const id = this.nextId++;
    const body = JSON.stringify({ jsonrpc: '2.0', id, method, params });
    const effectiveTimeout = timeoutMs ?? DEFAULT_TIMEOUT_MS;

    const controller = new AbortController();
    const timer = setTimeout(effectiveTimeout).then(() => {
      controller.abort();
      throw new Error(`Stitch MCP call "${method}" timed out after ${effectiveTimeout / 1000}s`);
    });

    try {
      const response = await Promise.race([
        fetch(STITCH_MCP_URL, {
          method: 'POST',
          headers: this.buildHeaders(),
          body,
          signal: controller.signal,
        }),
        timer,
      ]) as Response;

      const sid = response.headers.get('Mcp-Session-Id');
      if (sid) {
        this.sessionId = sid;
      }

      const contentType = response.headers.get('content-type') ?? '';
      let parsed: McpResponse;

      if (contentType.includes('text/event-stream')) {
        const text = await response.text();
        const dataLines = text.split('\n').filter(l => l.startsWith('data: '));
        const lastData = dataLines[dataLines.length - 1];
        if (!lastData) {
          throw new Error(`Stitch MCP SSE response had no data lines for "${method}"`);
        }
        parsed = JSON.parse(lastData.slice(6)) as McpResponse;
      } else {
        parsed = (await response.json()) as McpResponse;
      }

      controller.abort();

      if (parsed.error) {
        throw new Error(`Stitch MCP error [${parsed.error.code}]: ${parsed.error.message}`);
      }

      return parsed.result;
    } catch (err) {
      controller.abort();
      throw err;
    }
  }

  async connect(): Promise<void> {
    await this.rpc('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'intake-agent-stitch', version: '1.0.0' },
    });
  }

  async callTool(name: string, args: Record<string, unknown>): Promise<unknown> {
    if (!this.sessionId) {
      await this.connect();
    }

    const isGeneration = GENERATION_TOOLS.has(name);
    const timeoutMs = isGeneration
      ? readPositiveIntEnv('INTAKE_STITCH_GENERATION_TIMEOUT_MS', GENERATION_TIMEOUT_MS)
      : readPositiveIntEnv('INTAKE_STITCH_DEFAULT_TIMEOUT_MS', DEFAULT_TIMEOUT_MS);
    const maxAttempts = isGeneration
      ? readPositiveIntEnv('INTAKE_STITCH_GENERATION_MAX_RETRIES', GENERATION_MAX_RETRIES)
      : 1;
    let lastError: Error | undefined;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        console.error(
          `[stitch] ${name} attempt ${attempt}/${maxAttempts} starting (timeout=${Math.round(timeoutMs / 1000)}s)`,
        );
        const result = (await this.rpc('tools/call', { name, arguments: args }, timeoutMs)) as {
          content?: Array<{ type: string; text?: string }>;
          structuredContent?: unknown;
          isError?: boolean;
        };

        if (result?.isError) {
          const errorText = result.content?.map(c => c.type === 'text' ? c.text : '').join('') ?? 'unknown';
          throw new Error(`Stitch tool "${name}" failed: ${errorText}`);
        }

        if (result?.structuredContent) {
          return result.structuredContent;
        }

        const textContent = result?.content?.find(c => c.type === 'text');
        if (textContent?.text) {
          try {
            return JSON.parse(textContent.text);
          } catch {
            return textContent.text;
          }
        }

        return result;
      } catch (err) {
        lastError = err instanceof Error ? err : new Error(String(err));
        console.error(`[stitch] ${name} attempt ${attempt}/${maxAttempts} failed: ${lastError.message}`);
        if (attempt < maxAttempts) {
          const backoff = attempt * 10_000;
          console.error(`[stitch] ${name} attempt ${attempt} failed: ${lastError.message} — retrying in ${backoff / 1000}s`);
          await setTimeout(backoff);
          if (!this.sessionId) await this.connect();
        }
      }
    }

    throw lastError!;
  }

  async close(): Promise<void> {
    this.sessionId = null;
  }
}
