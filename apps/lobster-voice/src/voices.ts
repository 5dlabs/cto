export type VoiceLevel = "normal" | "error" | "wait";
export type TtsProvider = "elevenlabs" | "openai" | "xai";

export interface VoiceConfig {
  provider: TtsProvider;
  voiceId: string;
  model?: string;
  stability?: number;
  similarityBoost?: number;
  style?: number;
  speed?: number;
  language?: string;
}

const DEFAULTS: Record<VoiceLevel, VoiceConfig> = {
  normal: {
    provider: "elevenlabs",
    voiceId: "pFZP5JQG7iQjIQuC4Bku",
    model: "eleven_flash_v2_5",
    stability: 0.55,
    similarityBoost: 0.75,
    style: 0.3,
  },
  error: {
    provider: "xai",
    voiceId: "rex",
    language: "en",
  },
  wait: {
    provider: "openai",
    voiceId: "alloy",
    model: "tts-1",
    speed: 1.0,
  },
};

export function getVoiceConfig(level: VoiceLevel): VoiceConfig {
  const envPrefix = `LOBSTER_VOICE_${level.toUpperCase()}`;
  const voiceId = process.env[envPrefix] ?? DEFAULTS[level].voiceId;
  const model = process.env.LOBSTER_VOICE_MODEL ?? DEFAULTS[level].model;

  return {
    ...DEFAULTS[level],
    voiceId,
    model,
  };
}

export function resolveLevel(level: string | undefined): VoiceLevel {
  if (level === "error") return "error";
  if (level === "wait") return "wait";
  return "normal";
}
