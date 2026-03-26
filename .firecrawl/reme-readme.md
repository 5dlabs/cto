![ReMe Logo](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/docs/_static/figure/reme_logo.png)

[![Python Version](https://img.shields.io/badge/python-3.10+-blue)](https://pypi.org/project/reme-ai/)[![PyPI Version](https://img.shields.io/pypi/v/reme-ai.svg?logo=pypi)](https://pypi.org/project/reme-ai/)[![PyPI Downloads](https://img.shields.io/pypi/dm/reme-ai)](https://pepy.tech/project/reme-ai/)[![GitHub commit activity](https://img.shields.io/github/commit-activity/m/agentscope-ai/ReMe?style=flat-square)](https://github.com/agentscope-ai/ReMe)

[![License](https://img.shields.io/badge/license-Apache--2.0-black)](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/LICENSE)[![English](https://img.shields.io/badge/English-Click-yellow)](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/README.md)[![简体中文](https://img.shields.io/badge/%E7%AE%80%E4%BD%93%E4%B8%AD%E6%96%87-%E7%82%B9%E5%87%BB%E6%9F%A5%E7%9C%8B-orange)](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/README_ZH.md)[![GitHub Stars](https://img.shields.io/github/stars/agentscope-ai/ReMe?style=social)](https://github.com/agentscope-ai/ReMe)[![DeepWiki](https://img.shields.io/badge/DeepWiki-Ask_Devin-navy.svg)](https://deepwiki.com/agentscope-ai/ReMe)

**A memory management toolkit for AI agents — Remember Me, Refine Me.**

\> For the older version, please refer to the \[0.2.x documentation\](docs/README\_0\_2\_x.md).

\-\-\-

🧠 ReMe is a memory management framework designed for \*\*AI agents\*\*, providing
both \[file-based\](#-file-based-memory-system-remelight) and \[vector-based\](#-vector-based-memory-system) memory systems.

It tackles two core problems of agent memory: \*\*limited context window\*\* (early information is truncated or lost in long
conversations) and \*\*stateless sessions\*\* (new sessions cannot inherit history and always start from scratch).

ReMe gives agents \*\*real memory\*\* — old conversations are automatically compacted, important information is persistently
stored, and relevant context is automatically recalled in future interactions.

ReMe achieves state-of-the-art results on the LoCoMo and HaluMem benchmarks; see the \[Experimental results\](#experimental-results).

**What you can do with ReMe**

\- \*\*Personal assistant\*\*: Provide long-term memory for agents like \[CoPaw\](https://github.com/agentscope-ai/CoPaw),
 remembering user preferences and conversation history.
\- \*\*Coding assistant\*\*: Record code style preferences and project context, maintaining a consistent development
 experience across sessions.
\- \*\*Customer service bot\*\*: Track user issue history and preference settings for personalized service.
\- \*\*Task automation\*\*: Learn success/failure patterns from historical tasks to continuously optimize execution
 strategies.
\- \*\*Knowledge Q&A\*\*: Build a searchable knowledge base with semantic search and exact matching support.
\- \*\*Multi-turn dialogue\*\*: Automatically compress long conversations while retaining key information within limited
 context windows.

\-\-\-

\## 📁 File-based memory system (ReMeLight)

\> Memory as files, files as memory.

Treat \*\*memory as files\*\* — readable, editable, and copyable.
\[CoPaw\](https://github.com/agentscope-ai/CoPaw) integrates long-term memory and context management by inheriting from
\`ReMeLight\`.

\| Traditional memory system \| File-based ReMe \|
\|---------------------------\|----------------------\|
\| 🗄️ Database storage \| 📝 Markdown files \|
\| 🔒 Opaque \| 👀 Always readable \|
\| ❌ Hard to modify \| ✏️ Directly editable \|
\| 🚫 Hard to migrate \| 📦 Copy to migrate \|

\`\`\`
working\_dir/
├── MEMORY.md # Long-term memory: persistent info such as user preferences
├── memory/
│ └── YYYY-MM-DD.md # Daily journal: automatically written after each conversation
├── dialog/ # Raw conversation records: full dialog before compression
│ └── YYYY-MM-DD.jsonl # Daily conversation messages in JSONL format
└── tool\_result/ # Cache for long tool outputs (auto-managed, expired entries auto-cleaned)
 └── .txt
\`\`\`

\### Core capabilities

\[ReMeLight\](reme/reme\_light.py) is the core class of the file-based memory system. It provides full memory management
capabilities for AI agents:

| Category | Method | Function | Key components |
| --- | --- | --- | --- |
| Context Management | `check_context` | 📊 Check context size | [ContextChecker](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/reme/memory/file_based/components/context_checker.py) — checks whether context exceeds thresholds and splits messages |
| `compact_memory` | 📦 Compact history into summary | [Compactor](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/reme/memory/file_based/components/compactor.py) — ReActAgent that generates structured context summaries |
| `compact_tool_result` | ✂️ Compact long tool outputs | [ToolResultCompactor](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/reme/memory/file_based/components/tool_result_compactor.py) — truncates long tool outputs and stores them in `tool_result/` while keeping file references in messages |
| `pre_reasoning_hook` | 🔄 Pre-reasoning hook | `compact_tool_result` \+ `check_context` \+ `compact_memory` \+ `summary_memory` (async) |
| Long-term Memory | `summary_memory` | 📝 Persist important memory to files | [Summarizer](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/reme/memory/file_based/components/summarizer.py) — ReActAgent + file tools (`read` / `write` / `edit`) |
| `memory_search` | 🔍 Semantic memory search | [MemorySearch](https://raw.githubusercontent.com/agentscope-ai/ReMe/main/reme/memory/file_based/tools/memory_search.py) — hybrid retrieval with vectors + BM25 |
| Session Memory | `get_in_memory_memory` | 💾 Create in-session memory instance | Returns ReMeInMemoryMemory with dialog\_path configured for persistence |
| `await_summary_tasks` | ⏳ Wait for async summary tasks | Block until all background summary tasks complete |
| - | `start` | 🚀 Start memory system | Initialize file storage, file watcher, and embedding cache; clean up expired tool result files |
| - | `close` | 📕 Shutdown and cleanup | Clean up tool result files, stop file watcher, and persist embedding cache |

\-\-\-

\### 🚀 Quick start

\#### Installation

\*\*Install from source:\*\*

\`\`\`bash
git clone https://github.com/agentscope-ai/ReMe.git
cd ReMe
pip install -e ".\[light\]"
\`\`\`

\*\*Update to the latest version:\*\*

\`\`\`bash
git pull
pip install -e ".\[light\]"
\`\`\`

\#### Environment variables

\`ReMeLight\` uses environment variables to configure the embedding model and storage backends:

\| Variable \| Description \| Example \|
\|----------------------\|-------------------------------\|-----------------------------------------------------\|
\| \`LLM\_API\_KEY\` \| LLM API key \| \`sk-xxx\` \|
\| \`LLM\_BASE\_URL\` \| LLM base URL \| \`https://dashscope.aliyuncs.com/compatible-mode/v1\` \|
\| \`EMBEDDING\_API\_KEY\` \| Embedding API key (optional) \| \`sk-xxx\` \|
\| \`EMBEDDING\_BASE\_URL\` \| Embedding base URL (optional) \| \`https://dashscope.aliyuncs.com/compatible-mode/v1\` \|

\#### Python usage

\`\`\`python
import asyncio

from reme.reme\_light import ReMeLight

async def main():
 # Initialize ReMeLight
 reme = ReMeLight(
 default\_as\_llm\_config={"model\_name": "qwen3.5-35b-a3b"},
 # default\_embedding\_model\_config={"model\_name": "text-embedding-v4"},
 default\_file\_store\_config={"fts\_enabled": True, "vector\_enabled": False},
 enable\_load\_env=True,
 )
 await reme.start()

 messages = \[...\] # List of conversation messages

 # 1\. Check context size (token counting, determine if compaction is needed)
 messages\_to\_compact, messages\_to\_keep, is\_valid = await reme.check\_context(
 messages=messages,
 memory\_compact\_threshold=90000, # Threshold to trigger compaction (tokens)
 memory\_compact\_reserve=10000, # Token count to reserve for recent messages
 )

 # 2\. Compact conversation history into a structured summary
 summary = await reme.compact\_memory(
 messages=messages,
 previous\_summary="",
 max\_input\_length=128000, # Model context window (tokens)
 compact\_ratio=0.7, # Trigger compaction when exceeding max\_input\_length \* 0.7
 language="zh", # Summary language (e.g., "zh" / "")
 )

 # 3\. Compact long tool outputs (prevent tool results from blowing up context)
 messages = await reme.compact\_tool\_result(messages)

 # 4\. Pre-reasoning hook (auto compact tool results + check context + generate summaries)
 processed\_messages, compressed\_summary = await reme.pre\_reasoning\_hook(
 messages=messages,
 system\_prompt="You are a helpful AI assistant.",
 compressed\_summary="",
 max\_input\_length=128000,
 compact\_ratio=0.7,
 memory\_compact\_reserve=10000,
 enable\_tool\_result\_compact=True,
 tool\_result\_compact\_keep\_n=3,
 )

 # 5\. Persist important memory to files (writes to memory/YYYY-MM-DD.md)
 summary\_result = await reme.summary\_memory(
 messages=messages,
 language="zh",
 )

 # 6\. Semantic memory search (vector + BM25 hybrid retrieval)
 result = await reme.memory\_search(query="Python version preference", max\_results=5)

 # 7\. Create in-session memory instance (manages context for one conversation)
 memory = reme.get\_in\_memory\_memory() # Auto-configures dialog\_path
 for msg in messages:
 await memory.add(msg)
 token\_stats = await memory.estimate\_tokens(max\_input\_length=128000)
 print(f"Current context usage: {token\_stats\['context\_usage\_ratio'\]:.1f}%")
 print(f"Message token count: {token\_stats\['messages\_tokens'\]}")
 print(f"Estimated total tokens: {token\_stats\['estimated\_tokens'\]}")

 # 8\. Mark messages as compressed (auto-persists to dialog/YYYY-MM-DD.jsonl)
 # await memory.mark\_messages\_compressed(messages\_to\_compact)

 # Shutdown ReMeLight
 await reme.close()

if \_\_name\_\_ == "\_\_main\_\_":
 asyncio.run(main())
\`\`\`

\> 📂 Full example: \[test\_reme\_light.py\](tests/light/test\_reme\_light.py)
\> 📋 Sample run log: \[test\_reme\_light\_log.txt\](tests/light/test\_reme\_light\_log.txt) (223,838 tokens → 1,105 tokens, 99.5%
\> compression)

\### Architecture of the file-based ReMeLight memory system

\[CoPaw MemoryManager\](https://github.com/agentscope-ai/CoPaw/blob/main/src/copaw/agents/memory/memory\_manager.py)
inherits
\`ReMeLight\` and integrates its memory capabilities into the agent reasoning loop:

\`\`\`mermaid
graph LR
 Agent\[Agent\] -->\|Before each reasoning step\| Hook\[pre\_reasoning\_hook\]
 Hook --> TC\[compact\_tool\_result\
\
Compact tool outputs\]
 TC --> CC\[check\_context\
\
Token counting\]
 CC -->\|Exceeds limit\| CM\[compact\_memory\
\
Generate summary\]
 CC -->\|Exceeds limit\| SM\[summary\_memory\
\
Async persistence\]
 SM -->\|ReAct + FileIO\| Files\[memory/\*.md\]
 CC -->\|Exceeds limit\| MMC\[mark\_messages\_compressed\
\
Persist raw dialog\]
 MMC --> Dialog\[dialog/\*.jsonl\]
 Agent -->\|Explicit call\| Search\[memory\_search\
\
Vector+BM25\]
 Agent -->\|In - session\| InMem\[ReMeInMemoryMemory\
\
Token-aware memory\]
 InMem -->\|Compress/Clear\| Dialog
 Files -.->\|FileWatcher\| Store\[(FileStore\
\
Vector+FTS index)\]
 Search --> Store
\`\`\`

\-\-\-

\#### 1\. \`check\_context\` — context checking

\[ContextChecker\](reme/memory/file\_based/components/context\_checker.py) uses token counting to determine whether the
context exceeds thresholds and automatically splits messages into a "to compact" group and a "to keep" group.

\`\`\`mermaid
graph LR
 M\[messages\] --> H\[AsMsgHandler\
\
Token counting\]
 H --> C{total > threshold?}
 C -->\|No\| K\[Return all messages\]
 C -->\|Yes\| S\[Keep from tail\
\
reserve tokens\]
 S --> CP\[messages\_to\_compact\
\
Earlier messages\]
 S --> KP\[messages\_to\_keep\
\
Recent messages\]
 S --> V{is\_valid

Tool calls aligned?}
\`\`\`

\- \*\*Core logic\*\*: keep \`reserve\` tokens from the tail; mark the rest as messages to compact.
\- \*\*Integrity guarantee\*\*: preserves complete user-assistant turns and tool\_use/tool\_result pairs without splitting
 them.

\-\-\-

\#### 2\. \`compact\_memory\` — conversation compaction

\[Compactor\](reme/memory/file\_based/components/compactor.py) uses a ReActAgent to compact conversation history into a \*
\*structured context summary\*\*.

\`\`\`mermaid
graph LR
 M\[messages\] --> H\[AsMsgHandler\
\
format\_msgs\_to\_str\]
 H --> A\[ReActAgent\
\
reme\_compactor\]
 P\[previous\_summary\] -->\|Incremental update\| A
 A --> S\[Structured summary\
\
Goal/Progress/Decisions...\]
\`\`\`

\*\*Summary structure\*\* (context checkpoints):

\| Field \| Description \|
\|-----------------------\|------------------------------------------------------------------------\|
\| \`## Goal\` \| User goals \|
\| \`## Constraints\` \| Constraints and preferences \|
\| \`## Progress\` \| Task progress \|
\| \`## Key Decisions\` \| Key decisions \|
\| \`## Next Steps\` \| Next step plans \|
\| \`## Critical Context\` \| Critical data such as file paths, function names, error messages, etc. \|

\- \*\*Incremental updates\*\*: when \`previous\_summary\` is provided, new conversations are merged into the existing summary.

\-\-\-

\#### 3\. \`summary\_memory\` — persistent memory

\[Summarizer\](reme/memory/file\_based/components/summarizer.py) uses a \*\*ReAct + file tools\*\* pattern so that the AI can
decide what to write and where to write it.

\`\`\`mermaid
graph LR
 M\[messages\] --> A\[ReActAgent\
\
reme\_summarizer\]
 A -->\|read\| R\[Read memory/YYYY-MM-DD.md\]
 R --> T{Reason: how to merge?}
 T -->\|write\| W\[Overwrite\]
 T -->\|edit\| E\[Edit in place\]
 W --> F\[memory/YYYY-MM-DD.md\]
 E --> F
\`\`\`

\*\*File tools\*\* (\[FileIO\](reme/memory/file\_based/tools/file\_io.py)):

\| Tool \| Function \|
\|---------\|-----------------------\|
\| \`read\` \| Read file content \|
\| \`write\` \| Overwrite file \|
\| \`edit\` \| Find-and-replace edit \|

\-\-\-

\#### 4\. \`compact\_tool\_result\` — tool result compaction

\[ToolResultCompactor\](reme/memory/file\_based/components/tool\_result\_compactor.py) addresses the problem of long tool
outputs bloating the context.

\`\`\`mermaid
graph LR
 M\[messages\] --> L{Iterate tool\_result

len > threshold?}
 L -->\|No\| K\[Keep as-is\]
 L -->\|Yes\| T\[truncate\_text\
\
Truncate to threshold\]
 T --> S\[Write full content\
\
tool\_result/uuid.txt\]
 S --> R\[Append file path reference\
\
to message\]
 R --> C\[cleanup\_expired\_files\
\
Delete expired files\]
\`\`\`

\- \*\*Auto cleanup\*\*: expired files (older than \`retention\_days\`) are deleted automatically during \`start\` / \`close\` /
 \`compact\_tool\_result\`.

\-\-\-

\#### 5\. \`memory\_search\` — memory retrieval

\[MemorySearch\](reme/memory/file\_based/tools/memory\_search.py) provides \*\*vector + BM25 hybrid retrieval\*\*.

\`\`\`mermaid
graph LR
 Q\[query\] --> E\[Embedding\
\
Vectorization\]
 E --> V\[vector\_search\
\
Semantic similarity\]
 Q --> B\[BM25\
\
Keyword matching\]
 V -->\|" weight: 0.7 "\| M\[Deduplicate + weighted merge\]
 B -->\|" weight: 0.3 "\| M
 M --> F\[min\_score filter\]
 F --> R\[Top-N results\]
\`\`\`

\- \*\*Fusion mechanism\*\*: vector weight 0.7 + BM25 weight 0.3 — balancing semantic similarity and exact matches.

\-\-\-

\#### 6\. \`ReMeInMemoryMemory\` — in-session memory

\[ReMeInMemoryMemory\](reme/memory/file\_based/reme\_in\_memory\_memory.py) extends AgentScope's \`InMemoryMemory\` to provide
token-aware memory management and raw conversation persistence.

\`\`\`mermaid
graph LR
 C\[content\] --> G\[get\_memory\
\
exclude\_mark=COMPRESSED\]
 G --> F\[Filter out compressed messages\]
 F --> P{prepend\_summary?}
 P -->\|Yes\| S\[Prepend previous summary\]
 S --> O\[Output messages\]
 P -->\|No\| O
 M\[mark\_messages\_compressed\] --> D\[Persist to dialog/YYYY-MM-DD.jsonl\]
 D --> R\[Remove from memory\]
\`\`\`

\| Function \| Description \|
\|----------------------------------\|----------------------------------------------------------\|
\| \`get\_memory\` \| Filter messages by mark and auto-append summary \|
\| \`estimate\_tokens\` \| Estimate token usage of the context \|
\| \`state\_dict\` / \`load\_state\_dict\` \| Serialize/deserialize state (session persistence) \|
\| \`mark\_messages\_compressed\` \| Mark messages compressed and persist to dialog directory \|
\| \`clear\_content\` \| Persist all messages before clearing memory \|

\*\*Raw conversation persistence\*\*: When messages are compressed or cleared, they are automatically saved to
\`{dialog\_path}/{date}.jsonl\` with one JSON-formatted message per line.

\-\-\-

\#### 7\. \`pre\_reasoning\_hook\` — pre-reasoning processing

This is a unified entry point that wires all the above components together and automatically manages context before each
reasoning step.

\`\`\`mermaid
graph LR
 M\[messages\] --> TC\[compact\_tool\_result\
\
Compact long tool outputs\]
 TC --> CC\[check\_context\
\
Compute remaining space\]
 CC --> D{messages\_to\_compact

Non-empty?}
 D -->\|No\| K\[Return original messages + summary\]
 D -->\|Yes\| V{is\_valid?}
 V -->\|No\| K
 V -->\|Yes\| CM\[compact\_memory\
\
Sync summary generation\]
 V -->\|Yes\| SM\[add\_async\_summary\_task\
\
Async persistence\]
 CM --> R\[Return messages\_to\_keep + new summary\]
\`\`\`

\*\*Execution flow\*\*:

1\. \`compact\_tool\_result\` — compact long tool outputs.
2\. \`check\_context\` — check whether the context exceeds limits.
3\. \`compact\_memory\` — generate compact summary (sync).
4\. \`summary\_memory\` — persist memory (async in the background).

\-\-\-

\## 🗃️ Vector-based memory system

\[ReMe Vector Based\](reme/reme.py) is the core class for the vector-based memory system. It manages three types of
memories:

\| Memory type \| Use case \|
\|-----------------------\|-------------------------------------------------------------------\|
\| \*\*Personal memory\*\* \| Records user preferences and habits \|
\| \*\*Procedural memory\*\* \| Records task execution experience and patterns of success/failure \|
\| \*\*Tool memory\*\* \| Records tool usage experience and parameter tuning \|

\### Core capabilities

\| Method \| Function \| Description \|
\|--------------------\|--------------\|-------------------------------------------------------------\|
\| \`summarize\_memory\` \| 🧠 Summarize \| Automatically extract and store memories from conversations \|
\| \`retrieve\_memory\` \| 🔍 Retrieve \| Retrieve related memories based on a query \|
\| \`add\_memory\` \| ➕ Add \| Manually add memories into the vector store \|
\| \`get\_memory\` \| 📖 Get \| Get a single memory by ID \|
\| \`update\_memory\` \| ✏️ Update \| Update existing memory content or metadata \|
\| \`delete\_memory\` \| 🗑️ Delete \| Delete a specific memory \|
\| \`list\_memory\` \| 📋 List \| List memories with filtering and sorting \|

\### Installation and environment variables

Installation and environment configuration are the same as \[ReMeLight\](#installation).
API keys are configured via environment variables and can be stored in a \`.env\` file at the project root.

\### Python usage

\`\`\`python
import asyncio

from reme import ReMe

async def main():
 # Initialize ReMe
 reme = ReMe(
 working\_dir=".reme",
 default\_llm\_config={
 "backend": "openai",
 "model\_name": "qwen3.5-plus",
 },
 default\_embedding\_model\_config={
 "backend": "openai",
 "model\_name": "text-embedding-v4",
 "dimensions": 1024,
 },
 default\_vector\_store\_config={
 "backend": "local", # Supports local/chroma/qdrant/elasticsearch
 },
 )
 await reme.start()

 messages = \[\
 {"role": "user", "content": "Help me write a Python script", "time\_created": "2026-02-28 10:00:00"},\
 {"role": "assistant", "content": "Sure, I'll help you with that.", "time\_created": "2026-02-28 10:00:05"},\
 \]

 # 1\. Summarize memories from conversation (automatically extract user preferences, task experience, etc.)
 result = await reme.summarize\_memory(
 messages=messages,
 user\_name="alice", # Personal memory
 # task\_name="code\_writing", # Procedural memory
 )
 print(f"Summary result: {result}")

 # 2\. Retrieve related memories
 memories = await reme.retrieve\_memory(
 query="Python programming",
 user\_name="alice",
 # task\_name="code\_writing",
 )
 print(f"Retrieved memories: {memories}")

 # 3\. Manually add a memory
 memory\_node = await reme.add\_memory(
 memory\_content="The user prefers concise code style.",
 user\_name="alice",
 )
 print(f"Added memory: {memory\_node}")
 memory\_id = memory\_node.memory\_id

 # 4\. Get a single memory by ID
 fetched\_memory = await reme.get\_memory(memory\_id=memory\_id)
 print(f"Fetched memory: {fetched\_memory}")

 # 5\. Update memory content
 updated\_memory = await reme.update\_memory(
 memory\_id=memory\_id,
 user\_name="alice",
 memory\_content="The user prefers concise code with comments.",
 )
 print(f"Updated memory: {updated\_memory}")

 # 6\. List all memories for the user (supports filtering and sorting)
 all\_memories = await reme.list\_memory(
 user\_name="alice",
 limit=10,
 sort\_key="time\_created",
 reverse=True,
 )
 print(f"User memory list: {all\_memories}")

 # 7\. Delete a specific memory
 await reme.delete\_memory(memory\_id=memory\_id)
 print(f"Deleted memory: {memory\_id}")

 # 8\. Delete all memories (use with care)
 # await reme.delete\_all()

 await reme.close()

if \_\_name\_\_ == "\_\_main\_\_":
 asyncio.run(main())
\`\`\`

\### Technical architecture

\`\`\`mermaid
graph LR
 User\[User / Agent\] --> ReMe\[Vector Based ReMe\]
 ReMe --> Summarize\[Summarize memories\]
 ReMe --> Retrieve\[Retrieve memories\]
 ReMe --> CRUD\[CRUD operations\]
 Summarize --> PersonalSum\[PersonalSummarizer\]
 Summarize --> ProceduralSum\[ProceduralSummarizer\]
 Summarize --> ToolSum\[ToolSummarizer\]
 Retrieve --> PersonalRet\[PersonalRetriever\]
 Retrieve --> ProceduralRet\[ProceduralRetriever\]
 Retrieve --> ToolRet\[ToolRetriever\]
 PersonalSum --> VectorStore\[Vector database\]
 ProceduralSum --> VectorStore
 ToolSum --> VectorStore
 PersonalRet --> VectorStore
 ProceduralRet --> VectorStore
 ToolRet --> VectorStore
\`\`\`

\### Experimental results

Evaluations are conducted on two benchmarks: \*\*LoCoMo\*\* and \*\*HaluMem\*\*. Experimental settings:

1\. \*\*ReMe backbone\*\*: as specified in each table.
2\. \*\*Evaluation protocol\*\*: LLM-as-a-Judge following MemOS — each answer is scored by GPT-4o-mini.

Baseline results are reproduced from their respective papers under aligned settings where possible.

\### LoCoMo

\| Method \| Single Hop \| Multi Hop \| Temporal \| Open Domain \| Overall \|
\|----------\|------------\|-----------\|-----------\|-------------\|-----------\|
\| MemoryOS \| 62.43 \| 56.50 \| 37.18 \| 40.28 \| 54.70 \|
\| Mem0 \| 66.71 \| 58.16 \| 55.45 \| 40.62 \| 61.00 \|
\| MemU \| 72.77 \| 62.41 \| 33.96 \| 46.88 \| 61.15 \|
\| MemOS \| 81.45 \| 69.15 \| 72.27 \| 60.42 \| 75.87 \|
\| HiMem \| 89.22 \| 70.92 \| 74.77 \| 54.86 \| 80.71 \|
\| Zep \| 88.11 \| 71.99 \| 74.45 \| 66.67 \| 81.06 \|
\| TiMem \| 81.43 \| 62.20 \| 77.63 \| 52.08 \| 75.30 \|
\| TSM \| 84.30 \| 66.67 \| 71.03 \| 58.33 \| 76.69 \|
\| MemR3 \| 89.44 \| 71.39 \| 76.22 \| 61.11 \| 81.55 \|
\| \*\*ReMe\*\* \| \*\*89.89\*\* \| \*\*82.98\*\* \| \*\*83.80\*\* \| \*\*71.88\*\* \| \*\*86.23\*\* \|

\### HaluMem

\| Method \| Memory Integrity \| Memory Accuracy \| QA Accuracy \|
\|-------------\|------------------\|-----------------\|-------------\|
\| MemoBase \| 14.55 \| 92.24 \| 35.53 \|
\| Supermemory \| 41.53 \| 90.32 \| 54.07 \|
\| Mem0 \| 42.91 \| 86.26 \| 53.02 \|
\| ProMem \| \*\*73.80\*\* \| 89.47 \| 62.26 \|
\| \*\*ReMe\*\* \| 67.72 \| \*\*94.06\*\* \| \*\*88.78\*\* \|

\-\-\-

\## 🧪 Procedural memory paper

\> Our procedural (task) memory paper is available on \[arXiv\](https://arxiv.org/abs/2512.10696).

\### 🌍 \[Appworld benchmark\](benchmark/appworld/quickstart.md)

We evaluate ReMe on the Appworld environment using Qwen3-8B (non-thinking mode):

\| Method \| Avg@4 \| Pass@4 \|
\|----------\|---------------------\|---------------------\|
\| w/o ReMe \| 0.1497 \| 0.3285 \|
\| w/ ReMe \| 0.1706 \*\*(+2.09%)\*\* \| 0.3631 \*\*(+3.46%)\*\* \|

Pass@K measures the probability that at least one of K generated candidates successfully completes the task (score=1).
The current experiments use an internal AppWorld environment, which may differ slightly from the public version.

For more details on how to reproduce the experiments, see \[quickstart.md\](benchmark/appworld/quickstart.md).

\### 🔧 \[BFCL-V3 benchmark\](benchmark/bfcl/quickstart.md)

We evaluate ReMe on the BFCL-V3 multi-turn-base task (random split 50 train / 150 val) using Qwen3-8B (thinking mode):

\| Method \| Avg@4 \| Pass@4 \|
\|----------\|---------------------\|---------------------\|
\| w/o ReMe \| 0.4033 \| 0.5955 \|
\| w/ ReMe \| 0.4450 \*\*(+4.17%)\*\* \| 0.6577 \*\*(+6.22%)\*\* \|

For more details on how to reproduce the experiments, see \[quickstart.md\](benchmark/bfcl/quickstart.md).

\## ⭐ Community & support

\- \*\*Star & Watch\*\*: Starring helps more agent developers discover ReMe; Watching keeps you up to date with new releases
 and features.
\- \*\*Share your results\*\*: Share how ReMe empowers your agents in Issues or Discussions — we are happy to showcase great
 community use cases.
\- \*\*Need a new feature?\*\* Open a feature request; we’ll evolve ReMe together with the community.
\- \*\*Code contributions\*\*: All forms of contributions are welcome. Please see
 the \[contribution guide\](docs/contribution.md).
\- \*\*Acknowledgements\*\*: We thank excellent open-source projects such as OpenClaw, Mem0, MemU, and CoPaw for their
 inspiration and support.

\### Contributors

Thanks to all who have contributed to ReMe:

[![Contributors](https://contrib.rocks/image?repo=agentscope-ai/ReMe)](https://github.com/agentscope-ai/ReMe/graphs/contributors)

\-\-\-

\## 📄 Citation

\`\`\`bibtex
@software{AgentscopeReMe2025,
 title = {AgentscopeReMe: Memory Management Kit for Agents},
 author = {ReMe Team},
 url = {https://reme.agentscope.io},
 year = {2025}
}
\`\`\`

\-\-\-

\## ⚖️ License

This project is open-sourced under the Apache License 2.0. See \[LICENSE\](./LICENSE) for details.

\-\-\-

\## 🤔 Why ReMe?

ReMe stands for \*\*Remember Me\*\* and \*\*Refine Me\*\*, symbolizing our goal to help AI agents "remember" users and "refine"
themselves through interactions. We hope ReMe is not just a cold memory module, but a partner that truly helps agents
understand users, accumulate experience, and continuously evolve.

\-\-\-

\## 📈 Star history

\[!\[Star History Chart\](https://api.star-history.com/svg?repos=agentscope-ai/ReMe&type=Date)\](https://www.star-history.com/#agentscope-ai/ReMe&Date)