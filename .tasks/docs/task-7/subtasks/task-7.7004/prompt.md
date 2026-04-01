Implement subtask 7004: Implement Git tree API commit flow (blobs → tree → commit → update ref)

## Objective
Implement the function that takes generated file contents and commits them to the pipeline branch in a single commit using GitHub's low-level Git Data API: creating blobs, assembling a tree, creating a commit object, and updating the branch ref.

## Steps
1. Create `src/services/git-committer.ts`.
2. Define a `FileEntry` type: `{ path: string, content: string }`.
3. Implement `commitFiles(client: GitHubClient, opts: { baseSha: string, branchRef: string, files: FileEntry[], message: string }): Promise<string>` returning the new commit SHA.
4. Step 1 — Create blobs: For each file, `POST /repos/5dlabs/sigma-1/git/blobs` with `{ content: base64(file.content), encoding: 'base64' }`. Collect blob SHAs. These can be done concurrently with `Promise.all` for performance.
5. Step 2 — Create tree: `POST /repos/5dlabs/sigma-1/git/trees` with `{ base_tree: baseSha, tree: [{ path, mode: '100644', type: 'blob', sha: blobSha }] }`. Get tree SHA.
6. Step 3 — Create commit: `POST /repos/5dlabs/sigma-1/git/commits` with `{ message, tree: treeSha, parents: [baseSha] }`. Get commit SHA.
7. Step 4 — Update ref: `PATCH /repos/5dlabs/sigma-1/git/refs/heads/pipeline/{runId}` with `{ sha: commitSha }`.
8. Return the commit SHA for traceability.

## Validation
Unit test: mock all 4 API call types; verify blob creation is called once per file with base64-encoded content. Unit test: verify tree creation payload includes all blob SHAs with correct paths and mode '100644'. Unit test: verify commit creation references the tree SHA and baseSha as parent. Unit test: verify ref update is called with the commit SHA. Unit test: with 5 files, verify exactly 5 blob calls, 1 tree call, 1 commit call, 1 ref update call.