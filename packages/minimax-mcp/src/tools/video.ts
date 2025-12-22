/**
 * MiniMax Video Generation Tool
 *
 * Generate videos from text or images using Hailuo 2.3 model.
 * Supports up to 1080p resolution and 6-10 second durations.
 */

import { z } from "zod";
import { MinimaxClient } from "../client.js";

// Input schema for video generation
export const videoInputSchema = z.object({
  prompt: z.string().min(1).describe("Text description of the video to generate"),
  image_url: z
    .string()
    .optional()
    .describe("Optional base64 or URL of first frame image for image-to-video"),
  model: z
    .enum(["MiniMax-Hailuo-2.3", "MiniMax-Hailuo-2.3Fast", "video-01"])
    .optional()
    .default("video-01")
    .describe("Video model to use"),
  duration: z
    .number()
    .refine((n) => n === 6 || n === 10, {
      message: "Duration must be 6 or 10 seconds",
    })
    .optional()
    .default(6)
    .describe("Video duration in seconds (6 or 10)"),
  resolution: z
    .enum(["1080p", "768p", "512p"])
    .optional()
    .default("1080p")
    .describe("Video resolution"),
  wait_for_completion: z
    .boolean()
    .optional()
    .default(false)
    .describe("If true, wait for video generation to complete before returning"),
});

export type VideoInput = z.infer<typeof videoInputSchema>;

export interface VideoOutput {
  task_id: string;
  status: string;
  file_id?: string;
  download_url?: string;
  message: string;
}

/**
 * Execute video generation with Hailuo 2.3
 */
export async function executeVideoTool(
  client: MinimaxClient,
  input: VideoInput
): Promise<VideoOutput> {
  // Start video generation
  const response = await client.generateVideo({
    prompt: input.prompt,
    model: input.model,
    first_frame_image: input.image_url,
  });

  const taskId = response.task_id;

  // If not waiting, return immediately with task_id
  if (!input.wait_for_completion) {
    return {
      task_id: taskId,
      status: "Queueing",
      message:
        "Video generation started. Use minimax_check_task to poll for completion, " +
        "then minimax_download_file to retrieve the video.",
    };
  }

  // Wait for completion
  const status = await client.waitForTask(taskId);

  if (status.status === "Success" && status.file_id) {
    const fileInfo = await client.getFile(status.file_id);
    return {
      task_id: taskId,
      status: "Success",
      file_id: status.file_id,
      download_url: fileInfo.file.download_url,
      message: "Video generation completed successfully.",
    };
  }

  return {
    task_id: taskId,
    status: status.status,
    message: `Video generation status: ${status.status}`,
  };
}

export const videoToolDefinition = {
  name: "minimax_generate_video",
  description:
    "Generate video from text or image using MiniMax Hailuo 2.3. " +
    "Supports text-to-video and image-to-video generation up to 1080p resolution. " +
    "Returns a task_id for async polling - use minimax_check_task to monitor progress.",
  inputSchema: {
    type: "object" as const,
    properties: {
      prompt: {
        type: "string",
        description: "Text description of the video to generate",
      },
      image_url: {
        type: "string",
        description:
          "Optional: Base64 or URL of first frame image for image-to-video mode",
      },
      model: {
        type: "string",
        enum: ["MiniMax-Hailuo-2.3", "MiniMax-Hailuo-2.3Fast", "video-01"],
        default: "video-01",
        description: "Video model (default: video-01 / Hailuo 2.3)",
      },
      duration: {
        type: "number",
        enum: [6, 10],
        default: 6,
        description: "Video duration in seconds (6 or 10)",
      },
      resolution: {
        type: "string",
        enum: ["1080p", "768p", "512p"],
        default: "1080p",
        description: "Video resolution",
      },
      wait_for_completion: {
        type: "boolean",
        default: false,
        description: "If true, wait for generation to complete (may take minutes)",
      },
    },
    required: ["prompt"],
  },
};

// Task status check tool
export const checkTaskInputSchema = z.object({
  task_id: z.string().min(1).describe("Task ID to check status for"),
});

export type CheckTaskInput = z.infer<typeof checkTaskInputSchema>;

export interface TaskStatusOutput {
  task_id: string;
  status: string;
  file_id?: string;
  download_url?: string;
  message: string;
}

/**
 * Check status of an async task
 */
export async function executeCheckTaskTool(
  client: MinimaxClient,
  input: CheckTaskInput
): Promise<TaskStatusOutput> {
  const status = await client.checkTaskStatus(input.task_id);

  let downloadUrl: string | undefined;
  if (status.status === "Success" && status.file_id) {
    try {
      const fileInfo = await client.getFile(status.file_id);
      downloadUrl = fileInfo.file.download_url;
    } catch {
      // File info fetch failed, continue without download URL
    }
  }

  return {
    task_id: input.task_id,
    status: status.status,
    file_id: status.file_id,
    download_url: downloadUrl,
    message: getStatusMessage(status.status),
  };
}

function getStatusMessage(status: string): string {
  switch (status) {
    case "Queueing":
      return "Task is queued and waiting to start";
    case "Processing":
      return "Task is currently being processed";
    case "Success":
      return "Task completed successfully. Use file_id to download the result.";
    case "Fail":
      return "Task failed. Check the error message for details.";
    default:
      return `Unknown status: ${status}`;
  }
}

export const checkTaskToolDefinition = {
  name: "minimax_check_task",
  description:
    "Check the status of an async MiniMax task (video or music generation). " +
    "Returns the current status and file_id when complete.",
  inputSchema: {
    type: "object" as const,
    properties: {
      task_id: {
        type: "string",
        description: "The task_id returned from video or music generation",
      },
    },
    required: ["task_id"],
  },
};

