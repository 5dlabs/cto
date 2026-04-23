#!/usr/bin/env bun
/**
 * project-api — HTTP sidecar for the Morgan OpenClaw pod.
 *
 * Surfaces the shared `/workspace/repos/` PVC to the cto-app UI and to
 * Morgan's tool layer. Single-tenant: runs next to the agent, trusts its
 * local filesystem, talks to GitHub for repo existence/clone, and persists
 * the "active project" pointer the agent reads to set its working dir.
 *
 * Endpoints (all JSON):
 *   GET  /health
 *   GET  /projects                 → ProjectDescriptor[]
 *   POST /projects                 → { project, mode }
 *   GET  /projects/:name           → ProjectDescriptor
 *   POST /projects/:name/prd       → { project, path, bytesWritten }
 *   GET  /projects/active          → { name }
 *   POST /projects/active          → { name }
 */

import { CONFIG } from "./config";
import { empty, json, problem } from "./http";
import {
  createProject,
  getActiveProject,
  getProject,
  listProjects,
  setActiveProject,
  validateSlug,
} from "./projects";
import { writePrd } from "./prd";

interface RouteCtx {
  req: Request;
  url: URL;
  origin: string | null;
}

async function handle(ctx: RouteCtx): Promise<Response> {
  const { req, url, origin } = ctx;
  const { pathname } = url;

  if (req.method === "OPTIONS") return empty(204, origin);

  if (pathname === "/health" && req.method === "GET") {
    return json(
      {
        ok: true,
        service: "project-api",
        reposRoot: CONFIG.reposRoot,
        githubOrg: CONFIG.githubOrg,
        githubAuth: Boolean(CONFIG.githubToken),
      },
      { origin },
    );
  }

  if (pathname === "/projects" && req.method === "GET") {
    const projects = await listProjects();
    return json(projects, { origin });
  }

  if (pathname === "/projects" && req.method === "POST") {
    const body = (await readJson(req)) as { name?: string };
    if (!body?.name || typeof body.name !== "string") {
      return problem(400, "body must include { name: string }", origin);
    }
    try {
      const res = await createProject(body.name);
      return json(res, { status: 201, origin });
    } catch (err) {
      return mapError(err, origin);
    }
  }

  if (pathname === "/projects/active" && req.method === "GET") {
    return json(await getActiveProject(), { origin });
  }

  if (pathname === "/projects/active" && req.method === "POST") {
    const body = (await readJson(req)) as { name?: string | null };
    try {
      const res = await setActiveProject(body?.name ?? null);
      return json(res, { origin });
    } catch (err) {
      return mapError(err, origin);
    }
  }

  // /projects/:name[/prd]
  const projectRoute = pathname.match(/^\/projects\/([^/]+)(?:\/(prd))?$/);
  if (projectRoute) {
    const rawName = decodeURIComponent(projectRoute[1] ?? "");
    try {
      validateSlug(rawName);
    } catch (err) {
      return mapError(err, origin);
    }
    const sub = projectRoute[2];

    if (!sub && req.method === "GET") {
      const p = await getProject(rawName);
      if (!p) return problem(404, `project "${rawName}" not found`, origin);
      return json(p, { origin });
    }

    if (sub === "prd" && req.method === "POST") {
      const body = (await readJson(req)) as { content?: string };
      if (typeof body?.content !== "string") {
        return problem(400, "body must include { content: string }", origin);
      }
      try {
        const res = await writePrd(rawName, body.content);
        return json(res, { origin });
      } catch (err) {
        return mapError(err, origin);
      }
    }

    return problem(405, "method not allowed", origin);
  }

  return problem(404, "not found", origin);
}

async function readJson(req: Request): Promise<unknown> {
  try {
    if (req.headers.get("content-length") === "0") return {};
    const text = await req.text();
    if (!text) return {};
    return JSON.parse(text);
  } catch {
    return {};
  }
}

function mapError(err: unknown, origin: string | null): Response {
  if (err && typeof err === "object" && "status" in err) {
    const status = Number((err as { status: unknown }).status);
    if (Number.isFinite(status)) {
      const msg =
        err instanceof Error
          ? err.message
          : String((err as { message?: unknown }).message ?? "error");
      return problem(status, msg, origin);
    }
  }
  if (err instanceof Error) return problem(500, err.message, origin);
  return problem(500, String(err), origin);
}

const server = Bun.serve({
  port: CONFIG.port,
  hostname: "0.0.0.0",
  fetch(req) {
    const url = new URL(req.url);
    const origin = req.headers.get("origin");
    return handle({ req, url, origin }).catch((err) => mapError(err, origin));
  },
});

// eslint-disable-next-line no-console
console.log(
  `[project-api] listening on :${server.port} — reposRoot=${CONFIG.reposRoot} org=${CONFIG.githubOrg} origins=${CONFIG.allowedOrigins.join(
    ",",
  )}`,
);
