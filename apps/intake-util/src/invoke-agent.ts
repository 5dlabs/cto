/**
 * invoke-agent — Unified agent invocation supporting subagent and A2A modes.
 */

import * as fs from 'fs';

type InvokeMode = 'subagent' | 'a2a' | 'acp';

interface InvokeAgentArgs {
  mode: InvokeMode;
  agent: string;
  sessionKey?: string;
  taskContext?: string;
  promptFile?: string;
  prompt?: string;
}

function normalizeMode(mode: InvokeMode): 'subagent' | 'a2a' {
  if (mode === 'acp') {
    console.warn('invoke-agent: --mode acp is deprecated; use --mode a2a');
    return 'a2a';
  }

  return mode;
}

export async function invokeAgent(args: InvokeAgentArgs): Promise<{ success: boolean; response?: string; error?: string }> {
  const agentHost = `http://${args.agent}.agents.svc:18789`;
  const promptContent = args.promptFile ? fs.readFileSync(args.promptFile, 'utf-8') : (args.prompt ?? '');
  const mode = normalizeMode(args.mode);

  if (mode === 'subagent') {
    // Native OpenClaw /hooks/agent
    const hooksToken = process.env['HOOKS_TOKEN'] ?? '';
    try {
      const res = await fetch(`${agentHost}/hooks/agent`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(hooksToken ? { Authorization: `Bearer ${hooksToken}` } : {}),
        },
        body: JSON.stringify({
          message: promptContent,
          sessionKey: args.sessionKey ?? args.taskContext ?? '',
        }),
      });
      if (!res.ok) return { success: false, error: `Agent returned ${res.status}` };
      const body = await res.text();
      return { success: true, response: body };
    } catch (err) {
      return { success: false, error: String(err) };
    }
  }

  if (mode === 'a2a') {
    // A2A JSON-RPC
    try {
      const res = await fetch(`${agentHost}/a2a`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: `req-${Date.now()}`,
          method: 'a2a.SendMessage',
          params: {
            message: {
              role: 'user',
              parts: [{ text: promptContent }],
            },
            ...(args.taskContext ? { taskId: args.taskContext } : {}),
          },
        }),
      });
      if (!res.ok) return { success: false, error: `A2A returned ${res.status}` };
      const body = await res.json() as { result?: unknown; error?: { message: string } };
      if (body.error) return { success: false, error: body.error.message };
      return { success: true, response: JSON.stringify(body.result) };
    } catch (err) {
      return { success: false, error: String(err) };
    }
  }

  return { success: false, error: `Unknown mode: ${args.mode}` };
}
