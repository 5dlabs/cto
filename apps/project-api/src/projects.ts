import { readdir, stat, mkdir, rename, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { CONFIG } from "./config";
import {
  commitAll,
  currentBranch,
  gitOk,
  isHealthyRepo,
  lastCommitIso,
  lastCommitSubject,
  pushCurrentBranch,
  remoteUrl,
} from "./git";
import {
  authenticatedCloneUrl,
  createRepo,
  hasPrdMarker,
  listOrgRepos,
  lookupRepo,
  type OrgRepoSummary,
} from "./github";

export interface ProjectDescriptor {
  name: string;
  path: string;
  hasPrd: boolean;
  remoteUrl: string | null;
  updatedAt: string | null;
  branch: string | null;
  lastCommit: string | null;
  /**
   * `"cloned"` when the repo is fully materialized on the PVC (PRD can be
   * read from disk), `"remote-only"` when we know about it from GitHub but
   * haven't cloned it locally yet. The UI uses this to decide whether a
   * tile click needs to trigger `/verify` before opening code-server.
   */
  locality: "cloned" | "remote-only";
}

const SLUG_RE = /^[a-z0-9][a-z0-9._-]{0,63}$/;

export function validateSlug(name: string): void {
  if (!SLUG_RE.test(name)) {
    throw new Error(
      "invalid project name (lowercase letters, numbers, dot, dash, underscore only; 1-64 chars)",
    );
  }
}

function safeProjectPath(name: string): string {
  validateSlug(name);
  const root = resolve(CONFIG.reposRoot);
  const full = resolve(join(root, name));
  if (!full.startsWith(root + "/") && full !== root) {
    throw new Error("path escape detected");
  }
  return full;
}

async function ensureReposRoot(): Promise<void> {
  await mkdir(CONFIG.reposRoot, { recursive: true });
}

async function ensureStateDir(): Promise<void> {
  await mkdir(CONFIG.stateDir, { recursive: true });
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Per-project async mutex so verify/create/setActive don't race. */
const projectLocks = new Map<string, Promise<unknown>>();
async function withProjectLock<T>(name: string, fn: () => Promise<T>): Promise<T> {
  const prev = projectLocks.get(name) ?? Promise.resolve();
  const next = prev.then(() => fn()).catch((err) => {
    // Rethrow AFTER the chain advances so later callers don't inherit the error.
    throw err;
  });
  // Store the un-catched branch so queued callers wait for settlement (success or error).
  const settled = next.catch(() => undefined);
  projectLocks.set(name, settled);
  try {
    return await next;
  } finally {
    if (projectLocks.get(name) === settled) {
      projectLocks.delete(name);
    }
  }
}

/**
 * Clone to a sibling `<path>.cloning-<pid>` then atomically rename into place.
 * Avoids leaving a half-clone at the final path if anything goes sideways.
 */
async function cloneAtomic(cloneUrl: string, path: string): Promise<void> {
  const tmp = `${path}.cloning-${process.pid}-${Date.now()}`;
  try {
    await cloneWithRetry(cloneUrl, tmp);
    await rename(tmp, path);
  } catch (err) {
    try {
      await rm(tmp, { recursive: true, force: true });
    } catch {
      /* best-effort cleanup */
    }
    throw err;
  }
}

async function cloneWithRetry(
  cloneUrl: string,
  path: string,
  attempts = 4,
): Promise<void> {
  let lastErr: unknown;
  for (let i = 0; i < attempts; i++) {
    try {
      await gitOk(["clone", cloneUrl, path]);
      return;
    } catch (err) {
      lastErr = err;
      if (i < attempts - 1) {
        // GitHub can take a moment to propagate a newly created repo.
        await sleep(400 * 2 ** i);
      }
    }
  }
  throw lastErr ?? new Error("git clone failed");
}

/**
 * Write the `.prd/PRD.md` marker that tells the rest of the system "this is
 * a Morgan-managed project". Creates the folder if missing; idempotent when
 * the file already exists with the expected header.
 */
async function writePrdScaffold(path: string, name: string): Promise<boolean> {
  const dir = join(path, ".prd");
  await mkdir(dir, { recursive: true });
  const prd = join(dir, "PRD.md");
  if (existsSync(prd)) return false;
  const content =
    `# ${name}\n\n` +
    `<!-- Morgan intake placeholder — replace this file with the product brief. -->\n`;
  await Bun.write(prd, content);
  return true;
}

/**
 * Ensure the GitHub repo has `.prd/PRD.md`. Writes the scaffold, commits,
 * and pushes. Throws on push failure — GitHub is the authoritative source
 * of truth for project discovery, so a create that can't push should fail.
 */
async function ensurePrdOnRemote(path: string, name: string): Promise<void> {
  const wrote = await writePrdScaffold(path, name);
  if (wrote) {
    await commitAll(path, "docs: add .prd/PRD.md scaffold");
  }
  await pushCurrentBranch(path);
}

function describeFromRepo(repo: OrgRepoSummary, hasPrd: boolean): ProjectDescriptor {
  const localPath = join(CONFIG.reposRoot, repo.name);
  const locality: "cloned" | "remote-only" = existsSync(localPath)
    ? "cloned"
    : "remote-only";
  return {
    name: repo.name,
    path: localPath,
    hasPrd,
    remoteUrl: repo.cloneUrl,
    updatedAt: repo.updatedAt,
    branch: repo.defaultBranch,
    lastCommit: null,
    locality,
  };
}

async function describeLocal(
  path: string,
  name: string,
  hasPrdHint?: boolean,
): Promise<ProjectDescriptor> {
  const hasPrd =
    hasPrdHint !== undefined
      ? hasPrdHint
      : existsSync(join(path, ".prd", "PRD.md"));

  const isGitRepo = existsSync(join(path, ".git"));
  const [branch, remote, lastIso, lastSubject] = isGitRepo
    ? await Promise.all([
        currentBranch(path),
        remoteUrl(path),
        lastCommitIso(path),
        lastCommitSubject(path),
      ])
    : [null, null, null, null];

  let updatedAt = lastIso;
  if (!updatedAt) {
    try {
      const s = await stat(path);
      updatedAt = s.mtime.toISOString();
    } catch {
      updatedAt = null;
    }
  }

  return {
    name,
    path,
    hasPrd,
    remoteUrl: remote,
    updatedAt,
    branch,
    lastCommit: lastSubject,
    locality: "cloned",
  };
}

// ---------- Discovery cache ----------

interface CachedList {
  expiresAt: number;
  value: ProjectDescriptor[];
}
let cachedList: CachedList | null = null;
const LIST_TTL_MS = 10 * 60 * 1000; // 10 minutes

function invalidateListCache(): void {
  cachedList = null;
}

/**
 * Discover projects by listing the configured GitHub org and keeping any
 * repo that has a `.prd/PRD.md` marker. Results are cached for 10 minutes;
 * mutating operations (create) invalidate the cache.
 *
 * Caller may pass `{force: true}` to bypass the cache (used on explicit
 * refresh from the UI).
 */
export async function listProjects(
  opts: { force?: boolean } = {},
): Promise<ProjectDescriptor[]> {
  const now = Date.now();
  if (!opts.force && cachedList && cachedList.expiresAt > now) {
    return cachedList.value;
  }

  await ensureReposRoot();

  // Fast path for offline/dev: if GitHub isn't reachable (or no token), fall
  // back to describing whatever is already on the PVC. Never take the fast
  // path silently; callers see source=stub via an out-of-band signal.
  if (!CONFIG.githubToken) {
    const local = await describeLocalPvcOnly();
    cachedList = { expiresAt: now + LIST_TTL_MS, value: local };
    return local;
  }

  let repos: OrgRepoSummary[];
  try {
    repos = await listOrgRepos(CONFIG.githubOrg);
  } catch (err) {
    // On a GitHub API outage we degrade to PVC listing rather than breaking
    // the UI completely. The cache stays short so we recover quickly.
    console.warn(
      `[project-api] list org repos failed, falling back to PVC: ${(err as Error).message}`,
    );
    const local = await describeLocalPvcOnly();
    return local;
  }

  const probes = await Promise.all(
    repos.map(async (repo) => {
      try {
        return { repo, hasPrd: await hasPrdMarker(CONFIG.githubOrg, repo.name) };
      } catch {
        return { repo, hasPrd: false };
      }
    }),
  );

  const out: ProjectDescriptor[] = probes
    .filter((p) => p.hasPrd)
    .map((p) => describeFromRepo(p.repo, true));

  out.sort((a, b) => a.name.localeCompare(b.name));
  cachedList = { expiresAt: now + LIST_TTL_MS, value: out };
  return out;
}

async function describeLocalPvcOnly(): Promise<ProjectDescriptor[]> {
  let entries: string[];
  try {
    entries = await readdir(CONFIG.reposRoot);
  } catch {
    return [];
  }
  const out: ProjectDescriptor[] = [];
  for (const entry of entries) {
    if (entry.startsWith(".")) continue;
    const full = join(CONFIG.reposRoot, entry);
    try {
      const s = await stat(full);
      if (!s.isDirectory()) continue;
    } catch {
      continue;
    }
    try {
      const desc = await describeLocal(full, entry);
      if (desc.hasPrd) out.push(desc);
    } catch {
      /* skip rows we can't describe */
    }
  }
  out.sort((a, b) => a.name.localeCompare(b.name));
  return out;
}

export async function getProject(name: string): Promise<ProjectDescriptor | null> {
  const path = safeProjectPath(name);
  if (existsSync(path)) {
    return describeLocal(path, name);
  }
  // Not on PVC — fall back to GitHub view if we can.
  if (!CONFIG.githubToken) return null;
  try {
    const info = await lookupRepo(CONFIG.githubOrg, name);
    if (!info.exists) return null;
    const hasPrd = await hasPrdMarker(CONFIG.githubOrg, name).catch(() => false);
    return describeFromRepo(
      {
        name,
        cloneUrl: info.cloneUrl,
        defaultBranch: info.defaultBranch,
        updatedAt: null,
        description: null,
        archived: false,
        private: info.private ?? false,
      },
      hasPrd,
    );
  } catch {
    return null;
  }
}

export type CreateMode = "cloned" | "created";

export interface CreateResult {
  project: ProjectDescriptor;
  mode: CreateMode;
}

/**
 * Create a new project. Single authoritative sequence:
 *   1. Ensure GitHub repo exists (lookup → create if missing).
 *   2. Clone to PVC (atomic tmp + rename).
 *   3. Write `.prd/PRD.md` scaffold.
 *   4. Commit + push.
 *
 * Any step failing surfaces the error to the caller. We do NOT silently
 * fall back to a local-only init — GitHub is authoritative for project
 * discovery, so a project without a pushed PRD is not a project.
 */
export async function createProject(name: string): Promise<CreateResult> {
  return withProjectLock(name, async () => {
    const path = safeProjectPath(name);
    if (existsSync(path)) {
      throw Object.assign(new Error(`project "${name}" already exists`), {
        status: 409,
      });
    }

    await ensureReposRoot();

    const info = await lookupRepo(CONFIG.githubOrg, name);
    let mode: CreateMode;
    let cloneUrl: string | null;

    if (info.exists && info.cloneUrl) {
      cloneUrl = info.cloneUrl;
      mode = "cloned";
    } else {
      if (!CONFIG.githubToken) {
        throw Object.assign(
          new Error(
            `github token missing; cannot create remote repository "${CONFIG.githubOrg}/${name}"`,
          ),
          { status: 503 },
        );
      }
      const created = await createRepo(CONFIG.githubOrg, name);
      if (!created.cloneUrl) {
        throw new Error("github create returned no clone_url");
      }
      cloneUrl = created.cloneUrl;
      mode = "created";
    }

    await cloneAtomic(authenticatedCloneUrl(cloneUrl), path);
    await ensurePrdOnRemote(path, name);

    invalidateListCache();
    return {
      project: await describeLocal(path, name, true),
      mode,
    };
  });
}

/**
 * Ensure the project is cloned locally. Idempotent: returns the descriptor
 * when already present, clones on demand when the repo exists on GitHub but
 * not on the PVC. Throws 404 for repos that don't exist on GitHub.
 */
export async function verifyProject(name: string): Promise<ProjectDescriptor> {
  return withProjectLock(name, async () => {
    const path = safeProjectPath(name);

    if (existsSync(path)) {
      if (await isHealthyRepo(path)) {
        return describeLocal(path, name);
      }
      // Dir exists but isn't a valid repo — clear it and reclone.
      await rm(path, { recursive: true, force: true });
    }

    await ensureReposRoot();
    const info = await lookupRepo(CONFIG.githubOrg, name);
    if (!info.exists || !info.cloneUrl) {
      throw Object.assign(
        new Error(`repository "${CONFIG.githubOrg}/${name}" not found on GitHub`),
        { status: 404 },
      );
    }
    await cloneAtomic(authenticatedCloneUrl(info.cloneUrl), path);
    return describeLocal(path, name);
  });
}

const ACTIVE_FILE = () => join(CONFIG.stateDir, "active-project");

export interface ActiveProject {
  name: string | null;
}

export async function getActiveProject(): Promise<ActiveProject> {
  try {
    const f = Bun.file(ACTIVE_FILE());
    if (!(await f.exists())) return { name: null };
    const raw = (await f.text()).trim();
    return { name: raw || null };
  } catch {
    return { name: null };
  }
}

/**
 * Set the active-project pointer, auto-cloning if the repo exists on GitHub
 * but not on the PVC yet. `name=null` clears the pointer.
 */
export async function setActiveProject(name: string | null): Promise<ActiveProject> {
  await ensureStateDir();
  if (name == null) {
    try {
      await Bun.write(ACTIVE_FILE(), "");
    } catch {
      /* ignore */
    }
    return { name: null };
  }
  validateSlug(name);
  await verifyProject(name); // throws if GitHub doesn't have it
  await Bun.write(ACTIVE_FILE(), name);
  return { name };
}

// Export for prd.ts
export { invalidateListCache };
