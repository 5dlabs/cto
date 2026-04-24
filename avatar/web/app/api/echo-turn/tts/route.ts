import { readFile } from "fs/promises";
import { resolve } from "path";
import { NextRequest, NextResponse } from "next/server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const ELEVENLABS_BASE = "https://api.elevenlabs.io/v1";
const FALLBACK_AUDIO = resolve(process.cwd(), "../../voice_clone_sample.mp3");

async function fallbackAudioResponse() {
  const audio = await readFile(FALLBACK_AUDIO);
  return new Response(new Uint8Array(audio), {
    headers: {
      "Cache-Control": "no-store",
      "Content-Type": "audio/mpeg",
      "X-Morgan-TTS-Fallback": "voice_clone_sample.mp3",
    },
  });
}

export async function POST(request: NextRequest) {
  const body = (await request.json().catch(() => ({}))) as { text?: string };
  const text = typeof body.text === "string" ? body.text.trim() : "";
  if (!text) {
    return NextResponse.json({ error: "text is required" }, { status: 400 });
  }

  const apiKey = process.env.ELEVENLABS_API_KEY?.trim();
  const voiceId = process.env.MORGAN_VOICE_ID?.trim() || "iP95p4xoKVk53GoZ742B";
  if (!apiKey || process.env.MORGAN_DEMO_FORCE_STUB === "1") {
    return fallbackAudioResponse();
  }

  const response = await fetch(`${ELEVENLABS_BASE}/text-to-speech/${voiceId}/stream`, {
    method: "POST",
    headers: {
      Accept: "audio/mpeg",
      "Content-Type": "application/json",
      "xi-api-key": apiKey,
    },
    body: JSON.stringify({
      model_id: "eleven_flash_v2_5",
      output_format: "mp3_44100_128",
      text,
    }),
    signal: AbortSignal.timeout(60_000),
  });

  if (!response.ok || !response.body) {
    return fallbackAudioResponse();
  }

  return new Response(response.body, {
    headers: {
      "Cache-Control": "no-store",
      "Content-Type": "audio/mpeg",
    },
  });
}
