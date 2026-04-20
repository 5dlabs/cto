"""Write interrupt events to the narrator interrupt FIFO file."""

import asyncio
import json
import os
from datetime import datetime, timezone

import aiofiles

from .config import settings


async def write_interrupt(session_id: str, payload: dict) -> None:
    """Append an interrupt JSON record to interrupt.jsonl."""
    os.makedirs(os.path.dirname(settings.interrupt_path), exist_ok=True)
    record = {
        "session_id": session_id,
        "ts": datetime.now(timezone.utc).isoformat(),
        **payload,
    }
    line = json.dumps(record) + "\n"
    async with aiofiles.open(settings.interrupt_path, "a") as fh:
        await fh.write(line)
