Implement subtask 1002: Provision sealed secrets for Linear API, Discord webhook, GitHub PAT, and NOUS_API_KEY

## Objective
Create four SealedSecret resources in the sigma1-dev namespace for `linear-api-token`, `discord-webhook-url`, `github-pat`, and `nous-api-key`, ensuring each secret holds its respective API credential.

## Steps
1. For each of the four secrets (`linear-api-token`, `discord-webhook-url`, `github-pat`, `nous-api-key`), create a standard Kubernetes Secret manifest with the appropriate data key (e.g., `token`, `url`, `pat`, `api-key`).
2. Use `kubeseal` to encrypt each Secret manifest into a SealedSecret YAML file:
   - `sealed-secret-linear-api-token.yaml`
   - `sealed-secret-discord-webhook-url.yaml`
   - `sealed-secret-github-pat.yaml`
   - `sealed-secret-nous-api-key.yaml`
3. All SealedSecrets target namespace `sigma1-dev`.
4. Apply all four SealedSecret manifests: `kubectl apply -f sealed-secret-*.yaml -n sigma1-dev`.
5. Verify the sealed-secrets controller decrypts them into usable Secret objects.

## Validation
`kubectl get secret -n sigma1-dev` lists `linear-api-token`, `discord-webhook-url`, `github-pat`, and `nous-api-key`. Each secret has the expected data key present (non-empty) when described with `kubectl get secret <name> -n sigma1-dev -o jsonpath='{.data}'`.