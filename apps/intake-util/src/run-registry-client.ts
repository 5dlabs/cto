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

export async function registerRun(args: RegisterRunArgs): Promise<boolean> {
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';
  try {
    const res = await fetch(`${linearUrl}/runs/${args.runId}/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        agent: args.agent,
        sessionKey: args.sessionKey,
        issueId: args.issueId,
        linearSessionId: args.linearSessionId,
      }),
    });
    return res.ok;
  } catch {
    console.error('[register-run] Failed to register run');
    return false;
  }
}

export async function deregisterRun(runId: string): Promise<boolean> {
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';
  try {
    const res = await fetch(`${linearUrl}/runs/${runId}`, { method: 'DELETE' });
    return res.ok;
  } catch {
    console.error('[deregister-run] Failed to deregister run');
    return false;
  }
}
