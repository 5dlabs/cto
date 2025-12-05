# 🟢 The Huddle: Template Architecture Review

**Model**: gemini-3-pro-preview
**Date**: 2025-12-05
**Topic**: Template Structure Restructuring (DRY vs. Scalability)

I've gathered the specialized agents to review `docs/template-structure-options.md` and the requirements. Here is the transcript of their review and voting session.

---

### 🗣️ Agent Discussion

**🦖 Rex (Rust/Backend Specialist)**
> "I've analyzed the compilation path for these templates. **Option A** is a maintenance nightmare—drift between `code/` and `healer/` containers is inevitable. **Option C** (Matrix) appeals to my love for abstraction, but debugging a script composed of 7 fragments at runtime is painful when `kubectl apply` fails.
>
> I prefer **Option D**. It gives me a concrete `container.sh` I can actually read and lint, while still importing the CLI logic. It keeps the 'unsafe' parts (bash scripts) readable."

**🔥 Blaze (Frontend/DX Specialist)**
> "I'm looking at the developer experience for adding new agents.
> **Option B is rejected immediately.** I don't want to create 3 files just to add 'Nova'.
> **Option C** is cool but feels over-engineered.
> **Option D** feels right. I create `agents/nova.md.hbs` and I'm done. If I need to debug why my node environment isn't setting up, I just look at `code/container.sh.hbs`. Simple."

**🔩 Bolt (Infrastructure/Ops Specialist)**
> "From an operational stability standpoint:
> - **Option A**: High risk of configuration drift.
> - **Option B**: Too much filesystem overhead.
> - **Option C**: Hard to audit. If a script fails in production, tracing it back to `clis/claude/invoke.sh.hbs` vs `core/git-setup.sh.hbs` takes too long during an incident.
> - **Option D**: Best compromise. The 'How to run' (CLI) is shared, but the 'What to run' (Workflow) is explicit. I can patch the `code` workflow without risking breaking the `healer` workflow."

**📋 Morgan (Project Manager)**
> "My priority is scalability. We need to add **Nova (AI)**, **Tess (Testing)**, and **Spark (Research)** soon.
> Both **C** and **D** allow 'Single File Agent' creation, which reduces onboarding time. I'll defer to engineering on debugging preference."

---

### 🗳️ The Vote

| Agent | Vote | Rationale |
| :--- | :--- | :--- |
| **Rex** | **Option D** | Type safety of explicit containers + DRY CLI partials. |
| **Blaze** | **Option D** | Best DX. Easy to add agents, easy to read scripts. |
| **Bolt** | **Option D** | Operational stability. Isolates workflow logic while sharing CLI configs. |
| **Morgan**| **Option D** | Scalable. Meets the "Single File Agent" requirement. |

### 🏆 Consensus: Option D (Hybrid)

**The team has unanimously selected Option D.**

**Why it won:**
1.  **Single-File Agents:** Adding a new agent only requires creating `templates/agents/name.md.hbs`.
2.  **Readable Containers:** `code/container.sh.hbs` exists as a cohesive file, making debugging easier than the "Matrix" approach.
3.  **DRY CLIs:** The complex CLI invocation logic (flags, output parsing) is extracted into partials, preventing duplication errors.
4.  **Workflow Isolation:** Healer and Play workflows can evolve independently (e.g., Healer needs worktrees, Play needs feature branches) without complex `if/else` logic in a shared master template.

---

### Implementation Plan

Proceed with **Option D** implementation:

1.  **Create `clis/` partials:** Extract just the execution logic (e.g., `claude ...`) into `clis/claude/invoke.sh.hbs`.
2.  **Create Workflow Containers:**
    *   Refactor `code/container.sh.hbs` to use `{{> clis/claude/invoke}}`.
    *   Create `healer/container.sh.hbs` using `{{> clis/factory/invoke}}` + Healer-specific git logic.
3.  **Verify Agents:** Ensure `agents/rex.md.hbs` handles identity for *both* workflows (already designed this way).

