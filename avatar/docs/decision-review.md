# Decision Review

## Current decision

Stay on the proven `LiveKit + LemonSlice` stack as the default Morgan path.

## Why this is the right default

- The browser client, token route, and Python agent are implemented and locally validated.
- The agent already exposes the first-line latency levers that matter most: preemptive generation, interruption recovery, STT mode swaps, TTS mode swaps, aligned transcripts, and structured latency logging.
- Rewriting the stack or moving immediately to a more integrated voice path would increase delivery risk before we have a real latency baseline from live credentials.

## What is already measured or verified

- Python agent imports successfully under Python `3.11`.
- Python lint passes.
- Python tests pass.
- Next.js lint passes.
- Next.js production build passes.
- Token minting route compiles and the custom room client compiles.

## What still requires live credentials

- End-to-end room join against LiveKit Cloud.
- LemonSlice avatar join and rendered video in a live room.
- OpenClaw gateway round-trip timings.
- Real `p50` and `p95` latency numbers from manual turns.

## Decision gates for the next live session

- If `p50 end_of_turn_to_first_audio` is near target, stay on the baseline stack and tune providers.
- If TTS dominates, switch to the Cartesia spike first.
- If STT endpointing dominates, switch to direct Deepgram Flux tuning first.
- Only run the integrated low-latency voice spike if the above swaps still miss target.
- Only revisit runtime/language changes if the worker itself becomes the bottleneck after provider tuning.
