import type { VoiceConfig } from "./voices";

const BASE_URL = "https://api.elevenlabs.io/v1";

export interface TtsOptions {
  text: string;
  voice: VoiceConfig;
  apiKey: string;
}

export async function synthesize(opts: TtsOptions): Promise<Buffer> {
  const url = `${BASE_URL}/text-to-speech/${opts.voice.voiceId}?output_format=mp3_22050_32`;

  const resp = await fetch(url, {
    method: "POST",
    headers: {
      "xi-api-key": opts.apiKey,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      text: opts.text,
      model_id: opts.voice.model ?? "eleven_flash_v2_5",
      apply_text_normalization: "on",
      voice_settings: {
        stability: opts.voice.stability ?? 0.5,
        similarity_boost: opts.voice.similarityBoost ?? 0.75,
        style: opts.voice.style ?? 0.3,
        use_speaker_boost: true,
      },
    }),
  });

  if (!resp.ok) {
    const body = await resp.text().catch(() => "");
    throw new Error(`ElevenLabs API ${resp.status}: ${body.slice(0, 200)}`);
  }

  const ab = await resp.arrayBuffer();
  return Buffer.from(ab);
}
