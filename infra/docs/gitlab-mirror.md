# GitHub → GitLab mirror (in-cluster)

The CronJob `gitlab-mirror` (namespace `gitlab`) pushes `main` and tags from
`https://github.com/5dlabs/cto.git` into the self-hosted GitLab project
`5dlabs/cto` every 5 minutes.

Why in-cluster: external GitHub Actions push hit Cloudflare's 100 MB request
limit on a full clone. SSH directly to `gitlab-shell` inside the cluster
bypasses Cloudflare entirely.

## Components

- Helm chart: `infra/charts/gitlab-mirror/`
- Argo Application: `infra/gitops/applications/platform/gitlab-mirror.yaml`
- Secret (out-of-band, **not** in Git): `gl-mirror-key` in `gitlab` ns
  - Key field: `id_ed25519` (private key)
  - Public key is registered as a deploy key on project `5dlabs/cto` with
    `can_push: true`
  - Backup: 1Password Operations vault, item id
    `ee6p5fqlidbn4lhld3eiz57234` ("GitLab Mirror Deploy Key (5dlabs/cto)")

## Recreate the secret

If the cluster is rebuilt:

```sh
op item get ee6p5fqlidbn4lhld3eiz57234 --vault Operations \
  --field "private key" --reveal > /tmp/id_ed25519
chmod 600 /tmp/id_ed25519

kubectl -n gitlab create secret generic gl-mirror-key \
  --from-file=id_ed25519=/tmp/id_ed25519
rm /tmp/id_ed25519
```

## Operating

- Trigger an out-of-band run:
  ```sh
  kubectl -n gitlab create job --from=cronjob/gitlab-mirror \
    gitlab-mirror-manual-$(date +%s)
  ```
- Tail latest run:
  ```sh
  kubectl -n gitlab logs -l app.kubernetes.io/name=gitlab-mirror --tail=200
  ```
- Pause: `kubectl -n gitlab patch cronjob gitlab-mirror -p '{"spec":{"suspend":true}}'`

## Branch protection

`main` on the GitLab side is protected with `allow_force_push: true` so the
mirror can rewrite history if GitHub force-pushes (rare but supported). If you
tighten this, the mirror will fail until you unprotect or relax push perms.

## Replaces

`.github/workflows/mirror-to-gitlab.yml` (deleted) — Cloudflare 100 MB limit
made it unusable for full clones.
