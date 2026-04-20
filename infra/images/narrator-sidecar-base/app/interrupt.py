"""Write interrupt events to the narrator interrupt FIFO file."""

import asyncio
import json
import os
from datetime import datetime, timezone

import aiofiles

from .config import settings


async def write_interrupt(session_id: str, text: str, source: str = "text") -> None:
    """Append an interrupt JSON record to interrupt.jsonl."""
    interrupt_dir = os.path.dirname(settings.interrupt_path)
    if interrupt_dir:
        os.makedirs(interrupt_dir, exist_ok=True)
    record = {
        "session_id": session_id,
        "text": text,
        "source": source,
        "ts": datetime.now(timezone.utc).isoformat(),
    }
    line = json.dumps(record) + "\n"
    async with aiofiles.open(settings.interrupt_path, "a") as fh:
        await fh.write(line)
