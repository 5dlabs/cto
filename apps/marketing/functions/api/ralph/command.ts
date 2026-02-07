// Cloudflare Pages Function - POST /api/ralph/command
// Sends commands to the Ralph loop (stored in KV for polling)

interface Env {
  RALPH_STATE: KVNamespace;
}

type Command = 'pause' | 'resume' | 'stop' | 'update-prompt';

interface CommandRequest {
  command: Command;
  payload?: string; // For update-prompt
}

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { env, request } = context;
  
  try {
    const body = await request.json() as CommandRequest;
    const { command, payload } = body;
    
    // Store command in KV for the Ralph loop to poll
    const commandEntry = {
      command,
      payload,
      timestamp: new Date().toISOString(),
      id: crypto.randomUUID(),
    };
    
    // Get existing command queue
    const queueJson = await env.RALPH_STATE.get('command-queue');
    const queue = queueJson ? JSON.parse(queueJson) : [];
    
    // Add new command
    queue.push(commandEntry);
    
    // Keep only last 10 commands
    const trimmedQueue = queue.slice(-10);
    
    await env.RALPH_STATE.put('command-queue', JSON.stringify(trimmedQueue), {
      expirationTtl: 3600,
    });
    
    return Response.json({ 
      success: true, 
      commandId: commandEntry.id,
      message: `Command '${command}' queued`
    });
  } catch (error) {
    console.error('Error processing command:', error);
    return Response.json({ error: 'Failed to process command' }, { status: 500 });
  }
};

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const { env, request } = context;
  
  try {
    // Get pending commands (for Ralph loop to poll)
    const url = new URL(request.url);
    const ack = url.searchParams.get('ack'); // Command ID to acknowledge
    
    const queueJson = await env.RALPH_STATE.get('command-queue');
    let queue = queueJson ? JSON.parse(queueJson) : [];
    
    // If acknowledging a command, remove it
    if (ack) {
      queue = queue.filter((cmd: { id: string }) => cmd.id !== ack);
      await env.RALPH_STATE.put('command-queue', JSON.stringify(queue), {
        expirationTtl: 3600,
      });
    }
    
    // Return oldest pending command
    const nextCommand = queue[0] || null;
    
    return Response.json({ 
      command: nextCommand,
      queueLength: queue.length 
    });
  } catch (error) {
    console.error('Error fetching commands:', error);
    return Response.json({ error: 'Failed to fetch commands' }, { status: 500 });
  }
};
