/**
 * MiniMax File Download Tool
 *
 * Download generated files (videos, music) by file_id.
 */

import { z } from "zod";
import { MinimaxClient } from "../client.js";
import * as fs from "fs/promises";
import * as path from "path";

// Input schema for file download
export const fileDownloadInputSchema = z.object({
  file_id: z.string().min(1).describe("File ID from completed task"),
  output_path: z
    .string()
    .optional()
    .describe("Optional local path to save the file"),
  return_base64: z
    .boolean()
    .optional()
    .default(false)
    .describe("If true, return file content as base64 (for small files)"),
});

export type FileDownloadInput = z.infer<typeof fileDownloadInputSchema>;

export interface FileDownloadOutput {
  file_id: string;
  filename: string;
  size_bytes: number;
  download_url?: string;
  saved_path?: string;
  base64_content?: string;
  message: string;
}

/**
 * Execute file download
 */
export async function executeFileDownloadTool(
  client: MinimaxClient,
  input: FileDownloadInput
): Promise<FileDownloadOutput> {
  // Get file info first
  const fileInfo = await client.getFile(input.file_id);
  const { file } = fileInfo;

  const result: FileDownloadOutput = {
    file_id: file.file_id,
    filename: file.filename,
    size_bytes: file.bytes,
    download_url: file.download_url,
    message: "File info retrieved successfully.",
  };

  // If output path specified, download and save
  if (input.output_path) {
    const buffer = await client.downloadFile(input.file_id);

    // Ensure directory exists
    const dir = path.dirname(input.output_path);
    await fs.mkdir(dir, { recursive: true });

    // Write file
    await fs.writeFile(input.output_path, buffer);
    result.saved_path = input.output_path;
    result.message = `File downloaded and saved to ${input.output_path}`;
  }

  // If base64 requested (useful for small files or inline use)
  if (input.return_base64) {
    const buffer = await client.downloadFile(input.file_id);
    result.base64_content = buffer.toString("base64");
    result.message = "File downloaded and converted to base64.";
  }

  return result;
}

export const fileDownloadToolDefinition = {
  name: "minimax_download_file",
  description:
    "Download a generated file (video, music) by its file_id. " +
    "Can save to a local path or return as base64. " +
    "Use this after minimax_check_task returns Success status.",
  inputSchema: {
    type: "object" as const,
    properties: {
      file_id: {
        type: "string",
        description: "File ID from a completed generation task",
      },
      output_path: {
        type: "string",
        description: "Optional: Local file path to save the downloaded file",
      },
      return_base64: {
        type: "boolean",
        default: false,
        description: "If true, return file content as base64 (for small files only)",
      },
    },
    required: ["file_id"],
  },
};

