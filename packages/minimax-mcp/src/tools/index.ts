/**
 * MiniMax MCP Tools - Export all tools
 */

export {
  chatInputSchema,
  chatToolDefinition,
  executeChatTool,
  type ChatInput,
  type ChatOutput,
} from "./chat.js";

export {
  videoInputSchema,
  videoToolDefinition,
  executeVideoTool,
  checkTaskInputSchema,
  checkTaskToolDefinition,
  executeCheckTaskTool,
  type VideoInput,
  type VideoOutput,
  type CheckTaskInput,
  type TaskStatusOutput,
} from "./video.js";

export {
  speechInputSchema,
  speechToolDefinition,
  executeSpeechTool,
  type SpeechInput,
  type SpeechOutput,
} from "./speech.js";

export {
  musicInputSchema,
  musicToolDefinition,
  executeMusicTool,
  type MusicInput,
  type MusicOutput,
} from "./music.js";

export {
  fileDownloadInputSchema,
  fileDownloadToolDefinition,
  executeFileDownloadTool,
  type FileDownloadInput,
  type FileDownloadOutput,
} from "./file.js";

