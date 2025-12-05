# 🏆 Template Structure Consensus

**Date**: 2025-12-05  
**Decision**: **Option D (Hybrid)** ✅

---

## Vote Tally

| Model | Vote | Confidence |
|-------|------|------------|
| **Gemini 3 Pro** | Option D | Unanimous team consensus |
| **GPT-5.1 Codex** | Option D | High |
| **Claude Sonnet 4.5** | Option D | 85% (conditional C if scale >25 agents) |
| **Composer** | Option D | High |
| **Grok** | Option C | High |
| **Claude Opus** | Option C | 8/10 |

### Results
- **Option D**: 4 votes ✅
- **Option C**: 2 votes

---

## Consensus: Option D (Hybrid)

```
templates/
├── shared/                    # Common utilities (partials)
│   ├── git.sh.hbs
│   ├── rust-env.sh.hbs
│   ├── node-env.sh.hbs
│   └── mcp.json.hbs
│
├── agents/                    # Agent identities (single files)
│   ├── rex.md.hbs
│   ├── blaze.md.hbs
│   ├── bolt.md.hbs
│   └── ...
│
├── clis/                      # CLI configs + invoke partials
│   ├── claude/
│   │   ├── config.json.hbs
│   │   ├── settings.json.hbs
│   │   └── invoke.sh.hbs
│   ├── factory/
│   │   └── invoke.sh.hbs
│   └── codex/
│       └── invoke.sh.hbs
│
├── code/                      # Play workflow (complete container)
│   ├── system-prompt.hbs
│   └── container.sh.hbs      # Uses {{> clis/{cli}/invoke}}
│
└── healer/                    # Healer workflow (complete container)
    ├── system-prompt.hbs
    └── container.sh.hbs      # Uses {{> clis/{cli}/invoke}}
```

---

## Why Option D Won

### Key Arguments (Recurring Across Votes)

1. **Agent Scalability** ⭐⭐⭐⭐⭐
   - Adding a new agent = 1 file (`agents/nova.md.hbs`)
   - Zero controller changes
   - Works immediately across all workflows and CLIs

2. **Debugging & Operational Excellence**
   - Readable, self-contained workflow containers
   - Ops can understand ONE file during incidents
   - No need to trace through 5+ partial files

3. **Lower Migration Risk**
   - Closest to current structure
   - Minimal controller refactoring
   - Production-ready faster

4. **Balanced DRY**
   - CLI invocation logic shared via partials
   - Accept ~30% duplication between workflow containers
   - Trade-off: Simpler debugging > Maximum DRY

5. **Workflow Isolation**
   - Code and Healer can evolve independently
   - No complex `if/else` in shared master template
   - Clear ownership per workflow

---

## Dissenting Opinion (Option C)

**Grok** and **Claude Opus** voted for Option C (Matrix Composition) because:

- Maximum DRY (each concept defined exactly once)
- Perfect separation: WHO × HOW × WHAT
- Better long-term scalability (25+ agents, 5+ workflows)
- Composition logic is a one-time cost

**Counter-argument** (from majority):
> "Option C's composition complexity is not worth the theoretical DRY benefits at current scale. We can migrate to C later if needed."

---

## Migration Plan

### Phase 1: Create Structure
1. Create `clis/` with CLI invoke partials
2. Keep `agents/` as single-file identities
3. Refactor `code/container.sh.hbs` to use `{{> clis/{cli}/invoke}}`
4. Create `healer/container.sh.hbs` using same partials

### Phase 2: Controller Updates
1. Update template selection logic
2. Inject agent + CLI context to Handlebars
3. Test all agent × CLI × workflow combinations

### Phase 3: Future Review
- If agent count exceeds 25
- If workflow count exceeds 5
- Consider migrating to Option C

---

## Action Items

- [ ] Implement Option D structure
- [ ] Create CLI invoke partials (`clis/{cli}/invoke.sh.hbs`)
- [ ] Refactor workflow containers to use partials
- [ ] Update controller template composition
- [ ] Test full matrix (9 agents × 6 CLIs × 2 workflows)
- [ ] Document new structure in README

---

**Decision Finalized**: Option D (Hybrid)  
**Approved By**: Multi-agent consensus (4-2 vote)

