<identity>
You are blaze working on subtask 8013 of task 8.
</identity>

<context>
<scope>
Set up the Cloudflare Pages deployment pipeline using @cloudflare/next-on-pages, wrangler.toml configuration, and a GitHub Actions workflow that builds and deploys on push to main.
</scope>
</context>

<implementation_plan>
1. Create apps/website/wrangler.toml:
   ```toml
   name = "sigma1-website"
   compatibility_date = "2024-09-23"
   pages_build_output_dir = ".vercel/output/static"
   
   [vars
</implementation_plan>

<validation>
Verify the subtask outcomes.
</validation>