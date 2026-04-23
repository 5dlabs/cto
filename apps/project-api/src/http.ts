import { CONFIG } from "./config";

export function corsHeaders(origin: string | null): Record<string, string> {
  const allowed = CONFIG.allowedOrigins;
  const allowOrigin =
    allowed.includes("*")
      ? origin ?? "*"
      : origin && allowed.includes(origin)
        ? origin
        : allowed[0] ?? "*";
  return {
    "Access-Control-Allow-Origin": allowOrigin,
    "Access-Control-Allow-Credentials": "true",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "content-type, authorization",
    Vary: "Origin",
  };
}

export function json(
  body: unknown,
  init: ResponseInit & { origin?: string | null } = {},
): Response {
  const { origin, ...rest } = init;
  return new Response(JSON.stringify(body), {
    ...rest,
    headers: {
      "content-type": "application/json; charset=utf-8",
      ...corsHeaders(origin ?? null),
      ...(rest.headers || {}),
    },
  });
}

export function empty(status = 204, origin: string | null = null): Response {
  return new Response(null, {
    status,
    headers: corsHeaders(origin),
  });
}

export function problem(
  status: number,
  message: string,
  origin: string | null = null,
  extra: Record<string, unknown> = {},
): Response {
  return json({ error: message, ...extra }, { status, origin });
}
