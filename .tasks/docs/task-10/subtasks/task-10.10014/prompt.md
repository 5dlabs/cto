Implement subtask 10014: Create GitHub Actions pr-review.yml workflow with Stitch code review integration

## Objective
Create .github/workflows/pr-review.yml that triggers on pull_request events (opened, synchronize). The workflow calls Stitch to perform code review using github_get_pull_request and github_get_pull_request_files, then posts the review as a PR comment.

## Steps
Create .github/workflows/pr-review.yml: `on: pull_request: types: [opened, synchronize]`. Job `stitch-review`: uses stitch-action or curl to call Stitch API with PR number and list of changed files from `github_get_pull_request_files`. Stitch posts back a review comment via `github_create_pull_request_comment`. Store STITCH_API_TOKEN in GitHub Actions secrets. The workflow should not block merge — it is advisory only. Example step: `- uses: actions/github-script@v7` calling octokit to get PR files, then POST to Stitch review endpoint with file contents.

## Validation
Open a test PR — within 2 minutes a bot comment appears on the PR from Stitch with code review content. Check GitHub Actions run log shows stitch-review job completed with exit 0. Verify the comment references at least one changed file by name.