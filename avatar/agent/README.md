# Morgan Avatar Agent

Python LiveKit agent for the Morgan talking-avatar proof of concept.

## What it does

- Connects Morgan to a LiveKit room as a voice agent.
- Publishes a LemonSlice avatar created from a still image.
- Routes user speech through configurable STT, LLM, and TTS backends.
- Writes turn-by-turn latency logs to `runs/`.

## Backends

| Component | Options |
|-----------|---------|
| STT | `livekit-flux` (default), `livekit-nova`, `deepgram-flux`, `deepgram-nova` |
| TTS | `livekit-elevenlabs` (default), `elevenlabs`, `cartesia`, `livekit-cartesia` |
| LLM | `openclaw` (default), `inference` |

## Run locally

1. Copy `.env.example` to `.env` and fill in credentials.
2. Create a Python 3.11 virtual environment and install:
   ```bash
   python3.11 -m venv .venv
   source .venv/bin/activate
   pip install -e '.[dev]'
   ```
3. Download required model files (first run only):
   ```bash
   python agent.py download-files
   ```
4. Start the agent:
   ```bash
   SSL_CERT_FILE="$(python -c 'import certifi; print(certifi.where())')" \
   REQUESTS_CA_BUNDLE="$(python -c 'import certifi; print(certifi.where())')" \
   python agent.py dev
   ```

The SSL environment variables ensure Python uses the `certifi` CA bundle,
which resolves certificate verification errors connecting to LiveKit Cloud.

## Routing contract

`morgan.5dlabs.ai` hosts a shared OpenClaw gateway. Without explicit routing,
requests fall back to a generic assistant persona. To reach the Morgan persona
the request **must** include:

```
x-openclaw-agent-id: morgan
```

The Python agent sends this header automatically when `MORGAN_LLM_AGENT_ID` is
set (see `providers.py` line 18). The `.env` default is `morgan`.

A future platform-layer change may allow hostname-only routing so that any
request to `morgan.5dlabs.ai` implies the Morgan persona without the header.

## Latency summaries

After a session, summarize the latest run:

```bash
python scripts/summarize_latency.py        # human-readable table
python scripts/summarize_latency.py --json  # raw JSON
```

The summary breaks down:
- **Greeting latency**: time from session start to first TTS audio
- **Conversational turns**: end-of-utterance to first audio
- **Per-component**: p50/p95 for EOU delay, STT delay, LLM TTFT, TTS TTFB
