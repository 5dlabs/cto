"""Parse ACP NDJSON lines into typed event structs."""

import json
from typing import Any, Literal, Union
from pydantic import BaseModel, Field


class ToolCallEvent(BaseModel):
    type: Literal["tool_call"] = "tool_call"
    tool_name: str
    tool_input: dict[str, Any] = Field(default_factory=dict)
    call_id: str = ""
    ts: str = ""


class ToolResultEvent(BaseModel):
    type: Literal["tool_result"] = "tool_result"
    tool_name: str = ""
    call_id: str = ""
    content: Any = None
    is_error: bool = False
    ts: str = ""


class ThinkingEvent(BaseModel):
    type: Literal["thinking"] = "thinking"
    content: str = ""
    ts: str = ""


class UserMessageEvent(BaseModel):
    type: Literal["user_message"] = "user_message"
    content: str = ""
    ts: str = ""


class AssistantMessageEvent(BaseModel):
    type: Literal["assistant_message"] = "assistant_message"
    content: str = ""
    ts: str = ""


class SystemEvent(BaseModel):
    type: Literal["system"] = "system"
    subtype: str = ""
    data: dict[str, Any] = Field(default_factory=dict)
    ts: str = ""


class UnknownEvent(BaseModel):
    type: Literal["unknown"] = "unknown"
    raw: dict[str, Any] = Field(default_factory=dict)


ACPEvent = Union[
    ToolCallEvent,
    ToolResultEvent,
    ThinkingEvent,
    UserMessageEvent,
    AssistantMessageEvent,
    SystemEvent,
    UnknownEvent,
]

# Map ACP type strings → model constructors
_TYPE_MAP = {
    "tool_call": ToolCallEvent,
    "tool_result": ToolResultEvent,
    "thinking": ThinkingEvent,
    "user_message": UserMessageEvent,
    "assistant_message": AssistantMessageEvent,
    "system": SystemEvent,
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

    event_type = data.get("type", "unknown")
    cls = _TYPE_MAP.get(event_type)
    if cls is None:
        return UnknownEvent(raw=data)
    try:
        return cls(**data)
    except Exception:
        return UnknownEvent(raw=data)


def event_to_narrator_text(event: ACPEvent) -> str:
    """Summarize an event as a short human-readable string for the LLM prompt."""
    match event:
        case ToolCallEvent():
            return f"[tool_call] {event.tool_name}({json.dumps(event.tool_input)[:120]})"
        case ToolResultEvent():
            content_str = str(event.content)[:120] if event.content else ""
            suffix = " (error)" if event.is_error else ""
            return f"[tool_result] {event.tool_name}{suffix}: {content_str}"
        case ThinkingEvent():
            return f"[thinking] {event.content[:200]}"
        case UserMessageEvent():
            return f"[user] {event.content[:200]}"
        case AssistantMessageEvent():
            return f"[assistant] {event.content[:200]}"
        case SystemEvent():
            return f"[system:{event.subtype}]"
        case _:
            return f"[event:{event.type}]"
