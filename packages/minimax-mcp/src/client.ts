/**
 * MiniMax API Client
 *
 * Handles authentication and communication with MiniMax's APIs:
 * - M2 Text Generation (OpenAI-compatible)
 * - Hailuo Video Generation
 * - Speech-2.6 Text-to-Speech
 * - Music-2.0 Generation
 */

import axios, { AxiosInstance, AxiosError } from "axios";

// Base URLs for different MiniMax services
const MINIMAX_BASE_URL = "https://api.minimax.io/v1";

// API Response types
export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface ChatCompletionRequest {
  model?: string;
  messages: ChatMessage[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface ChatCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: Array<{
    index: number;
    message: ChatMessage;
    finish_reason: string;
  }>;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface VideoGenerationRequest {
  prompt: string;
  model?: string;
  first_frame_image?: string; // Base64 or URL for image-to-video
  duration?: number; // 6 or 10 seconds
  resolution?: "1080p" | "768p" | "512p";
}

export interface VideoGenerationResponse {
  task_id: string;
  base_resp: {
    status_code: number;
    status_msg: string;
  };
}

export interface TaskStatusResponse {
  task_id: string;
  status: "Queueing" | "Processing" | "Success" | "Fail";
  file_id?: string;
  base_resp: {
    status_code: number;
    status_msg: string;
  };
}

export interface SpeechRequest {
  text: string;
  model?: string;
  voice_id?: string;
  speed?: number;
  vol?: number;
  pitch?: number;
  emotion?: string;
  language?: string;
}

export interface SpeechResponse {
  audio_file?: string; // Base64 encoded audio
  extra_info?: {
    audio_length: number;
    audio_sample_rate: number;
    audio_size: number;
  };
  base_resp: {
    status_code: number;
    status_msg: string;
  };
}

export interface MusicGenerationRequest {
  prompt: string;
  duration?: number;
  refer_voice?: string; // Reference audio for style
  refer_instrumental?: string; // Reference instrumental
}

export interface MusicGenerationResponse {
  task_id: string;
  base_resp: {
    status_code: number;
    status_msg: string;
  };
}

export interface FileDownloadResponse {
  file: {
    file_id: string;
    bytes: number;
    created_at: number;
    filename: string;
    purpose: string;
    download_url?: string;
  };
  base_resp: {
    status_code: number;
    status_msg: string;
  };
}

export interface MinimaxClientConfig {
  apiKey: string;
  groupId?: string;
  baseUrl?: string;
}

export class MinimaxApiError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public statusMsg?: string
  ) {
    super(message);
    this.name = "MinimaxApiError";
  }
}

/**
 * MiniMax API Client
 */
export class MinimaxClient {
  private client: AxiosInstance;
  private groupId?: string;

  constructor(config: MinimaxClientConfig) {
    if (!config.apiKey) {
      throw new Error("MINIMAX_API_KEY is required");
    }

    this.groupId = config.groupId;

    this.client = axios.create({
      baseURL: config.baseUrl || MINIMAX_BASE_URL,
      headers: {
        Authorization: `Bearer ${config.apiKey}`,
        "Content-Type": "application/json",
      },
      timeout: 120000, // 2 minute timeout for long operations
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        if (error.response) {
          const data = error.response.data as Record<string, unknown>;
          const baseResp = data?.base_resp as Record<string, unknown> | undefined;
          throw new MinimaxApiError(
            baseResp?.status_msg as string || error.message,
            baseResp?.status_code as number || error.response.status,
            baseResp?.status_msg as string
          );
        }
        throw new MinimaxApiError(error.message);
      }
    );
  }

  /**
   * Chat completion using M2 model (OpenAI-compatible endpoint)
   */
  async chatCompletion(
    request: ChatCompletionRequest
  ): Promise<ChatCompletionResponse> {
    const response = await this.client.post<ChatCompletionResponse>(
      "/chat/completions",
      {
        model: request.model || "MiniMax-M2",
        messages: request.messages,
        temperature: request.temperature ?? 0.7,
        max_tokens: request.max_tokens ?? 4096,
        stream: false,
      }
    );
    return response.data;
  }

  /**
   * Generate video from text or image
   */
  async generateVideo(
    request: VideoGenerationRequest
  ): Promise<VideoGenerationResponse> {
    const payload: Record<string, unknown> = {
      model: request.model || "video-01", // Hailuo 2.3 model ID
      prompt: request.prompt,
    };

    if (request.first_frame_image) {
      payload.first_frame_image = request.first_frame_image;
    }

    const response = await this.client.post<VideoGenerationResponse>(
      "/video_generation",
      payload
    );

    if (response.data.base_resp.status_code !== 0) {
      throw new MinimaxApiError(
        response.data.base_resp.status_msg,
        response.data.base_resp.status_code
      );
    }

    return response.data;
  }

  /**
   * Check the status of an async task (video/music generation)
   */
  async checkTaskStatus(taskId: string): Promise<TaskStatusResponse> {
    const response = await this.client.get<TaskStatusResponse>(
      `/query/video_generation`,
      {
        params: { task_id: taskId },
      }
    );

    if (response.data.base_resp.status_code !== 0) {
      throw new MinimaxApiError(
        response.data.base_resp.status_msg,
        response.data.base_resp.status_code
      );
    }

    return response.data;
  }

  /**
   * Poll task status until completion or failure
   */
  async waitForTask(
    taskId: string,
    maxWaitMs: number = 300000, // 5 minutes default
    pollIntervalMs: number = 5000 // 5 seconds
  ): Promise<TaskStatusResponse> {
    const startTime = Date.now();

    while (Date.now() - startTime < maxWaitMs) {
      const status = await this.checkTaskStatus(taskId);

      if (status.status === "Success") {
        return status;
      }

      if (status.status === "Fail") {
        throw new MinimaxApiError(
          `Task ${taskId} failed: ${status.base_resp.status_msg}`,
          status.base_resp.status_code
        );
      }

      // Still processing, wait and retry
      await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));
    }

    throw new MinimaxApiError(
      `Task ${taskId} timed out after ${maxWaitMs}ms`
    );
  }

  /**
   * Text-to-speech synthesis
   */
  async textToSpeech(request: SpeechRequest): Promise<SpeechResponse> {
    const groupIdParam = this.groupId ? `?GroupId=${this.groupId}` : "";

    const payload: Record<string, unknown> = {
      model: request.model || "speech-02-hd",
      text: request.text,
      stream: false,
      voice_setting: {
        voice_id: request.voice_id || "male-qn-qingse",
        speed: request.speed ?? 1.0,
        vol: request.vol ?? 1.0,
        pitch: request.pitch ?? 0,
      },
      audio_setting: {
        sample_rate: 32000,
        bitrate: 128000,
        format: "mp3",
      },
    };

    // Add emotion if specified
    if (request.emotion) {
      (payload.voice_setting as Record<string, unknown>).emotion = request.emotion;
    }

    // Add language if specified
    if (request.language) {
      (payload as Record<string, unknown>).language = request.language;
    }

    const response = await this.client.post<SpeechResponse>(
      `/t2a_v2${groupIdParam}`,
      payload
    );

    if (response.data.base_resp.status_code !== 0) {
      throw new MinimaxApiError(
        response.data.base_resp.status_msg,
        response.data.base_resp.status_code
      );
    }

    return response.data;
  }

  /**
   * Generate music from text
   */
  async generateMusic(
    request: MusicGenerationRequest
  ): Promise<MusicGenerationResponse> {
    const groupIdParam = this.groupId ? `?GroupId=${this.groupId}` : "";

    const payload: Record<string, unknown> = {
      model: "music-01",
      lyrics: request.prompt,
    };

    if (request.refer_voice) {
      payload.refer_voice = request.refer_voice;
    }

    if (request.refer_instrumental) {
      payload.refer_instrumental = request.refer_instrumental;
    }

    const response = await this.client.post<MusicGenerationResponse>(
      `/music_generation${groupIdParam}`,
      payload
    );

    if (response.data.base_resp.status_code !== 0) {
      throw new MinimaxApiError(
        response.data.base_resp.status_msg,
        response.data.base_resp.status_code
      );
    }

    return response.data;
  }

  /**
   * Get file info and download URL
   */
  async getFile(fileId: string): Promise<FileDownloadResponse> {
    const response = await this.client.get<FileDownloadResponse>(
      `/files/retrieve`,
      {
        params: { file_id: fileId },
      }
    );

    if (response.data.base_resp.status_code !== 0) {
      throw new MinimaxApiError(
        response.data.base_resp.status_msg,
        response.data.base_resp.status_code
      );
    }

    return response.data;
  }

  /**
   * Download file content
   */
  async downloadFile(fileId: string): Promise<Buffer> {
    const fileInfo = await this.getFile(fileId);

    if (!fileInfo.file.download_url) {
      throw new MinimaxApiError(`No download URL available for file ${fileId}`);
    }

    const response = await axios.get<ArrayBuffer>(fileInfo.file.download_url, {
      responseType: "arraybuffer",
    });

    return Buffer.from(response.data);
  }
}

/**
 * Create a MiniMax client from environment variables
 */
export function createClientFromEnv(): MinimaxClient {
  const apiKey = process.env.MINIMAX_API_KEY;
  const groupId = process.env.MINIMAX_GROUP_ID;

  if (!apiKey) {
    throw new Error(
      "MINIMAX_API_KEY environment variable is required. " +
        "Get your API key from https://platform.minimax.io/"
    );
  }

  return new MinimaxClient({
    apiKey,
    groupId,
  });
}

