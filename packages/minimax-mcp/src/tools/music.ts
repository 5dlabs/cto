/**
 * MiniMax Music Generation Tool
 *
 * Generate music from text descriptions using Music-2.0 model.
 * Supports human-like performances with vocals and emotional expression.
 */

import { z } from "zod";
import { MinimaxClient } from "../client.js";

// Input schema for music generation
export const musicInputSchema = z.object({
  prompt: z
    .string()
    .min(1)
    .describe("Text description of the music to generate (lyrics or style description)"),
  duration: z
    .number()
    .positive()
    .optional()
    .describe("Desired duration in seconds (actual may vary)"),
  include_vocals: z
    .boolean()
    .optional()
    .default(true)
    .describe("Whether to include AI-generated vocals"),
  reference_audio_url: z
    .string()
    .optional()
    .describe("Optional URL to reference audio for style matching"),
  wait_for_completion: z
    .boolean()
    .optional()
    .default(false)
    .describe("If true, wait for generation to complete before returning"),
});

export type MusicInput = z.infer<typeof musicInputSchema>;

export interface MusicOutput {
  task_id: string;
  status: string;
  file_id?: string;
  download_url?: string;
  message: string;
}

/**
 * Execute music generation with Music-2.0
 */
export async function executeMusicTool(
  client: MinimaxClient,
  input: MusicInput
): Promise<MusicOutput> {
  // Start music generation
  const response = await client.generateMusic({
    prompt: input.prompt,
    duration: input.duration,
    refer_voice: input.include_vocals ? undefined : "instrumental",
    refer_instrumental: input.reference_audio_url,
  });

  const taskId = response.task_id;

  // If not waiting, return immediately with task_id
  if (!input.wait_for_completion) {
    return {
      task_id: taskId,
      status: "Queueing",
      message:
        "Music generation started. Use minimax_check_task to poll for completion, " +
        "then minimax_download_file to retrieve the audio.",
    };
  }

  // Wait for completion (music generation can take longer)
  const status = await client.waitForTask(taskId, 600000); // 10 minute timeout

  if (status.status === "Success" && status.file_id) {
    const fileInfo = await client.getFile(status.file_id);
    return {
      task_id: taskId,
      status: "Success",
      file_id: status.file_id,
      download_url: fileInfo.file.download_url,
      message: "Music generation completed successfully.",
    };
  }

  return {
    task_id: taskId,
    status: status.status,
    message: `Music generation status: ${status.status}`,
  };
}

export const musicToolDefinition = {
  name: "minimax_generate_music",
  description:
    "Generate music from text description using MiniMax Music-2.0. " +
    "Creates human-like performances with rich emotional expression and realistic vocals. " +
    "Returns a task_id for async polling - use minimax_check_task to monitor progress.",
  inputSchema: {
    type: "object" as const,
    properties: {
      prompt: {
        type: "string",
        description:
          "Text description of the music to generate. Can be lyrics or a style description " +
          "(e.g., 'An upbeat electronic dance track with synth melodies' or actual song lyrics)",
      },
      duration: {
        type: "number",
        minimum: 5,
        maximum: 300,
        description: "Desired duration in seconds (actual duration may vary)",
      },
      include_vocals: {
        type: "boolean",
        default: true,
        description: "Whether to include AI-generated vocals",
      },
      reference_audio_url: {
        type: "string",
        description: "Optional URL to reference audio for style matching",
      },
      wait_for_completion: {
        type: "boolean",
        default: false,
        description: "If true, wait for generation to complete (may take several minutes)",
      },
    },
    required: ["prompt"],
  },
};

