#!/usr/bin/env node
/**
 * MiniMax MCP Server
 *
 * Model Context Protocol server for MiniMax AI suite:
 * - MiniMax-M2: Text generation (200k context, 128k output)
 * - Hailuo 2.3: Video generation (text-to-video, image-to-video)
 * - Speech-2.6: Text-to-speech (40 languages, 7 emotions)
 * - Music-2.0: Music generation with vocals
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ErrorCode,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";

import { createClientFromEnv, MinimaxClient, MinimaxApiError } from "./client.js";
import {
  chatToolDefinition,
  chatInputSchema,
  executeChatTool,
  videoToolDefinition,
  videoInputSchema,
  executeVideoTool,
  checkTaskToolDefinition,
  checkTaskInputSchema,
  executeCheckTaskTool,
  speechToolDefinition,
  speechInputSchema,
  executeSpeechTool,
  musicToolDefinition,
  musicInputSchema,
  executeMusicTool,
  fileDownloadToolDefinition,
  fileDownloadInputSchema,
  executeFileDownloadTool,
} from "./tools/index.js";

// Server metadata
const SERVER_NAME = "minimax-mcp";
const SERVER_VERSION = "0.1.0";

/**
 * Main MCP server class
 */
class MinimaxMcpServer {
  private server: Server;
  private client: MinimaxClient;

  constructor() {
    // Initialize MiniMax client from environment
    this.client = createClientFromEnv();

    // Create MCP server
    this.server = new Server(
      {
        name: SERVER_NAME,
        version: SERVER_VERSION,
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupHandlers();
    this.setupErrorHandling();
  }

  /**
   * Set up request handlers
   */
  private setupHandlers(): void {
    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        chatToolDefinition,
        videoToolDefinition,
        checkTaskToolDefinition,
        speechToolDefinition,
        musicToolDefinition,
        fileDownloadToolDefinition,
      ],
    }));

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case "minimax_chat": {
            const input = chatInputSchema.parse(args);
            const result = await executeChatTool(this.client, input);
            return {
              content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
          }

          case "minimax_generate_video": {
            const input = videoInputSchema.parse(args);
            const result = await executeVideoTool(this.client, input);
            return {
              content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
          }

          case "minimax_check_task": {
            const input = checkTaskInputSchema.parse(args);
            const result = await executeCheckTaskTool(this.client, input);
            return {
              content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
          }

          case "minimax_text_to_speech": {
            const input = speechInputSchema.parse(args);
            const result = await executeSpeechTool(this.client, input);

            // For speech, return audio info but truncate base64 for readability
            const displayResult = { ...result };
            if (displayResult.audio_base64 && displayResult.audio_base64.length > 100) {
              displayResult.audio_base64 =
                displayResult.audio_base64.substring(0, 100) +
                `... (${result.audio_base64?.length} chars total)`;
            }
            return {
              content: [{ type: "text", text: JSON.stringify(displayResult, null, 2) }],
            };
          }

          case "minimax_generate_music": {
            const input = musicInputSchema.parse(args);
            const result = await executeMusicTool(this.client, input);
            return {
              content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
          }

          case "minimax_download_file": {
            const input = fileDownloadInputSchema.parse(args);
            const result = await executeFileDownloadTool(this.client, input);

            // Truncate base64 content for display
            const displayResult = { ...result };
            if (displayResult.base64_content && displayResult.base64_content.length > 100) {
              displayResult.base64_content =
                displayResult.base64_content.substring(0, 100) +
                `... (${result.base64_content?.length} chars total)`;
            }
            return {
              content: [{ type: "text", text: JSON.stringify(displayResult, null, 2) }],
            };
          }

          default:
            throw new McpError(
              ErrorCode.MethodNotFound,
              `Unknown tool: ${name}`
            );
        }
      } catch (error) {
        // Handle MiniMax API errors
        if (error instanceof MinimaxApiError) {
          return {
            content: [
              {
                type: "text",
                text: JSON.stringify(
                  {
                    error: true,
                    message: error.message,
                    status_code: error.statusCode,
                    status_msg: error.statusMsg,
                  },
                  null,
                  2
                ),
              },
            ],
            isError: true,
          };
        }

        // Handle Zod validation errors
        if (error instanceof Error && error.name === "ZodError") {
          return {
            content: [
              {
                type: "text",
                text: JSON.stringify(
                  {
                    error: true,
                    message: "Invalid input parameters",
                    details: error.message,
                  },
                  null,
                  2
                ),
              },
            ],
            isError: true,
          };
        }

        // Re-throw MCP errors
        if (error instanceof McpError) {
          throw error;
        }

        // Handle unexpected errors
        const errorMessage = error instanceof Error ? error.message : String(error);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  error: true,
                  message: errorMessage,
                },
                null,
                2
              ),
            },
          ],
          isError: true,
        };
      }
    });
  }

  /**
   * Set up error handling
   */
  private setupErrorHandling(): void {
    this.server.onerror = (error) => {
      console.error("[MiniMax MCP Error]", error);
    };

    process.on("SIGINT", async () => {
      await this.server.close();
      process.exit(0);
    });
  }

  /**
   * Run the server
   */
  async run(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error(`MiniMax MCP Server v${SERVER_VERSION} running on stdio`);
  }
}

// Main entry point
const server = new MinimaxMcpServer();
server.run().catch((error) => {
  console.error("Failed to start MiniMax MCP server:", error);
  process.exit(1);
});

