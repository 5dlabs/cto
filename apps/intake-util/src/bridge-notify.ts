/**
 * bridge-notify — POST agent notification to both bridges via HTTP.
 * Replaces nats-notify. Degrades gracefully if bridges unavailable.
 */

interface BridgeNotifyArgs {
  from: string;
  to: string;
  message: string;
  metadata?: Record<string, string>;
  priority?: 'normal' | 'urgent';
}

async function postToBridge(url: string, payload: unknown): Promise<boolean> {
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    return res.ok;
  } catch {
    return false;
  }
}

export async function bridgeNotify(args: BridgeNotifyArgs): Promise<{ discord: boolean; linear: boolean }> {
  const discordUrl = process.env['DISCORD_BRIDGE_URL'] ?? 'http://discord-bridge.bots.svc:3200';
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';

  const payload = {
    from: args.from,
    to: args.to,
    subject: `agent.${args.to}.inbox`,
    message: args.message,
    priority: args.priority ?? 'normal',
    timestamp: new Date().toISOString(),
    type: 'message' as const,
    metadata: args.metadata,
  };

  const [discord, linear] = await Promise.all([
    postToBridge(`${discordUrl}/notify`, payload),
    postToBridge(`${linearUrl}/notify`, payload),
  ]);

  return { discord, linear };
}
