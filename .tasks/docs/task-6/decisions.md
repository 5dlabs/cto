## Decision Points

- Runtime choice: Bun vs. Node.js 20. The details mention both. Bun offers native Elysia performance but may have compatibility gaps with sharp and @aws-sdk/client-s3. This must be decided before project initialization (subtask 6001).
- Caption generation provider: the description mentions both OpenAI and Claude (Anthropic). The details only reference OpenAI chat API. A single provider must be selected to avoid maintaining two SDK integrations in v1.
- Signal notification mechanism for approval workflow: Signal does not have an official REST API. The implementation likely requires signal-cli or a third-party bridge. The specific integration approach and hosting must be decided before the approval workflow subtask.
- TikTok Content Posting API access: TikTok's API requires approved developer access and is restricted for business use. Confirm API access availability and fallback strategy (e.g., skip TikTok for v1) before implementing TikTokService.
- R2 bucket vs. S3: the stack mentions @aws-sdk/client-s3 but the storage is Cloudflare R2. Confirm the R2 endpoint, credentials format, and whether the S3-compatible SDK is the approved approach, or if a Cloudflare-native SDK is preferred.

## Coordination Notes

- Agent owner: nova
- Primary stack: Node.js 20+/Elysia 1.x/Effect 3.x