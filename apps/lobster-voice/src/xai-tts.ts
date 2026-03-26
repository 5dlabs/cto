import type { VoiceConfig } from "./voices";

const BASE_URL = "https://api.x.ai/v1/tts";

export async function synthesizeXai(text: string, voice: VoiceConfig, apiKey: string): Promise<Buffer> {
  const resp = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      text,
      voice_id: voice.voiceId,
      language: voice.language ?? "en",
    }),
  });

  if (!resp.ok) {
    const body = await resp.text().catch(() => "");
    throw new Error(`xAI TTS ${resp.status}: ${body.slice(0, 200)}`);
  }

  const ab = await resp.arrayBuffer();
  return Buffer.from(ab);
}
