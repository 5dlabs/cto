# CTO Agent Inventory: Tools, Templates, and Skills

Inventory derived from:

- **Tools:** `infra/charts/cto/cto-config.json` (canonical per-agent `tools.remote`)
- **Skills:** `templates/skills/skill-mappings.yaml` (default, job-type, optional)
- **Templates:** `crates/controller` uses config tools; run-type can add MCP tool patterns for some agents

Job types in skill-mappings: `coder`, `healer`, `intake`, `quality`, `test`, `deploy`, `security`, `review`, `integration`.

---

## 1. Morgan (Intake & PRD)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7 (resolve_library_id, get_library_docs), firecrawl (scrape, crawl, map, search), openmemory (query, store, list, get, reinforce), repomix (pack_codebase, pack_remote_repository, grep_repomix_output, read_repomix_output) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, firecrawl, repomix, project-development, skill-authoring, ask-questions, brainstorming, writing-plans |
| **Skills (intake)** | prd-analysis, multi-agent-patterns, frontend-stack-selection, doc-coauthoring, deep-research |
| **Skills (healer)** | incident-response |
| **Skills (optional)** | deep-research, pdf, docx, xlsx, pptx (trigger-based) |
| **Templates** | Controller: Morgan intake/documentation gets mcp_tools_firecrawl_*, context7_*, openmemory_* |

**Note:** Morgan has no Octocode in cto-config; skills reference octocode (skill content only). No Perplexity/Tavily in config.

---

## 2. Rex (Rust)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, rust-patterns, mcp-development, rust-error-handling |
| **Skills (coder)** | tool-design, firecrawl, compound-engineering, verification-before-completion, systematic-debugging, subagent-driven-development, requesting-code-review, receiving-code-review |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |
| **Skills (optional)** | better-auth, memory-systems, deep-research, ralph-technique, cargo-fuzz, fuzzing-obstacles |

---

## 3. Grizz (Go)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, go-patterns, go-concurrency |
| **Skills (coder)** | tool-design, firecrawl, compound-engineering, verification-before-completion, systematic-debugging, subagent-driven-development, requesting-code-review, receiving-code-review |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |
| **Skills (optional)** | better-auth, memory-systems, deep-research, ralph-technique, go-code-review |

---

## 4. Nova (Node.js)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, effect-patterns, elysia, drizzle-queries |
| **Skills (coder)** | tool-design, better-auth, firecrawl, compound-engineering, verification-before-completion, systematic-debugging, subagent-driven-development, requesting-code-review, receiving-code-review |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |
| **Skills (optional)** | shadcn-stack, memory-systems, deep-research, ralph-technique, sentry-tracing |

---

## 5. Blaze (Frontend)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), shadcn_* (list_components, get_component, get_component_demo, get_component_metadata, list_blocks, get_block, get_directory_structure), ai_elements_* (get_ai_elements_components, get_ai_elements_component), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, shadcn-stack, anime-js, effect-frontend-patterns, frontend-excellence, frontend-design, react-best-practices, web-design-guidelines |
| **Skills (coder)** | tool-design, firecrawl, compound-engineering, verification-before-completion, systematic-debugging, subagent-driven-development, requesting-code-review, receiving-code-review |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |
| **Skills (optional)** | tanstack-stack, better-auth, memory-systems, deep-research, ralph-technique, remotion, three-js, sentry-tracing, playwright-testing |

---

## 6. Tap (Mobile)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, expo-patterns, frontend-design, web-design-guidelines, react-native-best-practices, expo-building-ui, expo-data-fetching |
| **Skills (coder)** | tool-design, better-auth-expo, firecrawl, compound-engineering, verification-before-completion, systematic-debugging, subagent-driven-development, requesting-code-review, receiving-code-review |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |
| **Skills (optional)** | expo-deployment, expo-upgrading, expo-api-routes, expo-dev-client, expo-cicd-workflows, expo-use-dom, expo-tailwind |

---

## 7. Spark (Desktop)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, electron-patterns, frontend-design, web-design-guidelines |
| **Skills (coder)** | tool-design, better-auth-electron, firecrawl, compound-engineering, verification-before-completion, systematic-debugging |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |

---

## 8. Cleo (Quality)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (get_pull_request, get_pull_request_files, get_pull_request_comments, add_pull_request_review_comment, create_pull_request_review, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, code-review, evaluation, advanced-evaluation, repomix, firecrawl, code-maturity, ask-questions |
| **Skills (quality)** | tool-design, systematic-debugging |
| **Skills (optional)** | shadcn-stack, tanstack-stack, better-auth, rust-patterns, go-patterns, effect-patterns, deep-research |

---

## 9. Cipher (Security)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (list_code_scanning_alerts, get_code_scanning_alert, list_secret_scanning_alerts, get_secret_scanning_alert, get_pull_request, create_pull_request_review) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, security-analysis, observability, semgrep, codeql, sarif-parsing, differential-review, audit-prep-assistant, entry-point-analyzer, variant-analysis |
| **Skills (security)** | tool-design, property-based-testing, coverage-analysis |
| **Skills (optional)** | better-auth, rust-patterns, go-patterns, effect-patterns, cargo-fuzz, fuzzing-obstacles |

---

## 10. Tess (Testing)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), github (get_pull_request, get_pull_request_files, create_pull_request_review, get_pull_request_status), kubernetes (listResources, getResource, describeResource, getPodsLogs) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, llm-docs, github-mcp, testing-strategies, evaluation, advanced-evaluation, kubernetes-mcp, observability, webapp-testing, test-driven-development, verification-before-completion, playwright-testing |
| **Skills (test)** | tool-design, systematic-debugging, property-based-testing |
| **Skills (optional)** | shadcn-stack, better-auth, rust-patterns, go-patterns, effect-patterns, ralph-technique, cargo-fuzz, fuzzing-obstacles |

---

## 11. Stitch (Code Review)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, **octocode** (githubSearchCode, githubSearchRepositories, githubViewRepoStructure, githubGetFileContent, githubSearchPullRequests, packageSearch), openmemory (full), github (get_pull_request, get_pull_request_files, get_pull_request_comments, add_pull_request_review_comment, create_pull_request_review, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, llm-docs, github-mcp, pr-review |
| **Skills (review)** | code-review |

**Note:** Stitch is the only agent in cto-config with Octocode tools.

---

## 12. Atlas (Merge Gate)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7 (no firecrawl), openmemory (full), github (get_pull_request, get_pull_request_files, merge_pull_request, update_pull_request_branch, get_pull_request_status, create_pull_request_review, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, llm-docs, github-mcp, git-integration, git-worktrees, repomix, finishing-branch |
| **Skills (integration)** | multi-agent-patterns, parallel-agents, verification-before-completion, executing-plans |
| **Skills (optional)** | multi-agent-patterns |

---

## 13. Bolt (DevOps / SRE)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | context7, firecrawl (full), openmemory (full), kubernetes (listResources, getResource, createResource, updateResource, deleteResource, describeResource, getPodsLogs), github (create_pull_request, push_files, create_branch, get_file_contents) |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, kubernetes-mcp, observability, kubernetes-operators, argocd-gitops, secrets-management, storage-operators, mcp-development, mcp-builder |
| **Skills (deploy)** | tool-design, compound-engineering, verification-before-completion |
| **Skills (healer)** | incident-response |
| **Skills (optional)** | ai-ml-infra, cloudflare-workers, cloudflare-durable-objects, cloudflare-mcp-server, cloudflare-agents-sdk, cloudflare-wrangler |
| **Templates** | Controller: Bolt gets mcp_tools_kubernetes_*, github_*, context7_* |

---

## 14. Vex (VR – in skill-mappings only)

| Source | Contents |
|--------|----------|
| **Tools (cto-config)** | Not present in cto-config.json (omitted from inventory file). |
| **Skills (default)** | context-fundamentals, context-degradation, context-optimization, openmemory, context7, octocode, llm-docs, github-mcp, unity-vr, openxr |
| **Skills (coder)** | tool-design, firecrawl, compound-engineering, verification-before-completion, systematic-debugging |
| **Skills (healer)** | incident-response, observability, kubernetes-mcp |
| **Skills (optional)** | meta-xr, three-js |

---

## Tool name → display name (for UI)

| Prefix / Tool | Display |
|---------------|---------|
| context7_* | Context7 |
| firecrawl_* | Firecrawl |
| openmemory_* | OpenMemory |
| repomix_* | Repomix |
| octocode_* | Octocode |
| github_* | GitHub |
| kubernetes_* | Kubernetes |
| shadcn_* | shadcn/ui |
| ai_elements_* | AI Elements |

---

## Gaps / notes

- **Morgan:** Octocode is in skills but not in cto-config tools. Perplexity/Tavily not in config.
- **Stitch:** Only agent with Octocode in cto-config.
- **Atlas:** No Firecrawl in cto-config.
- **Vex:** In skill-mappings only; no entry in cto-config.json.
- **Controller templates:** Tools are resolved from cto-config per agent; run-type (intake, coder, healer, etc.) can add MCP tool patterns in code but do not add new tool names beyond what config exposes.
