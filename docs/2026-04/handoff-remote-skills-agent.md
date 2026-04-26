# Handoff: remote skills / cto-skills (Metaprompt-style)

Prompt template for the next agent. Fill in the `{$VARIABLE}` placeholders (or replace XML blocks) before running.

---

## Inputs

Minimal variables the instructions reference:

- `{$WORKTREE_PATH}`
- `{$GIT_BRANCH}`
- `{$PRIMARY_GOAL}`
- `{$COMPLETED_WORK_SUMMARY}`
- `{$OUT_OF_SCOPE_OR_REVERTS}`
- `{$TEST_ENV}`
- `{$SUCCESS_CRITERIA}`

---

## Instructions structure

1. Open with role and constraints (read-only vs allowed edits, branch/worktree).
2. Inline **completed work** and **explicit reversions** so the agent does not redo or fight them.
3. State the **remaining goal** as an ordered checklist.
4. Embed **test environment** and **commands** as evidence steps, not guesses.
5. Close with **success criteria** and **what to report back** (artifacts, logs, URLs).

---

## Instructions

You are an autonomous coding agent continuing work on the CTO platform. A prior session implemented (or partially implemented) **remote skills distribution**: a separate `cto-skills` repo, GitHub Releases with `hashes.txt` and per-agent tarballs named `{agent}-default.tar.gz` and `{agent}-{project}.tar.gz`, controller-side cache concepts, and optional `spec.skillsUrl` / `spec.skillsProject` on `CodeRun`.

**Authoritative context from the human**

- **Worktree / branch to treat as the prior working line (unless the human says otherwise):**  
  Path: `{$WORKTREE_PATH}`  
  Branch: `{$GIT_BRANCH}`

- **Primary goal for this pass:**  
  `{$PRIMARY_GOAL}`

- **What was already done (do not contradict; build on it):**  

  <completed_work>  
  {$COMPLETED_WORK_SUMMARY}  
  </completed_work>

- **Important: the human may have reverted or removed parts of this work in the main tree.** Read and respect:  

  <reverts_or_scope_limits>  
  {$OUT_OF_SCOPE_OR_REVERTS}  
  </reverts_or_scope_limits>  

  Do not reintroduce CRD fields, Helm PVCs, producer JSON keys, or controller modules the human removed unless they explicitly ask.

- **Where to run tests:**  

  <test_environment>  
  {$TEST_ENV}  
  </test_environment>

**Your workflow**

1. **Orient:** From `{$WORKTREE_PATH}` and `{$GIT_BRANCH}`, list what is actually present (files, diffs vs `main`). Do not assume prior chat transcripts are current.
2. **Align with reverts:** If the human removed `skillsUrl` / `skillsProject` from the CRD and stripped Helm/MCP/PM wiring, treat **remote skills integration in the CTO repo** as **out of scope** until asked—focus on **cto-skills repo CI**, release assets, tarball naming, or validation that does not require those fields.
3. **Execute the goal:** Work through `{$PRIMARY_GOAL}` as a numbered checklist. Prefer small, verifiable steps; capture command output and URLs (GitHub Actions run, release assets, sample `hashes.txt` lines).
4. **Verify:** Run or describe the minimal checks that prove correctness for this pass (e.g. download one tarball, confirm layout under `agent/skill/`, confirm `hashes.txt` lists expected assets).
5. **Report:** Summarize what you did, what is still open, and blockers. Attach paths, commit SHAs, and links.

**Success criteria (must all be satisfied or explicitly flagged)**

{$SUCCESS_CRITERIA}

**Output format**

- Start with a short **plan** (bullet list).
- Then **evidence** (commands + relevant output snippets, no secrets).
- End with **done / not done** vs the success criteria.

---

## Example fills (optional)

Replace the variables above with concrete values, for example:

| Variable | Example value |
|----------|-----------------|
| `{$WORKTREE_PATH}` | `/Users/jonathon/5dlabs/cto/.claude/worktrees/reverent-kalam` |
| `{$GIT_BRANCH}` | `claude/reverent-kalam` |
| `{$PRIMARY_GOAL}` | Validate `5dlabs/cto-skills` latest release: `rex-default` vs `rex-test-sandbox` skill counts; document how to enable remote skills in-cluster once CRD wiring is restored. |
| `{$COMPLETED_WORK_SUMMARY}` | Per-agent tarballs; merge of `_default` + project overlays; CI on `main` publishing `hashes.txt` and assets. |
| `{$OUT_OF_SCOPE_OR_REVERTS}` | User reverted `skillsUrl` / `skillsProject`, Helm `skillsCache`, `skills_cache` module, and PM/MCP stamping in main repo—do not re-add without explicit approval. |
| `{$TEST_ENV}` | Local `kubectl` + namespace, or `curl`/`gh` against public GitHub Releases only. |
| `{$SUCCESS_CRITERIA}` | `hashes.txt` lists both `rex-default.tar.gz` and `rex-test-sandbox.tar.gz`; tar contents show expected skill folders; short written test protocol for the next engineer. |

---

## Note on convention

This file follows the **Metaprompt** pattern (see Anthropic’s Prompt Generator notebook): **Inputs** name the variables, long content lives in **XML-tagged blocks**, and **Instructions** tell another model exactly how to behave and how to report results.
