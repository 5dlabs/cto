/**
 * MiniMax Speech Tool
 *
 * Text-to-speech synthesis using Speech-2.6 model.
 * Supports 40 languages and 7 emotional tones.
 */

import { z } from "zod";
import { MinimaxClient } from "../client.js";

// Supported languages (subset - full list has 40+)
const SUPPORTED_LANGUAGES = [
  "en", // English
  "zh", // Chinese
  "ja", // Japanese
  "ko", // Korean
  "es", // Spanish
  "fr", // French
  "de", // German
  "it", // Italian
  "pt", // Portuguese
  "ru", // Russian
  "ar", // Arabic
  "hi", // Hindi
  "th", // Thai
  "vi", // Vietnamese
  "id", // Indonesian
  "ms", // Malay
  "nl", // Dutch
  "pl", // Polish
  "tr", // Turkish
  "sv", // Swedish
] as const;

// Supported emotions
const SUPPORTED_EMOTIONS = [
  "neutral",
  "happy",
  "sad",
  "angry",
  "fear",
  "disgust",
  "surprise",
] as const;

// Input schema for speech synthesis
export const speechInputSchema = z.object({
  text: z.string().min(1).max(10000).describe("Text to convert to speech (max 10000 chars)"),
  model: z
    .enum(["speech-2.6-hd", "speech-2.6-turbo", "speech-02-hd", "speech-02-turbo"])
    .optional()
    .default("speech-2.6-hd")
    .describe("Speech model (hd for quality, turbo for speed)"),
  voice_id: z
    .string()
    .optional()
    .describe("Voice identifier (e.g., 'male-qn-qingse', 'female-shaonv')"),
  language: z
    .enum(SUPPORTED_LANGUAGES)
    .optional()
    .default("en")
    .describe("Language code (40 languages supported)"),
  emotion: z
    .enum(SUPPORTED_EMOTIONS)
    .optional()
    .default("neutral")
    .describe("Emotional tone for the speech"),
  speed: z
    .number()
    .min(0.5)
    .max(2.0)
    .optional()
    .default(1.0)
    .describe("Speech speed (0.5-2.0, default: 1.0)"),
  volume: z
    .number()
    .min(0.1)
    .max(2.0)
    .optional()
    .default(1.0)
    .describe("Volume level (0.1-2.0, default: 1.0)"),
  pitch: z
    .number()
    .min(-12)
    .max(12)
    .optional()
    .default(0)
    .describe("Pitch adjustment in semitones (-12 to 12, default: 0)"),
});

export type SpeechInput = z.infer<typeof speechInputSchema>;

export interface SpeechOutput {
  audio_base64?: string;
  audio_length_seconds?: number;
  sample_rate?: number;
  file_size_bytes?: number;
  format: string;
  message: string;
}

/**
 * Execute text-to-speech synthesis
 */
export async function executeSpeechTool(
  client: MinimaxClient,
  input: SpeechInput
): Promise<SpeechOutput> {
  const response = await client.textToSpeech({
    text: input.text,
    model: input.model,
    voice_id: input.voice_id,
    language: input.language,
    emotion: input.emotion,
    speed: input.speed,
    vol: input.volume,
    pitch: input.pitch,
  });

  if (!response.audio_file) {
    return {
      format: "mp3",
      message: "Speech synthesis completed but no audio data returned.",
    };
  }

  return {
    audio_base64: response.audio_file,
    audio_length_seconds: response.extra_info?.audio_length,
    sample_rate: response.extra_info?.audio_sample_rate,
    file_size_bytes: response.extra_info?.audio_size,
    format: "mp3",
    message: "Speech synthesis completed successfully.",
  };
}

export const speechToolDefinition = {
  name: "minimax_text_to_speech",
  description:
    "Convert text to natural speech using MiniMax Speech-2.6. " +
    "Supports 40 languages and 7 emotional tones (neutral, happy, sad, angry, fear, disgust, surprise). " +
    "Returns base64-encoded MP3 audio.",
  inputSchema: {
    type: "object" as const,
    properties: {
      text: {
        type: "string",
        maxLength: 10000,
        description: "Text to convert to speech (max 10000 characters)",
      },
      model: {
        type: "string",
        enum: ["speech-2.6-hd", "speech-2.6-turbo", "speech-02-hd", "speech-02-turbo"],
        default: "speech-2.6-hd",
        description: "Speech model (hd for quality, turbo for speed)",
      },
      voice_id: {
        type: "string",
        description: "Voice identifier (e.g., 'male-qn-qingse', 'female-shaonv')",
      },
      language: {
        type: "string",
        enum: SUPPORTED_LANGUAGES,
        default: "en",
        description: "Language code (en, zh, ja, ko, es, fr, de, etc.)",
      },
      emotion: {
        type: "string",
        enum: SUPPORTED_EMOTIONS,
        default: "neutral",
        description: "Emotional tone for the speech",
      },
      speed: {
        type: "number",
        minimum: 0.5,
        maximum: 2.0,
        default: 1.0,
        description: "Speech speed (0.5-2.0)",
      },
      volume: {
        type: "number",
        minimum: 0.1,
        maximum: 2.0,
        default: 1.0,
        description: "Volume level (0.1-2.0)",
      },
      pitch: {
        type: "number",
        minimum: -12,
        maximum: 12,
        default: 0,
        description: "Pitch adjustment in semitones (-12 to 12)",
      },
    },
    required: ["text"],
  },
};

