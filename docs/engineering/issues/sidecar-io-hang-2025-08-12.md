### Issue: Docs job hang potentially related to sidecar input endpoint changes

- **Date observed**: 2025-08-12
- **Environment**: agent-platform (ArgoCD/Argo Workflows)
- **Affected components**: sidecar input endpoints, docs job (Claude run), code runner workflow



#### Summary
After implementing the sidecar with input endpoints and changing the behavior to only open the file connection on demand, the docs job appears to hang and not exit cleanly even when work is complete. This behavior seems correlated with the recent change.

#### Current stuck workload
- **Pod**: `code-agent-platform-coderun-2-agent-docs-gc7zc-cd9a11ad-t26jv4k`
- **Observed state**: job hung at the last log line for an extended period; Claude did not exit.

#### Recent changes of interest


- Sidecar introduced with input endpoints


- Change to open the file connection on demand (previously open earlier in flow)



#### Symptoms


- Docs job reaches terminal-looking log line but process does not exit


- Requires manual intervention/timeout

#### Initial hypotheses (to validate; no fixes applied)
- File/FIFO lifecycle: reader/writer sequence may leave one end open, preventing EOF and process exit


- Sidecar input open-on-demand timing races with Claude process startup or shutdown


- Hanging file descriptors in the docs job container (e.g., FIFO or pipe remains open in background)


- Interaction with MCP configuration changes (e.g., `MCP_CLIENT_CONFIG`) indirectly affecting startup/teardown order

#### Data points to collect (next investigation steps)


- Container logs (stdout/stderr) around end-of-run for the stuck pod


- Process list and open file descriptors inside the pod when hung (`lsof`, `/proc/*/fd`)


- Confirmation of FIFO/pipe creation and close order in the docs job script vs. sidecar


- Timing of when the input connection is opened/closed relative to Claude start/exit


- Argo step termination behavior and any post-step hooks that could keep FDs open

#### Open questions


- Is the hang reproducible across runs, or intermittent?


- Does reverting to the previous (always-open) connection behavior eliminate the hang?


- Is the issue isolated to docs jobs, or also present in code tasks?



#### Status


- Cataloged. No code changes applied pending deeper investigation.

