#!/usr/bin/env python3
"""Real-model LLM adapter for CTO intake's OpenClaw-compatible llm-task argv.

Supports Anthropic Messages API and Google Gemini generateContent via stdlib urllib.
Secrets are read from env only and never printed.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


def load_payload(ns: argparse.Namespace) -> dict[str, Any]:
    raw = "{}"
    if ns.args_file:
        raw = Path(ns.args_file).read_text()
    elif ns.args_json:
        raw = ns.args_json
    try:
        return json.loads(raw or "{}")
    except Exception as exc:
        print(f"real-llm-invoke: invalid args JSON: {exc}", file=sys.stderr)
        sys.exit(2)


def schema_text(path: str) -> str:
    if not path:
        return ""
    try:
        return Path(path).read_text()[:30000]
    except Exception:
        return ""


def compact(obj: Any, limit: int = 120000) -> str:
    try:
        text = json.dumps(obj, ensure_ascii=False, indent=2)
    except Exception:
        text = str(obj)
    if len(text) > limit:
        return text[:limit] + "\n...[truncated]"
    return text


def build_messages(payload: dict[str, Any], action: str) -> tuple[str, str]:
    prompt = str(payload.get("prompt") or "")
    inp = payload.get("input", {})
    schema = str(payload.get("schema") or "")
    schema_body = schema_text(schema)
    mode = "JSON" if action == "json" else "plain text"
    system = (
        "You are a CTO intake workflow LLM. Return only the requested output, no markdown fences. "
        "If JSON is requested, output strictly valid JSON matching the supplied schema as closely as possible. "
        "Do not include explanations outside JSON."
    )
    user = f"""Task prompt:
{prompt}

Input JSON:
{compact(inp)}

Requested output mode: {mode}
Schema path: {schema}
Schema JSON:
{schema_body}
"""
    return system, user


def extract_json(text: str) -> Any:
    text = text.strip()
    if text.startswith("```"):
        text = re.sub(r"^```(?:json)?\s*", "", text)
        text = re.sub(r"\s*```$", "", text)
    try:
        return json.loads(text)
    except Exception:
        pass
    # Best-effort extract first top-level object/array.
    starts = [i for i in (text.find("{"), text.find("[")) if i >= 0]
    if not starts:
        raise ValueError("model returned no JSON object/array")
    start = min(starts)
    opener = text[start]
    closer = "}" if opener == "{" else "]"
    depth = 0
    in_str = False
    esc = False
    for i, ch in enumerate(text[start:], start):
        if in_str:
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
        else:
            if ch == '"':
                in_str = True
            elif ch == opener:
                depth += 1
            elif ch == closer:
                depth -= 1
                if depth == 0:
                    return json.loads(text[start:i+1])
    raise ValueError("could not parse JSON from model output")


def http_json(url: str, headers: dict[str, str], body: dict[str, Any], timeout: int = 180) -> dict[str, Any]:
    data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, headers=headers, method="POST")
    last: Exception | None = None
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                return json.loads(resp.read().decode())
        except urllib.error.HTTPError as exc:
            msg = exc.read().decode(errors="replace")[:1000]
            last = RuntimeError(f"HTTP {exc.code}: {msg}")
            if exc.code not in (429, 500, 502, 503, 504):
                break
        except Exception as exc:
            last = exc
        time.sleep(2 ** attempt)
    raise last or RuntimeError("request failed")


def call_anthropic(model: str, system: str, user: str, action: str) -> str:
    key = os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        raise RuntimeError("ANTHROPIC_API_KEY missing")
    # Map future/alias names to available public names only if needed.
    aliases = {
        "claude-opus-4-7-20260610": "claude-opus-4-1-20250805",
        "claude-opus-4-7": "claude-opus-4-1-20250805",
        "claude-sonnet-4-6-20260514": "claude-sonnet-4-5-20250929",
        "claude-sonnet-4-6": "claude-sonnet-4-5-20250929",
        "claude-haiku-4-5-20251001": "claude-haiku-4-5-20251001",
    }
    wire_model = aliases.get(model, model or "claude-sonnet-4-5-20250929")
    body = {
        "model": wire_model,
        "max_tokens": int(os.environ.get("REAL_LLM_MAX_TOKENS", "6000")),
        "temperature": float(os.environ.get("REAL_LLM_TEMPERATURE", "0.2")),
        "system": system,
        "messages": [{"role": "user", "content": user}],
    }
    if action == "json":
        body["messages"][0]["content"] += "\n\nReturn valid JSON only."
    resp = http_json(
        "https://api.anthropic.com/v1/messages",
        {
            "content-type": "application/json",
            "x-api-key": key,
            "anthropic-version": "2023-06-01",
        },
        body,
    )
    parts = resp.get("content", [])
    return "".join(part.get("text", "") for part in parts if isinstance(part, dict))


def call_gemini(model: str, system: str, user: str, action: str) -> str:
    key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
    if not key:
        raise RuntimeError("GEMINI_API_KEY/GOOGLE_API_KEY missing")
    wire_model = model or os.environ.get("GEMINI_MODEL", "gemini-3.1-pro-preview")
    body = {
        "systemInstruction": {"parts": [{"text": system}]},
        "contents": [{"role": "user", "parts": [{"text": user + ("\n\nReturn valid JSON only." if action == "json" else "")}]}],
        "generationConfig": {"temperature": 0.2, "maxOutputTokens": int(os.environ.get("REAL_LLM_MAX_TOKENS", "6000"))},
    }
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{wire_model}:generateContent?key={key}"
    resp = http_json(url, {"content-type": "application/json"}, body)
    candidates = resp.get("candidates", [])
    if not candidates:
        return ""
    parts = candidates[0].get("content", {}).get("parts", [])
    return "".join(part.get("text", "") for part in parts if isinstance(part, dict))


def choose_provider(payload: dict[str, Any]) -> tuple[str, str]:
    provider = str(payload.get("provider") or os.environ.get("REAL_LLM_PROVIDER") or "").lower()
    model = str(payload.get("model") or os.environ.get("REAL_LLM_MODEL") or "")
    # GitHub Copilot provider is not directly API-addressable here; use Anthropic key as real model backend.
    if provider in ("", "github-copilot", "copilot"):
        provider = os.environ.get("REAL_LLM_PROVIDER", "anthropic").lower()
        model = os.environ.get("REAL_LLM_MODEL", model)
    if provider in ("google", "gemini", "google-gemini"):
        return "gemini", model
    return "anthropic", model


def main() -> int:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--tool")
    parser.add_argument("--action", default="json")
    parser.add_argument("--args-json")
    parser.add_argument("--args-file")
    ns, _ = parser.parse_known_args()
    payload = load_payload(ns)
    action = ns.action or "json"
    system, user = build_messages(payload, action)
    provider, model = choose_provider(payload)
    if provider == "gemini":
        text = call_gemini(model, system, user, action)
    else:
        text = call_anthropic(model, system, user, action)
    if action == "json":
        try:
            obj = extract_json(text)
        except Exception as exc:
            print(f"real-llm-invoke: model did not return valid JSON: {exc}", file=sys.stderr)
            print(text[:2000], file=sys.stderr)
            return 3
        print(json.dumps(obj, ensure_ascii=False))
    else:
        print(text.strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
