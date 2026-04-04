Implement subtask 10017: CI/CD: GitHub Actions merge-to-main workflow (image build, push, manifest update)

## Objective
Create a GitHub Actions workflow triggered on merge to main that builds and pushes container images to the registry and updates ArgoCD application manifests with new image tags.

## Steps
Step-by-step:
1. Create `.github/workflows/deploy.yaml` triggered on `push` to `main`.
2. Define jobs:
   a. **build-and-push** (matrix strategy for each service):
      - Checkout code
      - Login to container registry (ghcr.io: `docker/login-action` with `GITHUB_TOKEN`)
      - Build Docker image for the service using its Dockerfile
      - Tag with both `latest` and the commit SHA (e.g., `ghcr.io/org/equipment-catalog:sha-abc1234`)
      - Push both tags
   b. **update-manifests** (depends on build-and-push):
      - Checkout the GitOps manifests repository (or manifests directory in the same repo)
      - For each service, update the image tag in the Deployment manifest (or Helm values) to the new SHA tag
      - Commit and push the manifest changes
      - ArgoCD will detect the change and sync automatically
3. Use a dedicated deploy key or GitHub App token for manifest repo writes.
4. Add build caching (Docker layer cache) to speed up builds.

## Validation
Merge a PR to main. Verify GitHub Actions workflow runs, images are pushed to the registry (check ghcr.io packages page), and manifest files are updated with the new image tag. Verify ArgoCD detects the change within 3 minutes.