/**
 * Utility to find the Claude Code CLI executable.
 * 
 * The @anthropic-ai/claude-code SDK requires the Claude Code CLI to be installed.
 * This module helps locate the CLI in various installation locations.
 */

import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { homedir } from 'os';

/**
 * Common locations where the Claude CLI might be installed.
 */
const COMMON_LOCATIONS = [
  // Bun cache (latest version)
  join(homedir(), '.bun/install/cache/@anthropic-ai/claude-code@2.1.22@@@1/cli.js'),
  // NPM global
  '/usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js',
  // Homebrew
  '/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/cli.js',
  // Local node_modules (relative to this project)
  join(dirname(import.meta.dir), 'node_modules/@anthropic-ai/claude-code/cli.js'),
];

/**
 * Find available Bun cache versions of claude-code.
 */
function findBunCacheVersions(): string[] {
  const bunCacheDir = join(homedir(), '.bun/install/cache');
  const paths: string[] = [];
  
  try {
    // Try to find claude-code directories in bun cache
    const pattern = '@anthropic-ai/claude-code@';
    const cacheContent = Bun.spawnSync({
      cmd: ['ls', bunCacheDir],
      stdout: 'pipe',
      stderr: 'pipe',
    });
    
    if (cacheContent.exitCode === 0) {
      const dirs = cacheContent.stdout.toString().split('\n');
      for (const dir of dirs) {
        if (dir.startsWith(pattern)) {
          const cliPath = join(bunCacheDir, dir, 'cli.js');
          if (existsSync(cliPath)) {
            paths.push(cliPath);
          }
        }
      }
    }
  } catch {
    // Ignore errors, fall back to common locations
  }
  
  return paths;
}

/**
 * Find the Claude Code CLI executable.
 * 
 * Search order:
 * 1. CLAUDE_CODE_PATH environment variable
 * 2. Bun cache (dynamically discovered)
 * 3. Common installation locations
 * 
 * @returns Path to the CLI executable, or null if not found
 */
export function findClaudeCli(): string | null {
  // 1. Check environment variable
  const envPath = process.env['CLAUDE_CODE_PATH'];
  if (envPath && existsSync(envPath)) {
    return envPath;
  }
  
  // 2. Check Bun cache (dynamic discovery)
  const bunPaths = findBunCacheVersions();
  // Sort to get latest version (higher version numbers first)
  bunPaths.sort().reverse();
  for (const path of bunPaths) {
    if (existsSync(path)) {
      return path;
    }
  }
  
  // 3. Check common locations
  for (const location of COMMON_LOCATIONS) {
    if (existsSync(location)) {
      return location;
    }
  }
  
  return null;
}

/**
 * Get the Claude CLI path or throw an error.
 */
export function getClaudeCliOrThrow(): string {
  const cliPath = findClaudeCli();
  if (!cliPath) {
    throw new Error(
      'Claude Code executable not found. Install it with: bun add -g @anthropic-ai/claude-code ' +
      'or set CLAUDE_CODE_PATH environment variable.'
    );
  }
  return cliPath;
}

/**
 * Check if Claude CLI is available.
 */
export function isClaudeCliAvailable(): boolean {
  return findClaudeCli() !== null;
}
