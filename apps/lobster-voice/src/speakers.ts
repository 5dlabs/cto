import type { VoiceConfig } from "./voices";

export type SpeakerId =
  | "narrator"
  | "optimist"
  | "pessimist"
  | "voter-1"
  | "voter-2"
  | "voter-3"
  | "voter-4"
  | "voter-5"
  | "alert"
  | "system"
  | "compiler";

export interface SpeakerConfig {
  id: SpeakerId;
  label: string;
  voice: VoiceConfig;
}

const SPEAKERS: Record<SpeakerId, SpeakerConfig> = {
  narrator: {
    id: "narrator",
    label: "Narrator (Nova — OpenAI)",
    voice: {
      provider: "openai",
      voiceId: "nova",
      model: "tts-1-hd",
      speed: 1.0,
    },
  },
  optimist: {
    id: "optimist",
    label: "Optimist (Charlie — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "IKne3meq5aSn9XLyUdCD",
      model: "eleven_flash_v2_5",
      stability: 0.5,
      similarityBoost: 0.8,
      style: 0.4,
    },
  },
  pessimist: {
    id: "pessimist",
    label: "Pessimist (Ara — xAI)",
    voice: {
      provider: "xai",
      voiceId: "ara",
      language: "en",
    },
  },
  "voter-1": {
    id: "voter-1",
    label: "Architect (Daniel — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "onwK4e9ZLuTAKqWW03F9",
      model: "eleven_flash_v2_5",
      stability: 0.65,
      similarityBoost: 0.75,
      style: 0.3,
    },
  },
  "voter-2": {
    id: "voter-2",
    label: "Pragmatist (Shimmer — OpenAI)",
    voice: {
      provider: "openai",
      voiceId: "shimmer",
      model: "tts-1-hd",
      speed: 1.0,
    },
  },
  "voter-3": {
    id: "voter-3",
    label: "Minimalist (Sal — xAI)",
    voice: {
      provider: "xai",
      voiceId: "sal",
      language: "en",
    },
  },
  "voter-4": {
    id: "voter-4",
    label: "Operator (Matilda — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "XrExE9yKIg1WjnnlVkGX",
      model: "eleven_flash_v2_5",
      stability: 0.5,
      similarityBoost: 0.8,
      style: 0.35,
    },
  },
  "voter-5": {
    id: "voter-5",
    label: "Strategist (Leo — xAI)",
    voice: {
      provider: "xai",
      voiceId: "leo",
      language: "en",
    },
  },
  alert: {
    id: "alert",
    label: "Alert (Rex — xAI)",
    voice: {
      provider: "xai",
      voiceId: "rex",
      language: "en",
    },
  },
  system: {
    id: "system",
    label: "System (Alloy — OpenAI)",
    voice: {
      provider: "openai",
      voiceId: "alloy",
      model: "tts-1",
      speed: 1.0,
    },
  },
  compiler: {
    id: "compiler",
    label: "Compiler (Adam — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "pNInz6obpgDQGcFmaJgB",
      model: "eleven_flash_v2_5",
      stability: 0.6,
      similarityBoost: 0.75,
      style: 0.25,
    },
  },
};

const ALL_IDS = Object.keys(SPEAKERS) as SpeakerId[];

export function getSpeaker(id: string): SpeakerConfig | undefined {
  return SPEAKERS[id as SpeakerId];
}

export function isSpeakerId(id: string): id is SpeakerId {
  return id in SPEAKERS;
}

export function allSpeakerIds(): SpeakerId[] {
  return ALL_IDS;
}
