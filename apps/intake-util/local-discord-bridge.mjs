#!/usr/bin/env node
/**
 * local-discord-bridge — Translates bridge-notify POSTs into
 * `openclaw message send` calls for local development.
 *
 * Listens on LOCAL_BRIDGE_PORT (default 3200) and accepts
 * POST /notify with the standard bridge payload.
 *
 * Speaker identity from `payload.from` is formatted as a bold
 * header in the Discord message.
 */

import { createServer } from 'node:http';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const exec = promisify(execFile);

const PORT = parseInt(process.env.LOCAL_BRIDGE_PORT ?? '3200', 10);
const CHANNEL_ID = process.env.DELIBERATION_CHANNEL_ID ?? '1471014430065164461';

const PERSONA_LABELS = {
  optimist: '🟢 Optimist',
  pessimist: '🔴 Pessimist',
  intake: '⚙️ Intake',
  'voter-1': '🏛️ Voter: Architect',
  'voter-2': '🔧 Voter: Pragmatist',
  'voter-3': '✂️ Voter: Minimalist',
  'voter-4': '📡 Voter: Operator',
  'voter-5': '🗺️ Voter: Strategist',
};

function formatMessage(payload) {
  const speaker = PERSONA_LABELS[payload.from] ?? payload.from;
  const meta = payload.metadata ?? {};
  const turn = meta.turn ? ` — Turn ${meta.turn}` : '';
  const step = meta.step ? ` [${meta.step}]` : '';

  // Truncate message to Discord's 2000 char limit (minus header)
  const header = `**[${speaker}]**${turn}${step}`;
  const maxBody = 1950 - header.length;
  const body = payload.message?.length > maxBody
    ? payload.message.slice(0, maxBody) + '…'
    : payload.message ?? '';

  return `${header}\n${body}`;
}

const server = createServer(async (req, res) => {
  if (req.method === 'POST' && req.url === '/notify') {
    let body = '';
    for await (const chunk of req) body += chunk;

    try {
      const payload = JSON.parse(body);
      const message = formatMessage(payload);

      await exec('openclaw', [
        'message', 'send',
        '--channel', 'discord',
        '--target', CHANNEL_ID,
        '-m', message,
      ], { timeout: 10000 });

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: true }));
      console.log(`✓ ${payload.from} → #intake`);
    } catch (err) {
      console.error(`✗ bridge error: ${err.message}`);
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: false, error: err.message }));
    }
  } else if (req.method === 'GET' && req.url === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, status: 'live' }));
  } else {
    res.writeHead(404);
    res.end('Not found');
  }
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`🌉 Local Discord bridge listening on http://127.0.0.1:${PORT}`);
  console.log(`   Target channel: ${CHANNEL_ID}`);
  console.log(`   POST /notify to send deliberation messages`);
});
