Implement subtask 3002: Implement headless browser screenshot capture service

## Objective
Create `src/modules/hermes/artifacts/screenshot-capture.ts` — a service that uses a headless browser to capture PNG screenshots of a given URL, returning the image buffer along with capture metadata (viewport, duration, URL).

## Steps
1. Evaluate and install headless browser library compatible with Bun runtime. Start with Playwright (`bun add playwright`); if Bun incompatibilities arise, fall back to `puppeteer-core` with a system Chromium binary.
2. Create `screenshot-capture.ts` with interface: `captureScreenshot(url: string, options?: CaptureOptions): Promise<CaptureResult>`.
3. `CaptureOptions`: `{ viewport?: { width: number; height: number }; timeout?: number; fullPage?: boolean }`. Defaults: 1920x1080, 30s timeout, fullPage true.
4. Implementation: launch headless Chromium, navigate to URL with `waitUntil: 'networkidle'`, capture full-page PNG screenshot, close browser context.
5. `CaptureResult`: `{ buffer: Buffer; metadata: { url: string; viewport: { width: number; height: number }; capturedAt: string; durationMs: number; fullPage: boolean } }`.
6. Implement browser pool or singleton pattern to avoid launching a new browser process per capture — reuse browser instance, create new contexts per capture for isolation.
7. Handle capture failures: timeout errors, navigation errors (invalid URL, DNS failure, HTTP errors). Throw typed `CaptureError` with `code: 'CAPTURE_FAILED' | 'NAVIGATION_FAILED' | 'TIMEOUT'`.
8. Add graceful shutdown hook to close browser process when the service stops.

## Validation
Integration test: call `captureScreenshot('https://example.com')` and verify the returned buffer is a valid PNG (check PNG magic bytes `89 50 4E 47`), metadata contains the correct URL, viewport is 1920x1080, and `durationMs` is > 0. Error test: call with an invalid URL (e.g., `https://this-does-not-exist.invalid`) and verify a `CaptureError` with code `NAVIGATION_FAILED` is thrown within the timeout period. Verify browser reuse by capturing two screenshots and confirming only one browser process is spawned.