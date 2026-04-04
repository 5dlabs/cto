Implement subtask 3003: Implement graceful degradation for missing API key, timeouts, and API errors

## Objective
Add all error handling paths to fetchResearchMemo: missing NOUS_API_KEY skip with info log, 30s timeout handling with warning log, and HTTP error handling with warning log. None of these should throw.

## Steps
1. At the top of `fetchResearchMemo`, check if `NOUS_API_KEY` is falsy. If so, log an info message exactly: 'Hermes integration skipped: NOUS_API_KEY not configured' and return null immediately.
2. Wrap the fetch call in a try/catch. If the AbortController fires (timeout after 30s), catch the AbortError, log a warning with the task context identifier (e.g., 'Hermes API timeout for task <id>'), and return null.
3. After the fetch, check `response.ok`. If the status is not 2xx, log a warning including the HTTP status code (e.g., 'Hermes API error: status 500 for task <id>'), and return null.
4. Catch any other unexpected errors (network errors, JSON parse errors), log them as warnings, and return null.
5. Ensure no code path in fetchResearchMemo can throw an unhandled exception — the pipeline must never fail due to Hermes unavailability.

## Validation
Unit test 1: With NOUS_API_KEY unset, fetchResearchMemo returns null and the info log 'Hermes integration skipped' is emitted. Unit test 2: With a mocked API that never responds (simulating 30s+ delay), fetchResearchMemo returns null and a timeout warning is logged. Unit test 3: With a mocked API returning HTTP 500, fetchResearchMemo returns null and the error status is logged. Unit test 4: With a mocked API returning malformed JSON, fetchResearchMemo returns null without throwing.