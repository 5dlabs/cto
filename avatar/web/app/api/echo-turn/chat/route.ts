import { NextRequest } from "next/server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const encoder = new TextEncoder();

function sse(payload: unknown): Uint8Array {
  return encoder.encode(`data: ${JSON.stringify(payload)}\n\n`);
}

function fallbackReply(message: string): string {
  const subject = message.trim() || "this";
  return `Absolutely. I can talk through ${subject}, keep the timing conversational, and stay visibly Morgan the golden retriever while the avatar video renders for this turn.`;
}

async function writeFallbackStream(
  controller: ReadableStreamDefaultController<Uint8Array>,
  message: string,
) {
  const words = fallbackReply(message).split(/(\s+)/);
  for (const word of words) {
    if (!word) {
      continue;
    }
    controller.enqueue(sse({ type: "delta", text: word }));
    await new Promise((resolve) => setTimeout(resolve, word.trim() ? 45 : 10));
  }
  controller.enqueue(sse({ type: "done" }));
}

function extractDelta(raw: string): string {
  try {
    const payload = JSON.parse(raw) as {
      choices?: Array<{
        delta?: { content?: string };
        message?: { content?: string };
      }>;
    };
    const choice = payload.choices?.[0];
    return choice?.delta?.content ?? choice?.message?.content ?? "";
  } catch {
    return "";
  }
}

async function proxyGatewayStream(
  controller: ReadableStreamDefaultController<Uint8Array>,
  message: string,
) {
  const gatewayUrl =
    process.env.MORGAN_GATEWAY_URL?.trim() || process.env.MORGAN_LLM_BASE_URL?.trim();
  const model = process.env.MORGAN_MODEL?.trim() || "openclaw/morgan";
  const token =
    process.env.MORGAN_GATEWAY_TOKEN?.trim() || process.env.OPENCLAW_TOKEN?.trim();
  const agentId = process.env.MORGAN_LLM_AGENT_ID?.trim();

  if (!gatewayUrl || process.env.MORGAN_DEMO_FORCE_STUB === "1") {
    await writeFallbackStream(controller, message);
    return;
  }

  const headers: Record<string, string> = {
    Accept: "text/event-stream",
    "Content-Type": "application/json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  if (agentId) {
    headers["x-openclaw-agent-id"] = agentId;
  }

  const response = await fetch(`${gatewayUrl.replace(/\/$/, "")}/v1/chat/completions`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      model,
      stream: true,
      messages: [
        {
          role: "system",
          content:
            "You are Morgan, a concise CTO companion. Keep replies short enough for avatar turn rendering.",
        },
        { role: "user", content: message },
      ],
      user: `morgan-echo-turn:${crypto.randomUUID()}`,
    }),
    signal: AbortSignal.timeout(120_000),
  });

  if (!response.ok || !response.body) {
    await writeFallbackStream(controller, message);
    return;
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split(/\r?\n/);
    buffer = lines.pop() ?? "";
    for (const line of lines) {
      if (!line.startsWith("data:")) {
        continue;
      }
      const raw = line.slice(5).trim();
      if (!raw || raw === "[DONE]") {
        continue;
      }
      const text = extractDelta(raw);
      if (text) {
        controller.enqueue(sse({ type: "delta", text }));
      }
    }
  }

  controller.enqueue(sse({ type: "done" }));
}

export async function POST(request: NextRequest) {
  const body = (await request.json().catch(() => ({}))) as { message?: string };
  const message = typeof body.message === "string" ? body.message : "";

  const stream = new ReadableStream<Uint8Array>({
    async start(controller) {
      try {
        await proxyGatewayStream(controller, message);
      } catch (error) {
        controller.enqueue(
          sse({
            type: "error",
            message: error instanceof Error ? error.message : "Morgan stream failed.",
          }),
        );
      } finally {
        controller.close();
      }
    },
  });

  return new Response(stream, {
    headers: {
      "Cache-Control": "no-store",
      "Content-Type": "text/event-stream; charset=utf-8",
      "X-Accel-Buffering": "no",
    },
  });
}
