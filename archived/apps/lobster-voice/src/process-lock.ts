import { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync, readdirSync } from "fs";
import { join } from "path";

function lockDir(): string {
  const ws = process.env.WORKSPACE ?? process.cwd();
  return join(ws, ".intake", ".voice-locks");
}

function lockPath(speaker: string): string {
  return join(lockDir(), `${speaker}.pid`);
}

function pidIsAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

export interface RogueInfo {
  speaker: string;
  existingPid: number;
}

/**
 * Acquire a process lock for a speaker identity.
 * Returns rogue info if a live duplicate is detected, null otherwise.
 */
export function acquireLock(speaker: string): RogueInfo | null {
  const dir = lockDir();
  mkdirSync(dir, { recursive: true });

  const path = lockPath(speaker);
  let rogue: RogueInfo | null = null;

  if (existsSync(path)) {
    try {
      const raw = readFileSync(path, "utf-8").trim();
      const existingPid = parseInt(raw, 10);
      if (!isNaN(existingPid) && existingPid !== process.pid && pidIsAlive(existingPid)) {
        rogue = { speaker, existingPid };
      }
    } catch {
      // stale or unreadable — overwrite
    }
  }

  writeFileSync(path, String(process.pid), "utf-8");

  process.on("exit", () => {
    try {
      const current = readFileSync(path, "utf-8").trim();
      if (current === String(process.pid)) {
        unlinkSync(path);
      }
    } catch {
      // already cleaned
    }
  });

  return rogue;
}

export function releaseLock(speaker: string): void {
  const path = lockPath(speaker);
  try {
    if (existsSync(path)) {
      const current = readFileSync(path, "utf-8").trim();
      if (current === String(process.pid)) {
        unlinkSync(path);
      }
    }
  } catch {
    // best-effort
  }
}

export interface ActiveLock {
  speaker: string;
  pid: number;
  alive: boolean;
}

export function listLocks(): ActiveLock[] {
  const dir = lockDir();
  if (!existsSync(dir)) return [];

  const results: ActiveLock[] = [];
  for (const file of readdirSync(dir)) {
    if (!file.endsWith(".pid")) continue;
    const speaker = file.replace(/\.pid$/, "");
    try {
      const raw = readFileSync(join(dir, file), "utf-8").trim();
      const pid = parseInt(raw, 10);
      if (!isNaN(pid)) {
        results.push({ speaker, pid, alive: pidIsAlive(pid) });
      }
    } catch {
      // skip unreadable
    }
  }
  return results;
}
