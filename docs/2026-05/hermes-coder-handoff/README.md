# Hermes Coder handoff: OVH VM deployment and CTO PRs

This is the handoff for the in-progress branch `hermes/sigma-quick-intake-linearless`.

## Current branch contents

The branch currently combines four streams of work:

1. **Deliberation audio safety**
   - `apps/lobster-voice` now records the transcript path, transcript SHA-256, session ID, and generated timestamp in render status.
   - Audio validation now records both the transcript hash and MP3 hash.
   - Video generation now requires a validation artifact for the current transcript hash, not just any previously validated MP3.
   - Render startup clears stale logs, MP3 output, and validation JSON so failed or stale artifacts cannot be reused.
   - `ELEVENLABS_API_KEY` is accepted as an alias for `ELEVEN_API_KEY`.

2. **Hermes Coder runtime**
   - `infra/gitops/agents/hermes-coder-values.yaml` requests 8 CPU and 64 GiB RAM for the persistent Coder pod, with no CPU limit.
   - The Hermes agent Helm chart has an optional browser-based remote desktop sidecar using `kasmweb/chrome`.
   - `desktop.5dlabs.ai` is wired through the platform Cloudflare tunnel to the Hermes Coder service on port 6901.
   - The remote desktop sidecar is disabled by default in chart values and enabled only for Hermes Coder GitOps values.

3. **OVH metal provider**
   - The `metal` CLI now accepts `OVH_SUBSIDIARY`.
   - Hermes and OpenClaw agent charts expose `OVH_SUBSIDIARY` from the API keys secret.
   - OVH ordering now uses the requested deployment region/datacenter for the cart `region` configuration instead of deriving it from the account subsidiary.
   - Provider docs clarify that `OVH_SUBSIDIARY` is the account/login subsidiary, not the deployment region.

4. **Scenario MCP plumbing**
   - Hermes chart values include Scenario MCP tool configuration.
   - Optional Scenario API key environment variables are mapped from the API keys secret.

## Checks already run

The following focused checks were run successfully while this work was in progress:

- `intake/tests/audio-safety-tests.sh`
- `npm run typecheck -- --pretty false` in `apps/lobster-voice`
- `npm exec --yes --package bun -- bun run build` in `apps/lobster-voice`
- `cargo test -p metal ovh:: --quiet`
- `helm lint infra/charts/hermes-agent --quiet`
- `helm template` for the Hermes agent chart with remote desktop disabled and enabled

## Important deployment findings

- The cloud/provider scope is **OVH only**.
- Jonathon's OVH login/account subsidiary is Canadian, so use `OVH_SUBSIDIARY=CA`.
- The account subsidiary is separate from placement. The requested OVH deployment region/datacenter should still be passed as the server region, for example `bhs` if deploying in Beauharnois.
- Public-cloud metal VMs are the default deployment path. Private-cloud or ISO work should stay deferred unless public-cloud metal cannot satisfy the Talos/CTO deployment path.
- Do not treat `OVH_SUBSIDIARY=CA` as a constraint that all resources must be deployed in Canada; it is the login/subsidiary context for OVH API/cart operations.

## What still needs to be done

1. **Refresh and split or land the PR**
   - Rebase this branch onto the current `main`; it is behind upstream.
   - Decide whether to keep this as one handoff PR or split it into separate PRs:
     - audio safety
     - Hermes Coder runtime/desktop/resources
     - OVH metal provider fixes
     - Scenario MCP plumbing
   - Rerun the focused checks after the rebase/split.

2. **Prepare OVH secrets**
   - Ensure the agent API keys secret contains:
     - `ovh-application-key`
     - `ovh-application-secret`
     - `ovh-consumer-key`
     - `ovh-subsidiary` set to `CA`
   - For remote desktop, ensure `cto-coder-api-keys` contains `remote-desktop-password`.
   - For Scenario MCP, add Scenario credentials only if the integration is intended to be active.

3. **Validate OVH ordering before paid checkout**
   - Confirm the selected OVH metal plan, datacenter/region, duration, and `none_64.en` OS choice for Talos.
   - Exercise the cart/configuration flow against OVH with the real account and verify the cart item contains:
     - `dedicated_datacenter`
     - `dedicated_os=none_64.en`
     - `region=<requested deployment region>`
   - Only proceed to checkout once Jonathon has approved the paid order details.

4. **Deploy the VM and CTO platform**
   - Provision the OVH metal VM through the `metal` CLI/provider once the ordering path is confirmed.
   - Bootstrap Talos on the VM using the existing CTO Talos/GitOps deployment path.
   - Apply or sync the CTO GitOps manifests after the PR containing the required chart/provider changes lands.
   - Verify the agent namespace and tunnels after sync, especially:
     - Hermes Coder pod resources
     - `coder.5dlabs.ai`
     - `desktop.5dlabs.ai`
     - Cloudflare Access protection for the desktop endpoint

## Risks and notes for the next agent

- The current branch has unrelated streams bundled together. Splitting will make review safer, but a single handoff PR is faster if the priority is preserving context.
- Remote desktop exposes a browser desktop surface. Keep it behind Cloudflare Access and a strong Kasm VNC password.
- OVH checkout can create paid resources. Treat the final checkout call as an approval boundary.
- The OVH fix is important before deployment: using the subsidiary as the cart region could order in the wrong placement context or fail cart validation.
