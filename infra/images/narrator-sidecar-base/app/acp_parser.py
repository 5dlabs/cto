"""Parse OpenClaw raw stream NDJSON lines into typed event structs.

OpenClaw raw stream events:
- assistant_text_stream: streaming text from assistant
- assistant_thinking_stream: reasoning/thinking output
- assistant_message_end: end of assistant message
- tool_call: tool invocation
- tool_result: tool result
- user_message: user input
"""

import json
from typing import Any, Literal, Union
from pydantic import BaseModel, Field


class AssistantTextStreamEvent(BaseModel):
    type: Literal["assistant_text_stream"] = "assistant_text_stream"
    content: str = ""
    ts: int = 0


class AssistantThinkingStreamEvent(BaseModel):
    type: Literal["assistant_thinking_stream"] = "assistant_thinking_stream"
    content: str = ""
    ts: int = 0


class AssistantMessageEndEvent(BaseModel):
    type: Literal["assistant_message_end"] = "assistant_message_end"
    rawText: str = ""
    rawThinking: str = ""
    ts: int = 0


class ToolCallEvent(BaseModel):
    type: Literal["tool_call"] = "tool_call"
    tool_name: str = ""
    tool_input: dict[str, Any] = Field(default_factory=dict)
    call_id: str = ""
    ts: int = 0


class ToolResultEvent(BaseModel):
    type: Literal["tool_result"] = "tool_result"
    tool_name: str = ""
    call_id: str = ""
    content: Any = None
    is_error: bool = False
    ts: int = 0


class UserMessageEvent(BaseModel):
    type: Literal["user_message"] = "user_message"
    content: str = ""
    ts: int = 0


class UnknownEvent(BaseModel):
    type: Literal["unknown"] = "unknown"
    raw: dict[str, Any] = Field(default_factory=dict)


ACPEvent = Union[
    AssistantTextStreamEvent,
    AssistantThinkingStreamEvent,
    AssistantMessageEndEvent,
    ToolCallEvent,
    ToolResultEvent,
    UserMessageEvent,
    UnknownEvent,
]

# Map event strings → model constructors
_TYPE_MAP: dict[str, type] = {
    "assistant_text_stream": AssistantTextStreamEvent,
    "assistant_thinking_stream": AssistantThinkingStreamEvent,
    "assistant_message_end": AssistantMessageEndEvent,
    "tool_call": ToolCallEvent,
    "tool_result": ToolResultEvent,
    "user_message": UserMessageEvent,
}


def parse_line(line: str) -> ACPEvent | None:
    """Parse a single NDJSON line. Returns None on empty/malformed input."""
    line = line.strip()
    if not line:
        return None
    try:
        data = json.loads(line)
    except json.JSONDecodeError:
        return None

    event_type = data.get("event", data.get("type", "unknown"))
    cls = _TYPE_MAP.get(event_type)
    if cls is None:
        return UnknownEvent(raw=data)
    try:
        # Normalize field names
        normalized = dict(data)
        normalized["type"] = event_type
        if "ts" in normalized and isinstance(normalized["ts"], int):
            pass  # already int
        return cls(**normalized)
    except Exception:
        return UnknownEvent(raw=data)


def event_to_narrator_text(event: ACPEvent) -> str:
    """Summarize an event as a short human-readable string for the LLM prompt."""
    match event:
        case AssistantTextStreamEvent():
            content = event.content.strip()[:200]
            return f"[text] {content}" if content else ""
        case AssistantThinkingStreamEvent():
            content = event.content.strip()[:200]
            return f"[thinking] {content}" if content else ""
        case AssistantMessageEndEvent():
            return "[message_end]"
        case ToolCallEvent():
            return f"[tool_call] {event.tool_name}"
        case ToolResultEvent():
            suffix = " (error)" if event.is_error else ""
            return f"[tool_result] {event.tool_name}{suffix}"
        case UserMessageEvent():
            return f"[user] {event.content[:200]}"
        case _:
            return f"[event:{event.type}]"
