# GPT-5.1 Codex Findings — Option Vote

## Inputs Reviewed
- `docs/template-structure-options.md`
- Conversation thread on Play vs Healer prompt sourcing, system prompts, and DRY goals

## Key Observations
1. **Play vs Healer divergence**: Workflows differ mainly in prompts/system framing and setup steps (docs service vs PVC). Keeping workflow containers readable is important for debugging each flow independently.
2. **CLI reuse**: All CLIs share identical invocation/config partials regardless of workflow, so isolating those under `templates/clis/` prevents duplication without complicating workflow logic.
3. **Agent scalability**: Single-file agent identities already exist and should remain workflow-agnostic; Options C and D both satisfy this, but D keeps render graphs simpler.
4. **Controller effort**: Option C’s matrix composition would require sizeable controller refactors to dynamically stitch every dimension. Option D reuses existing workflow render entry points and only swaps in partials, so it is lower lift and safer short term.
5. **Future agents/CLIs**: Option D still allows drop-in additions (new agent file, new CLI partial) while ensuring each workflow container remains self-contained for reviews, which matches the requested debugging ergonomics.

## Vote
**Option D (Hybrid)** — Provides the best balance of readability, low controller complexity, and ease of adding agents/CLIs while staying DRY where it matters (CLIs + shared utilities).

