import { readdir, stat, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { CONFIG } from "./config";
import {
  commitAll,
  currentBranch,
  gitOk,
  initEmptyRepo,
  lastCommitIso,
  lastCommitSubject,
  remoteUrl,
} from "./git";
import { authenticatedCloneUrl, createRepo, lookupRepo } from "./github";

export interface ProjectDescriptor {
  name: string;
  path: string;
  hasPrd: boolean;
  remoteUrl: string | null;
  updatedAt: string | null;
  branch: string | null;
  lastCommit: string | null;
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

async function ensureScaffold(path: string, name: string): Promise<void> {
  const readmePath = join(path, "README.md");
  if (!existsSync(readmePath)) {
    await Bun.write(
      readmePath,
      `# ${name}\n\nProject scaffold created by Morgan project-api.\n`,
    );
  }
  try {
    await commitAll(path, "docs: add project scaffold");
  } catch {
    // Non-fatal; scaffolding file still exists for immediate workspace visibility.
  }
}

async function initWithRemote(
  path: string,
  remote: string,
  name: string,
): Promise<void> {
  await mkdir(path, { recursive: true });
  await initEmptyRepo(path);
  await ensureScaffold(path, name);
  try {
    await gitOk(["remote", "add", "origin", remote], { cwd: path });
  } catch {
    // Non-fatal; repo is still initialized locally.
  }
}

async function describe(path: string, name: string): Promise<ProjectDescriptor> {
  const prdPath = join(path, "prd.md");
  const hasPrd = existsSync(prdPath);

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
  };
}

export async function listProjects(): Promise<ProjectDescriptor[]> {
  await ensureReposRoot();
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
      out.push(await describe(full, entry));
    } catch {
      /* skip rows we can't describe — probably being written concurrently */
    }
  }
  out.sort((a, b) => a.name.localeCompare(b.name));
  return out;
}

export async function getProject(name: string): Promise<ProjectDescriptor | null> {
  const path = safeProjectPath(name);
  if (!existsSync(path)) return null;
  return describe(path, name);
}

export type CreateMode = "cloned" | "created" | "initialized";

export interface CreateResult {
  project: ProjectDescriptor;
  mode: CreateMode;
}

/**
 * Create a new project directory under `reposRoot`. If the GitHub repo exists
 * at `<githubOrg>/<name>`, clone it; otherwise initialize a fresh empty repo
 * so Morgan can start writing immediately (the prd.md write happens later).
 */
export async function createProject(name: string): Promise<CreateResult> {
  const path = safeProjectPath(name);
  if (existsSync(path)) {
    throw Object.assign(new Error(`project "${name}" already exists`), {
      status: 409,
    });
  }

  await ensureReposRoot();

  const info = await lookupRepo(CONFIG.githubOrg, name);

  if (info.exists && info.cloneUrl) {
    const cloneUrl = authenticatedCloneUrl(info.cloneUrl);
    await cloneWithRetry(cloneUrl, path);
    return {
      project: await describe(path, name),
      mode: "cloned",
    };
  }

  if (CONFIG.githubToken) {
    const created = await createRepo(CONFIG.githubOrg, name);
    if (created.cloneUrl) {
      const cloneUrl = authenticatedCloneUrl(created.cloneUrl);
      try {
        await cloneWithRetry(cloneUrl, path);
      } catch {
        // Last-resort fallback: keep local flow unblocked even if GitHub's
        // new repo isn't clonable yet in this instant.
        await initWithRemote(path, created.cloneUrl, name);
      }
      return {
        project: await describe(path, name),
        mode: "created",
      };
    }
  }

  await initWithRemote(
    path,
    `https://github.com/${CONFIG.githubOrg}/${name}.git`,
    name,
  );
  return {
    project: await describe(path, name),
    mode: "initialized",
  };
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
  if (!existsSync(safeProjectPath(name))) {
    throw Object.assign(new Error(`project "${name}" not found`), {
      status: 404,
    });
  }
  await Bun.write(ACTIVE_FILE(), name);
  return { name };
}
