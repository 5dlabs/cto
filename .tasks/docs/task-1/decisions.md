## Decision Points

- Tokens can be consumed by extending `tailwind.config.ts` at build time (all values compiled into utility classes) OR by exporting CSS custom properties at runtime (e.g., `--color-primary`) and referencing them in Tailwind's `theme.extend` via `var()`. Build-time is simpler but less dynamic; runtime allows JS-driven theming and easier downstream overrides. This is a hard constraint that affects every component subtask.
- Next.js provides `next/font/google` for zero-CLS optimized font loading. Alternatively, fonts could be loaded via Google Fonts CDN link (simpler but worse performance) or self-hosted in `public/fonts` (full control, no external dependency). Affects layout shell, LCP, and all component typography.
- The `/api/snapshot` route can return a hardcoded `{ tokensApplied: true }` (static assertion, simple but no real validation) or perform runtime introspection by importing the tokens module and checking its keys/values exist (stronger guarantee, more coupling). Also: should the response include actual token values, or just component names? Token values could leak design decisions but aid downstream agents.
- The `/api/snapshot` route exists to validate the design snapshot E2E flow. Should it remain in production as a diagnostic/introspection endpoint, or should it be removed after validation is confirmed? If permanent, it needs security considerations (should not expose internals to public). If temporary, it should be clearly marked for removal.
- Since Stitch candidate generation failed, the implementation team is inventing a design token palette from scratch based on design-intent principles. This is an architectural decision that should be explicitly approved rather than silently defaulted. The chosen palette (colors, type scale, spacing) will become the source of truth for all downstream agents and future work.

## Coordination Notes

- Agent owner: blaze
- Primary stack: React/Next.js