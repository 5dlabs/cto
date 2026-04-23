import { CONFIG } from "./config";

export interface GitResult {
  code: number;
  stdout: string;
  stderr: string;
}

/**
 * Run a git subprocess. We deliberately shell out rather than pulling in a
 * library — the pod already has `git` on PATH, and we want the same behavior
 * Morgan would get from a terminal (config, credentials, etc).
 */
export async function git(
  args: string[],
  opts: { cwd?: string; env?: Record<string, string>; timeoutMs?: number } = {},
): Promise<GitResult> {
  const proc = Bun.spawn({
    cmd: ["git", ...args],
    cwd: opts.cwd,
    env: { ...process.env, ...(opts.env ?? {}) },
    stdout: "pipe",
    stderr: "pipe",
  });

  // Soft timeout safety-net (default 120s).
  const timeoutMs = opts.timeoutMs ?? 120_000;
  const killer = setTimeout(() => {
    try {
      proc.kill("SIGKILL");
    } catch {
      /* ignore */
    }
  }, timeoutMs);

  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  clearTimeout(killer);
  return { code: code ?? 0, stdout, stderr };
}

export class GitError extends Error {
  constructor(
    message: string,
    public readonly result: GitResult,
  ) {
    super(message);
    this.name = "GitError";
  }
}

export async function gitOk(
  args: string[],
  opts?: Parameters<typeof git>[1],
): Promise<GitResult> {
  const r = await git(args, opts);
  if (r.code !== 0) {
    throw new GitError(
      `git ${args.join(" ")} failed (exit ${r.code}): ${r.stderr.trim() || r.stdout.trim()}`,
      r,
    );
  }
  return r;
}

/** HEAD ref name (e.g. `main`, `feature/foo`), or null if headless / unborn. */
export async function currentBranch(cwd: string): Promise<string | null> {
  const r = await git(["symbolic-ref", "--quiet", "--short", "HEAD"], { cwd });
  if (r.code !== 0) return null;
  return r.stdout.trim() || null;
}

/** First remote URL (prefers `origin`). */
export async function remoteUrl(cwd: string): Promise<string | null> {
  const r = await git(["remote", "get-url", "origin"], { cwd });
  if (r.code === 0 && r.stdout.trim()) return r.stdout.trim();
  const all = await git(["remote", "-v"], { cwd });
  if (all.code !== 0) return null;
  const line = all.stdout.split(/\r?\n/)[0];
  if (!line) return null;
  const m = line.match(/^\S+\s+(\S+)\s+\(fetch\)/);
  return m?.[1] ?? null;
}

/** Subject of HEAD, or null if unborn. */
export async function lastCommitSubject(cwd: string): Promise<string | null> {
  const r = await git(["log", "-1", "--pretty=%s"], { cwd });
  if (r.code !== 0) return null;
  return r.stdout.trim() || null;
}

/** ISO timestamp of the HEAD commit, or null when unborn. */
export async function lastCommitIso(cwd: string): Promise<string | null> {
  const r = await git(["log", "-1", "--pretty=%cI"], { cwd });
  if (r.code !== 0) return null;
  const s = r.stdout.trim();
  return s || null;
}

/** Bootstrap a brand-new repo with an empty initial commit. */
export async function initEmptyRepo(cwd: string): Promise<void> {
  await gitOk(["init", "-b", "main"], { cwd });
  await gitOk(["config", "user.name", CONFIG.commitName], { cwd });
  await gitOk(["config", "user.email", CONFIG.commitEmail], { cwd });
  // Empty initial commit so the repo has a HEAD branch; keeps downstream
  // tooling happy (tags, refs, list commits, etc).
  await gitOk(["commit", "--allow-empty", "-m", "chore: initialize project"], {
    cwd,
  });
}

/**
 * Commit any local changes as Morgan. Safe to call repeatedly — no-ops when
 * there's nothing staged.
 */
export async function commitAll(
  cwd: string,
  message: string,
): Promise<GitResult | null> {
  await gitOk(["add", "-A"], { cwd });
  const status = await gitOk(["status", "--porcelain"], { cwd });
  if (!status.stdout.trim()) return null;
  await gitOk(["config", "user.name", CONFIG.commitName], { cwd });
  await gitOk(["config", "user.email", CONFIG.commitEmail], { cwd });
  return gitOk(["commit", "-m", message], { cwd });
}
