# MiniMax MCP Server

MCP (Model Context Protocol) server for [MiniMax AI](https://platform.minimax.io/) - providing access to their complete AI suite:

- **MiniMax-M2**: Text generation with 200k context window
- **Hailuo 2.3**: Text-to-video and image-to-video generation
- **Speech-2.6**: Text-to-speech with 40 languages and emotional tones
- **Music-2.0**: Text-to-music generation with vocals

## Installation

```bash
npm install @5dlabs/minimax-mcp
```

Or run directly with npx:

```bash
npx @5dlabs/minimax-mcp
```

## Configuration

Set the following environment variables:

```bash
export MINIMAX_API_KEY="your-api-key"
export MINIMAX_GROUP_ID="your-group-id"  # Optional, found in account settings
```

## Usage with Claude Desktop

Add to your Claude Desktop configuration (`~/.claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "minimax": {
      "command": "npx",
      "args": ["-y", "@5dlabs/minimax-mcp"],
      "env": {
        "MINIMAX_API_KEY": "your-api-key"
      }
    }
  }
}
```

## Available Tools

### minimax_chat

Generate text using MiniMax-M2 model with 200k context and 128k output capacity.

```typescript
{
  messages: [{ role: "user", content: "Hello!" }],
  model: "MiniMax-M2",  // optional
  temperature: 0.7,     // optional, 0.0-1.0
  max_tokens: 4096      // optional
}
```

### minimax_generate_video

Generate video from text or image using Hailuo 2.3.

```typescript
{
  prompt: "A serene mountain landscape with flowing rivers",
  model: "MiniMax-Hailuo-2.3",  // or "MiniMax-Hailuo-2.3Fast"
  duration: 6,                  // 6 or 10 seconds
  resolution: "1080p"           // "1080p", "768p", or "512p"
}
```

Returns a `task_id` - use `minimax_check_task` to poll for completion.

### minimax_text_to_speech

Convert text to natural speech with emotional expression.

```typescript
{
  text: "Hello, welcome to our service!",
  model: "speech-2.6-hd",  // or "speech-2.6-turbo"
  language: "en",          // 40 languages supported
  emotion: "happy",        // neutral, happy, sad, angry, fear, disgust, surprise
  speed: 1.0               // 0.5-2.0
}
```

### minimax_generate_music

Generate music from text descriptions.

```typescript
{
  prompt: "An upbeat electronic track with synth melodies",
  duration: 30,           // duration in seconds
  include_vocals: false   // whether to include AI vocals
}
```

Returns a `task_id` - use `minimax_check_task` to poll for completion.

### minimax_check_task

Poll the status of async tasks (video/music generation).

```typescript
{
  task_id: "abc123"
}
```

### minimax_download_file

Download a generated file by its file_id.

```typescript
{
  file_id: "xyz789",
  output_path: "/path/to/save/file"  // optional
}
```

## API Documentation

- [MiniMax Platform Docs](https://platform.minimax.io/docs)
- [M2 Text Generation](https://platform.minimax.io/docs/api-reference/text-openai-api)
- [Hailuo Video](https://platform.minimax.io/docs/api-reference/video-generation-intro)
- [Speech 2.6](https://platform.minimax.io/docs/api-reference/speech-t2a-http)
- [Music 2.0](https://platform.minimax.io/docs/api-reference/music-generation)

## License

MIT

