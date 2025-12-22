/**
 * MiniMax Chat Tool
 *
 * Text generation using MiniMax-M2 model with 200k context window
 * and 128k output capacity. OpenAI-compatible API.
 */

import { z } from "zod";
import { MinimaxClient, ChatMessage } from "../client.js";

// Input schema for chat tool
export const chatInputSchema = z.object({
  messages: z
    .array(
      z.object({
        role: z.enum(["system", "user", "assistant"]),
        content: z.string(),
      })
    )
    .min(1)
    .describe("Array of chat messages with role and content"),
  model: z
    .string()
    .optional()
    .default("MiniMax-M2")
    .describe("Model to use (default: MiniMax-M2)"),
  temperature: z
    .number()
    .min(0)
    .max(1)
    .optional()
    .default(0.7)
    .describe("Sampling temperature (0.0-1.0, default: 0.7)"),
  max_tokens: z
    .number()
    .positive()
    .optional()
    .default(4096)
    .describe("Maximum tokens to generate (default: 4096, max: 128000)"),
});

export type ChatInput = z.infer<typeof chatInputSchema>;

export interface ChatOutput {
  content: string;
  model: string;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  finish_reason: string;
}

/**
 * Execute chat completion with MiniMax M2
 */
export async function executeChatTool(
  client: MinimaxClient,
  input: ChatInput
): Promise<ChatOutput> {
  const response = await client.chatCompletion({
    model: input.model,
    messages: input.messages as ChatMessage[],
    temperature: input.temperature,
    max_tokens: input.max_tokens,
  });

  const choice = response.choices[0];

  return {
    content: choice.message.content,
    model: response.model,
    usage: response.usage,
    finish_reason: choice.finish_reason,
  };
}

export const chatToolDefinition = {
  name: "minimax_chat",
  description:
    "Generate text using MiniMax-M2 model. Features 200k context window and up to 128k output tokens. " +
    "Supports advanced reasoning, function calling, and real-time streaming. " +
    "Compatible with OpenAI API format.",
  inputSchema: {
    type: "object" as const,
    properties: {
      messages: {
        type: "array",
        items: {
          type: "object",
          properties: {
            role: {
              type: "string",
              enum: ["system", "user", "assistant"],
              description: "The role of the message sender",
            },
            content: {
              type: "string",
              description: "The content of the message",
            },
          },
          required: ["role", "content"],
        },
        minItems: 1,
        description: "Array of chat messages",
      },
      model: {
        type: "string",
        default: "MiniMax-M2",
        description: "Model to use (default: MiniMax-M2)",
      },
      temperature: {
        type: "number",
        minimum: 0,
        maximum: 1,
        default: 0.7,
        description: "Sampling temperature (0.0-1.0)",
      },
      max_tokens: {
        type: "number",
        minimum: 1,
        maximum: 128000,
        default: 4096,
        description: "Maximum tokens to generate",
      },
    },
    required: ["messages"],
  },
};

