# project-api

HTTP sidecar that runs next to Morgan on the OpenClaw pod. It surfaces the
shared `/workspace/repos/` PVC to the cto-app UI and to Morgan's tool layer:

- List projects (directories under the repos root).
- Create a project: looks up `<GITHUB_ORG>/<name>` on GitHub and
  `git clone`s if it exists, otherwise `git init`s an empty repo.
- Track an "active project" pointer that the agent reads on startup to set
  its working directory.
- Write a PRD (`prd.md` at repo root) and auto-commit it.

Matches the neighbor services (`intake-agent`, `cto-tools`) in stack choice:
Bun + TypeScript, no HTTP framework dependency (uses `Bun.serve`).

## Endpoints

| Method | Path | Body | Returns |
|--------|------|------|---------|
| GET  | `/health`              | — | `{ ok, service, reposRoot, githubOrg, githubAuth }` |
| GET  | `/projects`            | — | `ProjectDescriptor[]` |
| POST | `/projects`            | `{ name }` | `{ project, mode: "cloned" \| "initialized" }` |
| GET  | `/projects/:name`      | — | `ProjectDescriptor` |
| POST | `/projects/:name/prd`  | `{ content }` | `{ project, path, bytesWritten }` |
| GET  | `/projects/active`     | — | `{ name }` |
| POST | `/projects/active`     | `{ name \| null }` | `{ name }` |

```ts
interface ProjectDescriptor {
  name: string;
  path: string;            // absolute, e.g. /workspace/repos/foo
  hasPrd: boolean;
  remoteUrl: string | null;
  updatedAt: string | null;
  branch: string | null;
  lastCommit: string | null;
}
```

## Environment

| Var | Default | Notes |
|-----|---------|-------|
| `PORT`            | `8091` | HTTP port |
| `REPOS_ROOT`      | `/workspace/repos` | Parent of per-project checkouts |
| `STATE_DIR`       | `/workspace/.openclaw` | Persists `active-project` pointer |
| `GITHUB_ORG`      | `5dlabs` | Namespace checked by create |
| `GITHUB_TOKEN`    | — | PAT or app token (optional for public repos) |
| `ALLOWED_ORIGINS` | `*` | Comma-separated for CORS; use the cto-app origin in prod |
| `GIT_COMMIT_NAME` | `Morgan` | Author for automated commits |
| `GIT_COMMIT_EMAIL`| `morgan@5dlabs.ai` | |

## Local dev

```bash
bun install
GITHUB_TOKEN=ghp_xxx REPOS_ROOT=/tmp/demo-repos STATE_DIR=/tmp/demo-state \
  bun run dev
```

Sanity check:

```bash
curl http://localhost:8091/health
curl -XPOST http://localhost:8091/projects -H 'content-type: application/json' \
  -d '{"name":"morgan-md-sandbox"}'
```

## Deploy

Runs as a sidecar on the `openclaw-morgan` pod (see
`infra/charts/openclaw-agent`). Exposed via the
`morgan-project-api.5dlabs.ai` ingress.

Container publishing is automated by
`.github/workflows/project-api-publish.yml`, which builds and pushes
`ghcr.io/5dlabs/project-api` on `main` when files under
`apps/project-api/**` change.

The UI reads the base URL from `VITE_PROJECT_API_URL`; the default baked
into the cto-app build matches the public ingress host.

## Safety

- Project names are validated against `^[a-z0-9][a-z0-9._-]{0,63}$` before
  touching disk.
- All filesystem paths are resolved and checked to live strictly under
  `REPOS_ROOT` — no `..` traversal.
- `git clone` is run as a subprocess with a 120s soft timeout.
- The service trusts its network peers (it's pod-local behind an
  authenticated ingress); no auth layer of its own. Don't expose it to the
  public internet without fronting it with something that does auth.
