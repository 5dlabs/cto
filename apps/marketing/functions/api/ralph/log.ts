// Cloudflare Pages Function - POST /api/ralph/log
// Append to the progress log

interface Env {
  RALPH_STATE: KVNamespace;
}

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { env, request } = context;
  
  try {
    const body = await request.json() as { message: string; level?: string };
    const { message, level = 'info' } = body;
    
    // Get current state
    const stateJson = await env.RALPH_STATE.get('current-state');
    if (!stateJson) {
      return Response.json({ error: 'No active session' }, { status: 404 });
    }
    
    const state = JSON.parse(stateJson);
    
    // Format log entry
    const timestamp = new Date().toISOString().replace('T', ' ').slice(0, 19);
    const prefix = level === 'error' ? '❌' : level === 'success' ? '✅' : '→';
    const logEntry = `[${timestamp}] ${prefix} ${message}`;
    
    // Append to log (keep last 100 entries)
    state.progressLog = [...(state.progressLog || []), logEntry].slice(-100);
    
    // Update state
    await env.RALPH_STATE.put('current-state', JSON.stringify(state), {
      expirationTtl: 3600,
    });
    
    return Response.json({ success: true });
  } catch (error) {
    console.error('Error appending log:', error);
    return Response.json({ error: 'Failed to append log' }, { status: 500 });
  }
};
