/**
 * Runtime configuration for the project-api service.
 *
 * All knobs default to values that make sense on the Morgan OpenClaw pod
 * (shared PVC mounted at `/workspace`), and can be overridden via env for
 * local dev or alternative deployments.
 */

export interface Config {
  /** HTTP port. */
  port: number;
  /** Parent directory that contains per-project repo checkouts. */
  reposRoot: string;
  /** Where to persist service state (active project pointer, etc). */
  stateDir: string;
  /** Default GitHub org/user we clone from when a name is created. */
  githubOrg: string;
  /** Token used for repo-exists check + clone auth (optional for public). */
  githubToken: string | null;
  /** CORS: list of allowed origins, or ["*"] for permissive dev mode. */
  allowedOrigins: string[];
  /** git user.name / user.email used for automated commits (PRD writes). */
  commitName: string;
  commitEmail: string;
}

function readEnv(key: string, fallback?: string): string | undefined {
  const v = process.env[key];
  if (v != null && v !== "") return v;
  return fallback;
}

function parseOrigins(raw: string | undefined): string[] {
  if (!raw || raw.trim() === "*") return ["*"];
  return raw
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

export function loadConfig(): Config {
  return {
    port: Number(readEnv("PORT", "8091")),
    reposRoot: readEnv("REPOS_ROOT", "/workspace/repos")!,
    stateDir: readEnv("STATE_DIR", "/workspace/.openclaw")!,
    githubOrg: readEnv("GITHUB_ORG", "5dlabs")!,
    githubToken: readEnv("GITHUB_TOKEN") ?? readEnv("GH_TOKEN") ?? null,
    allowedOrigins: parseOrigins(readEnv("ALLOWED_ORIGINS", "*")),
    commitName: readEnv("GIT_COMMIT_NAME", "Morgan")!,
    commitEmail: readEnv("GIT_COMMIT_EMAIL", "morgan@5dlabs.ai")!,
  };
}

export const CONFIG: Config = loadConfig();
