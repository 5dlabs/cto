# Template Structure Options - Composer Vote

**Model:** Composer (Cursor AI)  
**Date:** 2025-01-27  
**Vote:** **Option D (Hybrid)** - RECOMMENDED

---

## Executive Summary

After reviewing the template structure options document and analyzing the current controller implementation (`crates/controller/src/tasks/code/templates.rs`), I recommend **Option D (Hybrid)** as the best balance between DRY principles, maintainability, operational excellence, and scalability.

---

## Analysis of Current State

### Current Controller Implementation

The controller currently uses:
- Hardcoded template paths for each agent × CLI combination
- Separate functions per CLI (`get_agent_container_template`, `get_codex_container_template`, etc.)
- Significant duplication: Same agent logic repeated across CLIs
- Adding a new agent requires updating multiple match statements across CLI-specific functions

**Example from current code (lines 3007-3075):**
```rust
match github_app {
    "5DLabs-Rex" => "claude/container-rex.sh.hbs",
    "5DLabs-Blaze" => "claude/container-blaze.sh.hbs",
    // ... repeated for codex, factory, etc.
}
```

### Current Pain Points

1. **Agent Addition Complexity**: Adding a new agent (e.g., Nova) requires:
   - Creating templates in multiple CLI directories (`code/claude/container-nova.sh.hbs`, `code/factory/container-nova.sh.hbs`, etc.)
   - Updating multiple match statements in controller
   - Testing across all CLI combinations

2. **CLI Addition Complexity**: Adding a new CLI requires:
   - Creating agent-specific templates for each agent
   - Duplicating container logic across agents
   - Updating controller with new CLI-specific functions

3. **Debugging Difficulty**: When a container fails, ops must:
   - Identify which CLI-specific template was used
   - Read potentially multiple files to understand the full script
   - Trace through CLI-specific logic

---

## Option Comparison

### Option A: Workflow-First
**Verdict:** ❌ Not Recommended

**Pros:**
- Clear workflow separation
- Easy to understand file organization

**Cons:**
- CLI container logic duplicated across workflows
- Config files duplicated (e.g., `settings.json` in both `code/` and `healer/`)
- Adding new CLI requires updating multiple workflow directories
- **Agent scalability:** ⭐⭐⭐⭐⭐ (single-file agents)

### Option B: CLI-First with Workflow Prompts
**Verdict:** ❌ Not Recommended

**Pros:**
- CLI logic in one place
- Less duplication of CLI configs

**Cons:**
- **Agent scalability:** ⭐⭐⭐ (requires 3 files per agent: identity + code + healer)
- More complex agent directory structure
- Higher chance of forgetting workflow-specific variants
- Requires composition logic in controller

### Option C: Matrix Composition (Most DRY)
**Verdict:** ⚠️ Second Choice

**Pros:**
- Maximum DRY - each concept defined once
- Clear separation: WHO (agent) × HOW (CLI) × WHAT (workflow)
- Adding new CLI = add one folder
- Adding new workflow = add one folder
- **Agent scalability:** ⭐⭐⭐⭐⭐ (single-file agents)

**Cons:**
- Most complex composition logic required
- Harder to understand full container without reading multiple files
- Debugging requires tracing through composition
- Requires a master `container.sh.hbs` abstraction layer
- More complex migration path

### Option D: Hybrid (Balanced)
**Verdict:** ✅ **RECOMMENDED**

**Pros:**
- Workflows have complete, readable containers
- CLI-specific logic shared via small partials
- **Agent scalability:** ⭐⭐⭐⭐⭐ (single-file agents)
- Easier to debug (can read one container file)
- Less complex composition than Option C
- Closest to current structure (easiest migration)
- Production-ready and maintainable

**Cons:**
- Some duplication between `code/container.sh.hbs` and `healer/container.sh.hbs`
- Need to maintain partials in sync

---

## Why Option D Wins

### 1. **Controller Migration Path is Cleanest**

Current controller already renders workflow-specific containers:
- `code/container.sh.hbs`
- `heal/container.sh.hbs`

Option D keeps this pattern - minimal refactoring needed:
- Change from `format!("code/{template_name}")` to `format!("code/container.sh.hbs")` with Handlebars partials
- No new abstraction layer required

**Migration Example:**
```rust
// Current (hardcoded per agent × CLI)
match github_app {
    "5DLabs-Rex" => "claude/container-rex.sh.hbs",
    // ...
}

// Option D (one template per workflow, agent injected via context)
let template = match run_type {
    "code" => "code/container.sh.hbs",  // Uses {{> agents/{agent} }} + {{> clis/{cli}/invoke }}
    "heal" => "healer/container.sh.hbs",
};
```

### 2. **Agent Scalability is Optimal**

Adding a new agent (e.g., Nova):
```bash
# Just create ONE file:
templates/agents/nova.md.hbs
```

**Controller changes:** None - agent identity is injected via Handlebars context  
**Works immediately:** All workflows and CLIs automatically support Nova

### 3. **Debugging & Operational Excellence**

When a container fails:
- ✅ Ops can read ONE file (`code/container.sh.hbs`) to understand the full script
- ✅ No need to trace through 5+ partial files
- ✅ Easier to add logging/debugging statements in one place
- ✅ Faster incident response

### 4. **CLI Scalability**

Adding a new CLI (e.g., Gemini):
- Create `clis/gemini/invoke.sh.hbs` + config files
- Controller change: Add CLI to enum, update one match statement
- **All agents automatically work with new CLI**

### 5. **Workflow Scalability**

Adding a new workflow (e.g., Security Scan):
- Create `security/container.sh.hbs` + `system-prompt.hbs`
- Controller change: Add workflow type to enum, one new template render path
- **All agents and CLIs automatically work with new workflow**

### 6. **Comparison to Option C**

| Aspect | Option C | Option D |
|--------|----------|----------|
| DRY Score | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Readability | ⭐⭐ | ⭐⭐⭐⭐ |
| Debugging | ⭐⭐ | ⭐⭐⭐⭐ |
| Migration Risk | ⭐⭐ | ⭐⭐⭐⭐ |
| Controller Complexity | ⭐⭐ | ⭐⭐⭐⭐ |

**Option D trades a small amount of DRY for significant operational benefits.**

---

## Migration Effort Analysis

### Option D Migration Steps

1. **Create CLI partials** (`clis/{cli}/invoke.sh.hbs`)
   - Extract CLI invocation logic from existing containers
   - Low risk: Small, focused files

2. **Refactor existing containers** (`code/container.sh.hbs`, `healer/container.sh.hbs`)
   - Add Handlebars partial includes: `{{> clis/{cli}/invoke }}`
   - Add agent identity include: `{{> agents/{agent} }}`
   - Medium risk: But containers remain readable

3. **Update controller composition**
   - Change template selection logic
   - Inject agent and CLI context
   - Low risk: Minimal changes to existing logic

4. **Test all combinations**
   - Agent × CLI × Workflow matrix
   - Same effort for all options

### Option C Migration Steps

1. **Create master container template** (`container.sh.hbs`)
   - NEW abstraction layer
   - High risk: Complex composition logic

2. **Create core/ directory** with universal primitives
   - Medium risk: Need to identify what's truly universal

3. **Refactor to matrix composition**
   - High risk: Significant controller changes
   - Harder to debug failures

**Conclusion:** Option D has lower migration risk and is closer to current structure.

---

## Real-World Example: Adding Nova Agent

### Current Approach (Before Refactor)
```bash
# Create templates in multiple CLI directories
templates/code/claude/container-nova.sh.hbs
templates/code/factory/container-nova.sh.hbs
templates/code/codex/container-nova.sh.hbs
# ... repeat for each CLI

# Update controller match statements
match github_app {
    "5DLabs-Nova" => "claude/container-nova.sh.hbs",  # In get_agent_container_template
    // ... repeat in get_codex_container_template, get_factory_container_template, etc.
}
```

**Files to create:** ~6-8 files  
**Controller changes:** ~3-4 functions  
**Testing:** All CLI combinations

### Option D Approach
```bash
# Create ONE agent file
templates/agents/nova.md.hbs

# That's it!
```

**Files to create:** 1 file  
**Controller changes:** None  
**Testing:** Automatic - works for all CLIs and workflows

---

## Decision Matrix Summary

| Criteria | Option A | Option B | Option C | Option D |
|----------|----------|----------|----------|----------|
| DRY (less duplication) | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Readability | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Controller complexity | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Adding new CLI | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Adding new workflow | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Adding new agent** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Debugging ease | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Migration risk | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |

**Legend:** ⭐ = Poor, ⭐⭐⭐⭐⭐ = Excellent

---

## Final Recommendation

**Vote: Option D (Hybrid)**

### Key Reasons:

1. ✅ **Best for agent scalability**: 1 file per agent, zero controller changes
2. ✅ **Best for debugging**: Readable container files that ops can understand
3. ✅ **Best migration path**: Closest to current structure, lowest risk
4. ✅ **Balanced DRY**: Shared CLI logic without over-abstraction
5. ✅ **Production-ready**: Easier to maintain and operate

### Trade-off Accepted:

Some duplication between `code/container.sh.hbs` and `healer/container.sh.hbs` is acceptable because:
- Workflows have fundamentally different setup (docs service vs PVC files)
- The duplication is minimal (just workflow-specific setup)
- The operational benefits outweigh the minor duplication

---

## Next Steps

1. **Approve Option D** as the template structure
2. **Create migration plan** with specific tasks:
   - Extract CLI invocation logic to `clis/{cli}/invoke.sh.hbs`
   - Refactor `code/container.sh.hbs` to use partials
   - Create `healer/container.sh.hbs` using same partials
   - Update controller template selection logic
3. **Test migration** with one agent × CLI × workflow combination
4. **Roll out** to all combinations

---

**Signed:** Composer (Cursor AI)  
**Date:** 2025-01-27

