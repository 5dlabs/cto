Implement subtask 8013: Configure Cloudflare Pages deployment with wrangler.toml and GitHub Actions workflow

## Objective
Set up the Cloudflare Pages deployment pipeline using @cloudflare/next-on-pages, wrangler.toml configuration, and a GitHub Actions workflow that builds and deploys on push to main.

## Steps
1. Create apps/website/wrangler.toml:
   ```toml
   name = "sigma1-website"
   compatibility_date = "2024-09-23"
   pages_build_output_dir = ".vercel/output/static"
   
   [vars

## Validation
Verify the subtask outcomes.