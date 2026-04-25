import { createHash } from "crypto";
import { readFile } from "fs/promises";
import { resolve } from "path";
import { NextRequest, NextResponse } from "next/server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const ELEVENLABS_BASE = "https://api.elevenlabs.io/v1";
const FALLBACK_AUDIO = resolve(process.cwd(), "../../voice_clone_sample.mp3");
const TTS_MODEL_ID = "eleven_flash_v2_5";
const TTS_OUTPUT_FORMAT = "mp3_44100_128";
const CACHE_TTL_MS = 10 * 60 * 1000;
const MAX_CACHE_ENTRIES = 48;
const MAX_CACHE_BYTES = 24 * 1024 * 1024;
const DEFAULT_RATE_LIMIT_BACKOFF_MS = 60 * 1000;
const MAX_RATE_LIMIT_BACKOFF_MS = 5 * 60 * 1000;

type TtsCacheEntry = {
  buffer: ArrayBuffer;
  contentType: string;
  createdAt: number;
};

type TtsGlobals = {
  __echoTurnTtsCache?: Map<string, TtsCacheEntry>;
  __echoTurnTtsCacheBytes?: number;
  __echoTurnTtsRateLimitedUntil?: number;
};

const ttsGlobals = globalThis as unknown as TtsGlobals;
const ttsCache = ttsGlobals.__echoTurnTtsCache ?? new Map<string, TtsCacheEntry>();
ttsGlobals.__echoTurnTtsCache = ttsCache;
ttsGlobals.__echoTurnTtsCacheBytes ??= 0;

function fallbackHeaders(reason: string, retryAfterSeconds?: number) {
  const headers: Record<string, string> = {
    "Cache-Control": "no-store",
    "Content-Type": "audio/mpeg",
    "X-Morgan-TTS-Fallback": "voice_clone_sample.mp3",
    "X-Morgan-TTS-Fallback-Reason": reason,
  };
  if (retryAfterSeconds !== undefined) {
    headers["Retry-After"] = String(retryAfterSeconds);
  }
  return headers;
}

async function fallbackAudioResponse(reason = "fallback", retryAfterSeconds?: number) {
  const audio = await readFile(FALLBACK_AUDIO);
  return new Response(new Uint8Array(audio), {
    headers: fallbackHeaders(reason, retryAfterSeconds),
  });
}

function cacheKey(voiceId: string, text: string) {
  return createHash("sha256")
    .update(voiceId)
    .update("\0")
    .update(TTS_MODEL_ID)
    .update("\0")
    .update(TTS_OUTPUT_FORMAT)
    .update("\0")
    .update(text)
    .digest("hex");
}

function cacheSize(entry: TtsCacheEntry) {
  return entry.buffer.byteLength;
}

function pruneCache(now = Date.now()) {
  for (const [key, entry] of ttsCache) {
    if (now - entry.createdAt > CACHE_TTL_MS) {
      ttsGlobals.__echoTurnTtsCacheBytes =
        (ttsGlobals.__echoTurnTtsCacheBytes ?? 0) - cacheSize(entry);
      ttsCache.delete(key);
    }
  }

  while (
    ttsCache.size > MAX_CACHE_ENTRIES ||
    (ttsGlobals.__echoTurnTtsCacheBytes ?? 0) > MAX_CACHE_BYTES
  ) {
    const oldestKey = ttsCache.keys().next().value as string | undefined;
    if (!oldestKey) {
      break;
    }
    const oldest = ttsCache.get(oldestKey);
    if (oldest) {
      ttsGlobals.__echoTurnTtsCacheBytes =
        (ttsGlobals.__echoTurnTtsCacheBytes ?? 0) - cacheSize(oldest);
    }
    ttsCache.delete(oldestKey);
  }
}

function getCachedAudio(key: string) {
  pruneCache();
  const entry = ttsCache.get(key);
  if (!entry) {
    return null;
  }
  ttsCache.delete(key);
  ttsCache.set(key, entry);
  return entry;
}

function setCachedAudio(key: string, entry: TtsCacheEntry) {
  const existing = ttsCache.get(key);
  if (existing) {
    ttsGlobals.__echoTurnTtsCacheBytes =
      (ttsGlobals.__echoTurnTtsCacheBytes ?? 0) - cacheSize(existing);
  }
  ttsCache.set(key, entry);
  ttsGlobals.__echoTurnTtsCacheBytes =
    (ttsGlobals.__echoTurnTtsCacheBytes ?? 0) + cacheSize(entry);
  pruneCache();
}

async function cacheStream(key: string, body: ReadableStream<Uint8Array>, contentType: string) {
  try {
    const buffer = await new Response(body).arrayBuffer();
    if (buffer.byteLength <= MAX_CACHE_BYTES) {
      setCachedAudio(key, { buffer, contentType, createdAt: Date.now() });
    }
  } catch (error) {
    console.warn(
      "[echo-turn/tts] failed to cache ElevenLabs audio",
      error instanceof Error ? error.message : error,
    );
  }
}

function cachedAudioResponse(entry: TtsCacheEntry) {
  return new Response(new Uint8Array(entry.buffer), {
    headers: {
      "Cache-Control": "no-store",
      "Content-Type": entry.contentType,
      "X-Morgan-TTS-Cache": "hit",
    },
  });
}

function retryAfterMs(headerValue: string | null) {
  if (!headerValue) {
    return DEFAULT_RATE_LIMIT_BACKOFF_MS;
  }

  const seconds = Number(headerValue);
  if (Number.isFinite(seconds) && seconds > 0) {
    return Math.min(seconds * 1000, MAX_RATE_LIMIT_BACKOFF_MS);
  }

  const dateMs = Date.parse(headerValue);
  if (Number.isFinite(dateMs)) {
    return Math.min(Math.max(dateMs - Date.now(), 0), MAX_RATE_LIMIT_BACKOFF_MS);
  }

  return DEFAULT_RATE_LIMIT_BACKOFF_MS;
}

export async function POST(request: NextRequest) {
  const body = (await request.json().catch(() => ({}))) as { text?: string };
  const text = typeof body.text === "string" ? body.text.trim() : "";
  if (!text) {
    return NextResponse.json({ error: "text is required" }, { status: 400 });
  }

  const apiKey = process.env.ELEVENLABS_API_KEY?.trim();
  const voiceId = process.env.MORGAN_VOICE_ID?.trim() || "iP95p4xoKVk53GoZ742B";
  const key = cacheKey(voiceId, text);
  const cached = getCachedAudio(key);
  if (cached) {
    return cachedAudioResponse(cached);
  }

  if (!apiKey || process.env.MORGAN_DEMO_FORCE_STUB === "1") {
    return fallbackAudioResponse(!apiKey ? "missing-api-key" : "forced-stub");
  }

  const now = Date.now();
  const rateLimitedUntil = ttsGlobals.__echoTurnTtsRateLimitedUntil ?? 0;
  if (rateLimitedUntil > now) {
    const retryAfterSeconds = Math.ceil((rateLimitedUntil - now) / 1000);
    return fallbackAudioResponse("rate-limit-backoff", retryAfterSeconds);
  }

  const response = await fetch(`${ELEVENLABS_BASE}/text-to-speech/${voiceId}/stream`, {
    method: "POST",
    headers: {
      Accept: "audio/mpeg",
      "Content-Type": "application/json",
      "xi-api-key": apiKey,
    },
    body: JSON.stringify({
      model_id: TTS_MODEL_ID,
      output_format: TTS_OUTPUT_FORMAT,
      text,
    }),
    signal: AbortSignal.timeout(60_000),
  });

  if (!response.ok || !response.body) {
    if (response.status === 429) {
      const backoffMs = retryAfterMs(response.headers.get("retry-after"));
      ttsGlobals.__echoTurnTtsRateLimitedUntil = Date.now() + backoffMs;
      const retryAfterSeconds = Math.ceil(backoffMs / 1000);
      console.warn(
        `[echo-turn/tts] ElevenLabs rate limited; using fallback for ${retryAfterSeconds}s`,
      );
      return fallbackAudioResponse("rate-limited", retryAfterSeconds);
    }

    console.warn(`[echo-turn/tts] ElevenLabs returned ${response.status}; using fallback`);
    return fallbackAudioResponse(`upstream-${response.status}`);
  }

  const contentType = response.headers.get("content-type") || "audio/mpeg";
  const [clientBody, cacheBody] = response.body.tee();
  void cacheStream(key, cacheBody, contentType);

  return new Response(clientBody, {
    headers: {
      "Cache-Control": "no-store",
      "Content-Type": contentType,
      "X-Morgan-TTS-Cache": "miss",
    },
  });
}
