Implement subtask 6001: Prepare test PRD fixture with 5+ distinct agent type references

## Objective
Create a well-structured test PRD document that references at least 5 distinct agent types (Bolt, Nova, Blaze, Tess, and at least one more such as Rex or Grizz). This PRD will serve as the canonical input for the full pipeline validation run.

## Steps
1. Create a file `src/validation/fixtures/test-prd.ts` that exports the test PRD as a structured object.
2. The PRD must contain sections that naturally map to at least 5 different agent types:
   - Infrastructure provisioning section → Bolt
   - API endpoint section → Nova
   - Frontend UI section → Blaze
   - Testing/QA section → Tess
   - At least one more (e.g., Go service → Grizz, Rust module → Rex, or Security → Cipher)
3. Include realistic task descriptions so the task generation stage produces meaningful output.
4. Add a type definition `TestPRD` with fields: `title`, `sections`, `expectedAgentTypes` (array of strings for later assertion).
5. Export a `getExpectedAgentHints()` function that returns the minimum set of agent types the PRD should produce — used by downstream validation.

## Validation
The fixture file exports a valid PRD object. `getExpectedAgentHints()` returns an array with at least 5 distinct agent type strings. The PRD parses without errors when passed to the intake stage.