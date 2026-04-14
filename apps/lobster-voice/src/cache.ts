import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync, statSync, unlinkSync } from "fs";
import { join } from "path";
import { createHash } from "crypto";

const TTL_MS = 7 * 24 * 60 * 60 * 1000; // 7 days

function cacheDir(): string {
  const workspace = process.env.WORKSPACE ?? process.cwd();
  const dir = join(workspace, ".intake", ".voice-cache");
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  return dir;
}

function cacheKey(voiceId: string, text: string): string {
  return createHash("sha256").update(`${voiceId}:${text}`).digest("hex");
}

export function getCached(voiceId: string, text: string): Buffer | null {
  const dir = cacheDir();
  const key = cacheKey(voiceId, text);
  const path = join(dir, `${key}.mp3`);

  if (!existsSync(path)) return null;

  try {
    const stat = statSync(path);
    if (Date.now() - stat.mtimeMs > TTL_MS) {
      unlinkSync(path);
      return null;
    }
    return readFileSync(path);
  } catch {
    return null;
  }
}

export function getCachedPath(voiceId: string, text: string): string | null {
  const dir = cacheDir();
  const key = cacheKey(voiceId, text);
  const path = join(dir, `${key}.mp3`);

  if (!existsSync(path)) return null;

  try {
    const stat = statSync(path);
    if (Date.now() - stat.mtimeMs > TTL_MS) {
      unlinkSync(path);
      return null;
    }
    return path;
  } catch {
    return null;
  }
}

export function putCached(voiceId: string, text: string, audio: Buffer): string {
  const dir = cacheDir();
  const key = cacheKey(voiceId, text);
  const path = join(dir, `${key}.mp3`);
  writeFileSync(path, audio);
  return path;
}

export function pruneExpired(): number {
  const dir = cacheDir();
  let pruned = 0;

  try {
    for (const file of readdirSync(dir)) {
      if (!file.endsWith(".mp3")) continue;
      const path = join(dir, file);
      try {
        const stat = statSync(path);
        if (Date.now() - stat.mtimeMs > TTL_MS) {
          unlinkSync(path);
          pruned++;
        }
      } catch {
        continue;
      }
    }
  } catch {
    // cache dir doesn't exist yet
  }

  return pruned;
}
