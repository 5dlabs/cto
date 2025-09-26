# Output Format

The Cursor Agent CLI provides multiple output formats with the `--output-format` option when combined with `--print`. These formats include structured formats for programmatic use (`json`, `stream-json`) and a simplified text format for human-readable progress tracking.

The default `--output-format` is `stream-json`. This option is only valid when
printing (`--print`) or when print mode is inferred (non-TTY stdout or piped
stdin).

## JSON format

The `json` output format emits a single JSON object (followed by a newline) when the run completes successfully. Deltas and tool events are not emitted; text is aggregated into the final result.

On failure, the process exits with a non-zero code and writes an error message to stderr. No well-formed JSON object is emitted in failure cases.

### Success response

When successful, the CLI outputs a JSON object with the following structure:

```
{
  "type": "result",
  "subtype": "success",
  "is_error": false,
  "duration_ms": 1234,
  "duration_api_ms": 1234,
  "result": "<full assistant text>",
  "session_id": "<uuid>",
  "request_id": "<optional request id>"
}
```

FieldDescription`type`Always `"result"` for terminal results`subtype`Always `"success"` for successful completions`is_error`Always `false` for successful responses`duration_ms`Total execution time in milliseconds`duration_api_ms`API request time in milliseconds (currently equal to `duration_ms`)`result`Complete assistant response text (concatenation of all text deltas)`session_id`Unique session identifier`request_id`Optional request identifier (may be omitted)
## Stream JSON format

The `stream-json` output format emits newline-delimited JSON (NDJSON). Each line contains a single JSON object representing a real-time event during execution.

The stream ends with a terminal `result` event on success. On failure, the process exits with a non-zero code and the stream may end early without a terminal event; an error message is written to stderr.

### Event types

#### System initialization

Emitted once at the beginning of each session:

```
{
  "type": "system",
  "subtype": "init",
  "apiKeySource": "env|flag|login",
  "cwd": "/absolute/path",
  "session_id": "<uuid>",
  "model": "<model display name>",
  "permissionMode": "default"
}
```

Future fields like `tools` and `mcp_servers` may be added to this event.

#### User message

Contains the user's input prompt:

```
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [{ "type": "text", "text": "<prompt>" }]
  },
  "session_id": "<uuid>"
}
```

#### Assistant text delta

Emitted multiple times as the assistant generates its response. These events contain incremental text chunks:

```
{
  "type": "assistant",
  "message": {
    "role": "assistant",
    "content": [{ "type": "text", "text": "<delta chunk>" }]
  },
  "session_id": "<uuid>"
}
```

Concatenate all `message.content[].text` values in order to reconstruct the
complete assistant response.

#### Tool call events

Tool calls are tracked with start and completion events:

**Tool call started:**

```
{
  "type": "tool_call",
  "subtype": "started",
  "call_id": "<string id>",
  "tool_call": {
    "readToolCall": {
      "args": { "path": "file.txt" }
    }
  },
  "session_id": "<uuid>"
}
```

**Tool call completed:**

```
{
  "type": "tool_call",
  "subtype": "completed",
  "call_id": "<string id>",
  "tool_call": {
    "readToolCall": {
      "args": { "path": "file.txt" },
      "result": {
        "success": {
          "content": "file contents...",
          "isEmpty": false,
          "exceededLimit": false,
          "totalLines": 54,
          "totalChars": 1254
        }
      }
    }
  },
  "session_id": "<uuid>"
}
```

#### Tool call types

**Read file tool:**

- **Started**: `tool_call.readToolCall.args` contains `{ "path": "file.txt" }`
- **Completed**: `tool_call.readToolCall.result.success` contains file metadata and content

**Write file tool:**

- **Started**: `tool_call.writeToolCall.args` contains `{ "path": "file.txt", "fileText": "content...", "toolCallId": "id" }`
- **Completed**: `tool_call.writeToolCall.result.success` contains `{ "path": "/absolute/path", "linesCreated": 19, "fileSize": 942 }`

**Other tools:**

- May use `tool_call.function` structure with `{ "name": "tool_name", "arguments": "..." }`

#### Terminal result

The final event emitted on successful completion:

```
{
  "type": "result",
  "subtype": "success",
  "duration_ms": 1234,
  "duration_api_ms": 1234,
  "is_error": false,
  "result": "<full assistant text>",
  "session_id": "<uuid>",
  "request_id": "<optional request id>"
}
```

### Example sequence

Here's a representative NDJSON sequence showing the typical flow of events:

```
{"type":"system","subtype":"init","apiKeySource":"login","cwd":"/Users/user/project","session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff","model":"Claude 4 Sonnet","permissionMode":"default"}
{"type":"user","message":{"role":"user","content":[{"type":"text","text":"Read README.md and create a summary"}]},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"I'll "}]},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"read the README.md file"}]},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"tool_call","subtype":"started","call_id":"toolu_vrtx_01NnjaR886UcE8whekg2MGJd","tool_call":{"readToolCall":{"args":{"path":"README.md"}}},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"tool_call","subtype":"completed","call_id":"toolu_vrtx_01NnjaR886UcE8whekg2MGJd","tool_call":{"readToolCall":{"args":{"path":"README.md"},"result":{"success":{"content":"# Project\n\nThis is a sample project...","isEmpty":false,"exceededLimit":false,"totalLines":54,"totalChars":1254}}}},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":" and create a summary"}]},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"tool_call","subtype":"started","call_id":"toolu_vrtx_01Q3VHVnWFSKygaRPT7WDxrv","tool_call":{"writeToolCall":{"args":{"path":"summary.txt","fileText":"# README Summary\n\nThis project contains...","toolCallId":"toolu_vrtx_01Q3VHVnWFSKygaRPT7WDxrv"}}},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"tool_call","subtype":"completed","call_id":"toolu_vrtx_01Q3VHVnWFSKygaRPT7WDxrv","tool_call":{"writeToolCall":{"args":{"path":"summary.txt","fileText":"# README Summary\n\nThis project contains...","toolCallId":"toolu_vrtx_01Q3VHVnWFSKygaRPT7WDxrv"},"result":{"success":{"path":"/Users/user/project/summary.txt","linesCreated":19,"fileSize":942}}}},"session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff"}
{"type":"result","subtype":"success","duration_ms":5234,"duration_api_ms":5234,"is_error":false,"result":"I'll read the README.md file and create a summary","session_id":"c6b62c6f-7ead-4fd6-9922-e952131177ff","request_id":"10e11780-df2f-45dc-a1ff-4540af32e9c0"}
```

## Text format

The `text` output format provides a simplified, human-readable stream of agent actions. Instead of detailed JSON events, it outputs concise text descriptions of what the agent is doing in real-time.

This format is useful for monitoring agent progress without the overhead of parsing structured data, making it ideal for logging, debugging, or simple progress tracking.

### Example output

```
Read file
Edited file
Ran terminal command
Created new file
```

Each action appears on a new line as the agent performs it, providing immediate feedback on the agent's progress through the task.

## Implementation notes

- Each event is emitted as a single line terminated by `\n`
- `thinking` events are suppressed in print mode and will not appear in either output format
- Field additions may occur over time in a backward-compatible way (consumers should ignore unknown fields)
- The stream format provides real-time updates, while the JSON format waits for completion before outputting results
- Concatenate all `assistant` message deltas to reconstruct the complete response
- Tool call IDs can be used to correlate start/completion events
- Session IDs remain consistent throughout a single agent execution