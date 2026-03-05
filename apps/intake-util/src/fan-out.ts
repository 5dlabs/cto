/**
 * fan-out.ts — Bounded-concurrency parallel LLM invocation.
 *
 * Spawns one `openclaw.invoke --tool llm-task --action json` per item,
 * using a semaphore to cap parallelism. Retries failed items twice
 * with exponential backoff. Merges results into a single array.
 */

export interface FanOutOpts {
  items: unknown[];
  promptPath: string;
  schemaPath: string;
  context: Record<string, unknown>;
  provider: string;
  model: string;
  concurrency: number;
}

export interface FanOutResult<T = unknown> {
  results: T[];
  failures: { index: number; task_id?: number; error: string }[];
}

class Semaphore {
  private queue: (() => void)[] = [];
  private running = 0;

  constructor(private max: number) {}

  async acquire(): Promise<void> {
    if (this.running < this.max) {
      this.running++;
      return;
    }
    return new Promise<void>((resolve) => {
      this.queue.push(() => {
        this.running++;
        resolve();
      });
    });
  }

  release(): void {
    this.running--;
    const next = this.queue.shift();
    if (next) next();
  }
}

async function invokeOne(
  item: unknown,
  promptPath: string,
  schemaPath: string,
  context: Record<string, unknown>,
  provider: string,
  model: string,
): Promise<unknown> {
  const input = { ...context, task: item };

  const argsJson = JSON.stringify({
    prompt: `{{${promptPath}}}`,
    input,
    schema: `{{${schemaPath}}}`,
    provider,
    model,
  });

  const proc = Bun.spawn(
    ['openclaw.invoke', '--tool', 'llm-task', '--action', 'json', '--args-json', argsJson],
    { stdout: 'pipe', stderr: 'pipe' },
  );

  const [stdout, stderr] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);

  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(stderr.trim() || `openclaw.invoke exited with code ${exitCode}`);
  }

  const trimmed = stdout.trim();
  if (!trimmed) {
    throw new Error('Empty response from openclaw.invoke');
  }

  return JSON.parse(trimmed);
}

async function invokeWithRetry(
  item: unknown,
  promptPath: string,
  schemaPath: string,
  context: Record<string, unknown>,
  provider: string,
  model: string,
  maxRetries: number = 2,
): Promise<unknown> {
  let lastError: Error | undefined;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await invokeOne(item, promptPath, schemaPath, context, provider, model);
    } catch (err) {
      lastError = err instanceof Error ? err : new Error(String(err));
      if (attempt < maxRetries) {
        const delay = Math.pow(2, attempt) * 1000; // 1s, 2s
        await Bun.sleep(delay);
      }
    }
  }

  throw lastError!;
}

export async function fanOut(opts: FanOutOpts): Promise<FanOutResult> {
  const { items, promptPath, schemaPath, context, provider, model, concurrency } = opts;
  const sem = new Semaphore(concurrency);

  const settled = await Promise.allSettled(
    items.map(async (item, index) => {
      await sem.acquire();
      try {
        const result = await invokeWithRetry(item, promptPath, schemaPath, context, provider, model);
        return { index, result };
      } finally {
        sem.release();
      }
    }),
  );

  const results: unknown[] = [];
  const failures: FanOutResult['failures'] = [];

  for (const entry of settled) {
    if (entry.status === 'fulfilled') {
      results.push(entry.value.result);
    } else {
      const reason = entry.reason;
      // Extract index from the error context — settled array preserves order
      const idx = settled.indexOf(entry);
      const taskId = (items[idx] as { id?: number })?.id;
      failures.push({
        index: idx,
        task_id: taskId,
        error: reason?.message || String(reason),
      });
    }
  }

  return { results, failures };
}
