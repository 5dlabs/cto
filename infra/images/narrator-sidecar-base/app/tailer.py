"""Async tail -f on the ACP NDJSON stream file."""

import asyncio
import os
from collections.abc import AsyncIterator

from .config import settings


async def tail_acp_stream(path: str | None = None, poll_interval: float = 0.1) -> AsyncIterator[str]:
    """
    Yield lines appended to `path` as they arrive, similar to `tail -f`.
    Waits for the file to exist if it doesn't yet.
    """
    stream_path = path or settings.raw_stream_path
    # Wait for file to appear
    while not os.path.exists(stream_path):
        await asyncio.sleep(1.0)

    with open(stream_path, "r") as fh:
        # Seek to end on first open so we only see new lines
        fh.seek(0, 2)
        while True:
            line = fh.readline()
            if line:
                yield line
            else:
                await asyncio.sleep(poll_interval)
