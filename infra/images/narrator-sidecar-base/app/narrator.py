"""LLM narrator: converts a rolling window of ACP events into narration phrases."""

import json
from openai import AsyncOpenAI
from .acp_parser import ACPEvent, event_to_narrator_text
from .config import settings

_SYSTEM_PROMPT = """\
You are the narrator for an AI agent's live session, speaking in the voice of Rex — \
a calm, dry, slightly sardonic British AI observer. You watch the agent's internal \
activity stream and produce brief audio narration phrases (1-2 short sentences max) \
that a viewer would hear while watching a live avatar.

Rules:
- Narrate WHAT the agent is doing, not HOW. ("Searching for relevant files" not "Calling the grep tool")
- Use present continuous tense. ("Reviewing the diff…", "Thinking through the architecture…")
- Match urgency to pace: rapid tool calls = "med", stuck in thinking = "low", errors = "high"
- If nothing interesting happened, return {"silent": true}
- Return ONLY valid JSON — no markdown fences, no prose outside the JSON.

Response schema (pick one):
  {"phrase": "<narration text>", "urgency": "low|med|high"}
  {"silent": true}
"""


class Narrator:
    def __init__(self):
        self._client = AsyncOpenAI(api_key=settings.openai_api_key) if settings.openai_api_key else None
        self._last_phrase: str | None = None
        self._last_phrase_time: float = 0

    async def narrate(self, events: list[ACPEvent]) -> dict:
        """
        Given a rolling window of recent ACP events, return a narration dict.
        Returns {"phrase": "...", "urgency": "low|med|high"} or {"silent": true}.
        """
        if not events or not self._client:
            return {"silent": True}

        # Filter out empty events
        lines = [event_to_narrator_text(e) for e in events if event_to_narrator_text(e)]
        if not lines:
            return {"silent": True}

        user_content = "Recent agent activity:\n" + "\n".join(lines[-20:])  # last 20 events

        try:
            response = await self._client.chat.completions.create(
                model=settings.llm_model,
                messages=[
                    {"role": "system", "content": _SYSTEM_PROMPT},
                    {"role": "user", "content": user_content},
                ],
                temperature=0.7,
                max_tokens=120,
                response_format={"type": "json_object"},
            )
            raw = response.choices[0].message.content or "{}"
            result = json.loads(raw)
        except Exception:
            return {"silent": True}

        # Validate shape
        if "silent" in result:
            return {"silent": True}
        phrase = result.get("phrase", "").strip()
        urgency = result.get("urgency", "low")
        if urgency not in ("low", "med", "high"):
            urgency = "low"
        if not phrase:
            return {"silent": True}
        # Deduplicate: don't repeat the same phrase
        if phrase == self._last_phrase:
            return {"silent": True}
        self._last_phrase = phrase
        return {"phrase": phrase, "urgency": urgency}
