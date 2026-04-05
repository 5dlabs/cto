# Pitch Deck Staging

Use this flow to publish a visual-first investor deck variant without touching the live deck.

## 1) Build and deploy staging

From `apps/website`:

```bash
npm run deploy:pitch-staging
```

This pushes the static export to the Cloudflare Pages preview branch named `pitch-staging`.

## 2) Bind the staging subdomain

In Cloudflare Pages for project `5dlabs-splash`:

1. Open **Custom domains**
2. Add `pitch-staging.5dlabs.ai`
3. Attach it to the `pitch-staging` branch deployment

## 3) Staging route

The visual-first draft deck lives at:

- `/investors/staging/`

If you bind `pitch-staging.5dlabs.ai`, the full URL is:

- `https://pitch-staging.5dlabs.ai/investors/staging/`

## Notes

- The staging page is set to `noindex` to avoid search indexing.
- Keep `pitch.5dlabs.ai` unchanged until this variant is approved.
