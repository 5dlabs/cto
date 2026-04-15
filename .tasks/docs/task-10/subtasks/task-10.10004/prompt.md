<identity>
You are atlas working on subtask 10004 of task 10.
</identity>

<context>
<scope>
Add a release packaging job that creates a GitHub Release with all deliverables: program binary (.so), IDL JSON, dashboard static export, demo CLI bundle, and security review document.
</scope>
</context>

<implementation_plan>
1. Add `release` job to ci.yml:
   - Trigger: on push of version tag (e.g., `v*`) or manual `workflow_dispatch`.
   - Set `needs: [anchor-test, deploy-devnet]` (or just anchor-test if release should work without deploy).
2. Download all artifacts from previous stages:
   - `cto_billing.so` from anchor-build.
   - `cto_billing.json` (IDL) from anchor-build.
   - Dashboard build output from dashboard-build.
   - `deployment-manifest.json` from deploy-devnet (if available).
3. Package artifacts:
   - Zip dashboard output: `zip -r dashboard-export.zip app/dashboard/out/` (or `.next/standalone/`).
   - Bundle demo CLI: `cd cli && bun build ./src/index.ts --outfile cto-billing-cli --compile --target=bun-linux-x64` (or just zip the source + package.json for `bun run`).
   - Copy `docs/security/solana-program-review.md` to release assets.
4. Create GitHub Release using `softprops/action-gh-release@v1` or `gh release create`:
   - Tag: use the git tag or generate `demo-YYYYMMDD-HHMMSS`.
   - Title: `CTO Billing - Hackathon Demo Release`.
   - Body: include deployment manifest details (program ID, devnet cluster), test results summary, links to docs.
   - Assets: cto_billing.so, cto_billing.json, dashboard-export.zip, cto-billing-cli (or zip), solana-program-review.md.
5. Verify the release appears on the GitHub Releases page with all assets downloadable.
</implementation_plan>

<validation>
Trigger the release job (via tag push or manual dispatch). A GitHub Release is created with title and description. All 5 asset types are attached and downloadable: .so binary (>100KB), IDL JSON (valid JSON), dashboard zip, CLI bundle, security review markdown. Release body contains the deployed program ID and cluster info.
</validation>