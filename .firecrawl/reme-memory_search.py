"""Memory search tool for semantic search in memory files."""

import json

from ....core.enumeration import MemorySource
from ....core.op import BaseTool
from ....core.runtime\_context import RuntimeContext
from ....core.schema import ToolCall
from ....core.utils import get\_logger

logger = get\_logger()

class MemorySearch(BaseTool):
 """Semantically search MEMORY.md and memory files."""

 def \_\_init\_\_(
 self,
 sources: list\[MemorySource\] \| None = None,
 min\_score: float = 0.1,
 max\_results: int = 5,
 vector\_weight: float = 0.7,
 candidate\_multiplier: float = 3.0,
 \*\*kwargs,
 ):
 """Initialize memory search tool."""
 assert 0.0 <= vector\_weight <= 1.0, f"vector\_weight must be between 0 and 1, got {vector\_weight}"
 kwargs.setdefault("max\_retries", 1)
 kwargs.setdefault("raise\_exception", False)
 super().\_\_init\_\_(\*\*kwargs)
 self.sources = sources or \[MemorySource.MEMORY\]
 self.min\_score = min\_score
 self.max\_results = max\_results
 self.vector\_weight = vector\_weight
 self.candidate\_multiplier = candidate\_multiplier

 def \_build\_tool\_call(self) -> ToolCall:
 return ToolCall(
 \*\*{
 "description": (
 "Mandatory recall step: semantically search MEMORY.md + memory/\*.md "
 "(and optional session transcripts) before answering questions about "
 "prior work, decisions, dates, people, preferences, or todos; returns "
 "top snippets with path + lines."
 ),
 "parameters": {
 "type": "object",
 "properties": {
 "query": {
 "type": "string",
 "description": "The semantic search query to find relevant memory snippets",
 },
 "max\_results": {
 "type": "integer",
 "description": "Maximum number of search results to return (optional), default 5",
 },
 "min\_score": {
 "type": "number",
 "description": "Minimum similarity score threshold for results (optional), default 0.1",
 },
 },
 "required": \["query"\],
 },
 },
 )

 async def execute(self) -> str:
 """Execute the memory search operation."""
 query: str = self.context.query.strip()
 min\_score: float = self.context.get("min\_score", self.min\_score)
 max\_results: int = self.context.get("max\_results", self.max\_results)

 assert query, "Query cannot be empty"
 assert (
 isinstance(min\_score, float) and 0.0 <= min\_score <= 1.0
 ), f"min\_score must be between 0 and 1, got {min\_score}"
 assert (
 isinstance(max\_results, int) and max\_results > 0
 ), f"max\_results must be a positive integer, got {max\_results}"

 # Use hybrid\_search from file\_store
 results = await self.file\_store.hybrid\_search(
 query=query,
 limit=max\_results,
 sources=self.sources,
 vector\_weight=self.vector\_weight,
 candidate\_multiplier=self.candidate\_multiplier,
 )

 # Filter by min\_score
 results = \[r for r in results if r.score >= min\_score\]

 return json.dumps(\[result.model\_dump(exclude\_none=True) for result in results\], indent=2, ensure\_ascii=False)

 async def call(self, context: RuntimeContext = None, \*\*kwargs):
 """Execute the tool with unified error handling.

 This method catches all exceptions and returns error messages
 to the LLM instead of raising them.
 """
 self.context = RuntimeContext.from\_context(context, \*\*kwargs)

 try:
 await self.before\_execute()
 response = await self.execute()
 response = await self.after\_execute(response)
 return response

 except Exception as e:
 # Return error message to LLM instead of raising
 error\_msg = f"{self.\_\_class\_\_.\_\_name\_\_} failed: {str(e)}"
 logger.exception(error\_msg)
 return await self.after\_execute(error\_msg)