/**
 * run-registry-client — Register/deregister pipeline runs with linear-bridge.
 */

interface RegisterRunArgs {
  runId: string;
  agent: string;
  sessionKey?: string;
  issueId?: string;
  linearSessionId?: string;
}

function skipRegisterRun(): boolean {
  const v = process.env['INTAKE_REGISTER_RUN_SKIP'];
  return v === '1' || v?.toLowerCase() === 'true';
}

function skipDeregisterRun(): boolean {
  const v = process.env['INTAKE_DEREGISTER_RUN_SKIP'];
  return v === '1' || v?.toLowerCase() === 'true';
}

function registerBody(args: RegisterRunArgs): string {
  const entries: Record<string, string> = { agent: args.agent };
  if (args.sessionKey) entries['sessionKey'] = args.sessionKey;
  if (args.issueId) entries['issueId'] = args.issueId;
  if (args.linearSessionId) entries['linearSessionId'] = args.linearSessionId;
  return JSON.stringify(entries);
}

export async function registerRun(args: RegisterRunArgs): Promise<boolean> {
  if (skipRegisterRun()) {
    console.warn(
      '[register-run] Skipped (INTAKE_REGISTER_RUN_SKIP set); run will not appear in linear-bridge registry',
    );
    return true;
  }
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';
  try {
    const res = await fetch(`${linearUrl}/runs/${args.runId}/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: registerBody(args),
    });
    return res.ok;
  } catch {
    console.error('[register-run] Failed to register run');
    return false;
  }
}

export async function deregisterRun(runId: string): Promise<boolean> {
  if (skipDeregisterRun()) {
    console.warn('[deregister-run] Skipped (INTAKE_DEREGISTER_RUN_SKIP set)');
    return true;
  }
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';
  try {
    const res = await fetch(`${linearUrl}/runs/${runId}`, { method: 'DELETE' });
    return res.ok;
  } catch {
    console.error('[deregister-run] Failed to deregister run');
    return false;
  }
}
