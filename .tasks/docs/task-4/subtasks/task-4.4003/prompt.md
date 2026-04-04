Implement subtask 4003: Implement GitHub branch creation and file commit logic

## Objective
Implement the GitHub API calls to create a new branch pipeline/<session-id> from main and commit the generated scaffold and deliberation files to it.

## Steps
1. Create `src/design-snapshot/github-ops.ts` with functions for Git operations.
2. Implement `createPipelineBranch(sessionId: string)`: call `GET /repos/{owner}/{repo}/git/ref/heads/main` to get the latest commit SHA, then `POST /repos/{owner}/{repo}/git/refs` to create `refs/heads/pipeline/<session-id>` pointing to that SHA.
3. Implement `commitFiles(branchRef: string, files: Array<{ path: string, content: string }>)`: use the Git Trees API to create a tree with all files as blobs, create a commit pointing to that tree with parent as the branch tip, and update the branch ref to the new commit. This creates a single atomic commit.
4. Encode file content as base64 for the blob creation API.
5. Handle the case where the branch already exists (e.g., pipeline re-run) by deleting and recreating or force-updating the ref.

## Validation
Unit test with mocked GitHub API: verify createPipelineBranch makes the correct API calls (GET ref, POST ref) with proper owner/repo/branch naming. Verify commitFiles creates blobs, a tree, a commit, and updates the ref in the correct sequence. Verify file content is base64 encoded in blob creation calls.