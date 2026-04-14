import type { VoiceConfig } from "./voices";

const BASE_URL = "https://api.openai.com/v1/audio/speech";

export async function synthesizeOpenAI(text: string, voice: VoiceConfig, apiKey: string): Promise<Buffer> {
  const resp = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model: voice.model ?? "tts-1",
      input: text,
      voice: voice.voiceId,
      response_format: "mp3",
      speed: voice.speed ?? 1.0,
    }),
  });

  if (!resp.ok) {
    const body = await resp.text().catch(() => "");
    throw new Error(`OpenAI TTS ${resp.status}: ${body.slice(0, 200)}`);
  }

  const ab = await resp.arrayBuffer();
  return Buffer.from(ab);
}
