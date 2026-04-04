Implement subtask 10002: Configure Cloudflare Access application for SSO/MFA on the tunnel

## Objective
Set up a Cloudflare Access application on the tunnel to restrict access to authorized users, providing SSO/MFA at the edge without application-level auth code. This subtask is contingent on D7 resolving to Cloudflare Access (not JWT/RBAC).

## Steps
1. In the Cloudflare Zero Trust dashboard (or via Cloudflare API/Terraform if infrastructure-as-code is used), create an Access Application:
   - Application name: `sigma-1-pm`.
   - Application domain: the tunnel's public hostname.
   - Session duration: 24 hours.
2. Create an Access Policy:
   - Policy name: `sigma-1-authorized-users`.
   - Decision: Allow.
   - Include rule: Email addresses or identity provider group that maps to authorized team members.
3. If the cluster supports the Cloudflare Access CRD, create the Access application declaratively in a YAML manifest `manifests/production/cloudflare-access.yaml` and apply it.
4. If not CRD-managed, document the Cloudflare dashboard configuration steps in `docs/production/cloudflare-access-setup.md` with screenshots or API call examples.
5. Test that unauthenticated requests to the tunnel URL are redirected to the Cloudflare Access login page.

## Validation
Unauthenticated request (curl without cookies/tokens) to the tunnel URL returns a 302 redirect to the Cloudflare Access login page. Authenticated request (after login or with valid CF_Authorization cookie) returns HTTP 200 from the PM server.