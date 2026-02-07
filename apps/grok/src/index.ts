/**
 * MCP Investor Research Server - Entry Point
 * 
 * Usage:
 *   bun run src/index.ts          # Development
 *   bun run build && node dist/index.js  # Production
 *   npx @5dlab/mcp-investor-research    # Via NPM
 */

import { runServer } from './server.js';

runServer().catch((err) => {
  console.error('Failed to start server:', err);
  process.exit(1);
});
