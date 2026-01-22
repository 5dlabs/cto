// Cloudflare Pages Function - GET /api/ralph/state
// Returns the current Ralph loop state from KV

interface Env {
  RALPH_STATE: KVNamespace;
}

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const { env } = context;
  
  try {
    // Get state from KV
    const stateJson = await env.RALPH_STATE.get('current-state');
    
    if (!stateJson) {
      // Return default state if nothing stored
      return Response.json({
        sessionId: 'no-session',
        executor: {
          status: 'stopped',
          currentStep: 'No active session',
          stepNumber: 0,
          totalSteps: 0,
          lastUpdate: new Date().toISOString(),
          lastError: null,
        },
        watcher: {
          status: 'stopped',
          lastCheck: null,
          checkCount: 0,
        },
        stats: {
          totalRetries: 0,
          issuesDetected: 0,
          issuesFixed: 0,
          successfulSteps: 0,
          totalDuration: '0m',
        },
        hardeningActions: [],
        progressLog: ['No active Ralph loop. Start one from your terminal.'],
      });
    }
    
    return Response.json(JSON.parse(stateJson));
  } catch (error) {
    console.error('Error fetching state:', error);
    return Response.json({ error: 'Failed to fetch state' }, { status: 500 });
  }
};

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { env, request } = context;
  
  try {
    // Auth check - simple bearer token
    const authHeader = request.headers.get('Authorization');
    const expectedToken = await env.RALPH_STATE.get('auth-token');
    
    if (expectedToken && authHeader !== `Bearer ${expectedToken}`) {
      return Response.json({ error: 'Unauthorized' }, { status: 401 });
    }
    
    const state = await request.json();
    
    // Store in KV with 1 hour TTL (auto-cleanup old sessions)
    await env.RALPH_STATE.put('current-state', JSON.stringify(state), {
      expirationTtl: 3600,
    });
    
    return Response.json({ success: true });
  } catch (error) {
    console.error('Error storing state:', error);
    return Response.json({ error: 'Failed to store state' }, { status: 500 });
  }
};
