---
name: doc-coauthoring
description: Structured workflow for co-authoring documentation, PRDs, technical specs, and decision docs. Use when writing substantial documentation, creating proposals, drafting specifications, or any structured content that benefits from iterative refinement. Provides context gathering, section-by-section building, and reader testing.
---

# Document Co-Authoring Workflow

Guide users through collaborative document creation using three stages: Context Gathering, Refinement & Structure, and Reader Testing.

## When to Offer This Workflow

**Trigger conditions:**
- Writing documentation, proposals, specs, decision docs, RFCs
- Substantial writing tasks (not quick notes)
- Content that will be read by others

**Offer the workflow upfront:** Explain the three stages and ask if they want this structured approach or prefer freeform.

## Stage 1: Context Gathering

**Goal:** Close the gap between what the user knows and what you know.

### Initial Questions

Ask for meta-context (user can answer in shorthand):

1. What type of document is this? (tech spec, decision doc, proposal)
2. Who's the primary audience?
3. What's the desired impact when someone reads this?
4. Is there a template or format to follow?
5. Any other constraints or context?

### Info Dumping

Encourage the user to dump all context they have:
- Background on the project/problem
- Related discussions or documents
- Why alternatives aren't being used
- Organizational context
- Timeline pressures
- Technical architecture

**Tell them:** "Don't worry about organizing it - just get it all out."

### Clarifying Questions

After initial dump, generate 5-10 numbered questions based on gaps:

```
1. What's the timeline for this decision?
2. Who are the key stakeholders who need to approve?
3. What happened when you tried approach X?
...
```

User can answer in shorthand: "1: end of Q1, 2: eng leads + PM, 3: see #channel-name"

**Exit condition:** Questions show understanding - you can ask about edge cases and trade-offs without needing basics explained.

## Stage 2: Refinement & Structure

**Goal:** Build the document section by section through brainstorming, curation, and iterative refinement.

### Process for Each Section

1. **Clarifying Questions** - Ask 5-10 questions about what should be included
2. **Brainstorming** - Generate 5-20 numbered options based on section complexity
3. **Curation** - User indicates what to keep/remove/combine:
   - "Keep 1,4,7,9"
   - "Remove 3 (duplicates 1)"
   - "Combine 11 and 12"
4. **Gap Check** - Ask if anything important is missing
5. **Drafting** - Write the section based on selections
6. **Iteration** - Refine through surgical edits until satisfied

### Section Ordering

Start with whichever section has the most unknowns:
- Decision docs: Usually the core proposal
- Specs: Usually the technical approach
- PRDs: Usually the problem statement

**Leave summary sections for last.**

### Key Instruction for Users

Instead of editing the doc directly, have them indicate what to change:
- "Remove the X bullet - already covered by Y"
- "Make the third paragraph more concise"
- "Move section 3 before section 2"

This helps learn their style for future sections.

### Quality Checking

After 3 consecutive iterations with no substantial changes, ask:
> "Can anything be removed without losing important information?"

## Stage 3: Reader Testing

**Goal:** Test the document with a fresh perspective to catch blind spots.

### Step 1: Predict Reader Questions

Generate 5-10 questions readers might ask when discovering this document:
- What would they type into search?
- What would they ask Claude.ai?

### Step 2: Test with Fresh Context

**If sub-agents available:**
Invoke a sub-agent with just the document content and each question. Summarize what it got right/wrong.

**If no sub-agents:**
Have user open fresh Claude conversation, paste document, ask the predicted questions. Report back what Reader Claude struggled with.

### Step 3: Additional Checks

Ask (or have Reader Claude check):
- "What in this doc might be ambiguous to readers?"
- "What knowledge does this doc assume readers already have?"
- "Are there internal contradictions or inconsistencies?"

### Step 4: Fix Blind Spots

For each issue found, loop back to Stage 2 refinement for that section.

**Exit condition:** Reader Claude consistently answers questions correctly and doesn't surface new gaps.

## Final Review

When Reader Testing passes:

1. Recommend they do a final read-through themselves
2. Suggest double-checking facts, links, technical details
3. Ask them to verify it achieves the intended impact

**Final tips:**
- Consider linking this conversation in an appendix
- Use appendices for depth without bloating main doc
- Update as feedback comes from real readers

## Handling Deviations

| Situation | Response |
|-----------|----------|
| User wants to skip a stage | Ask if they want to skip and write freeform |
| User seems frustrated | Acknowledge time investment, suggest faster path |
| Missing context on something mentioned | Ask proactively, don't let gaps accumulate |
| User edits doc directly | Note changes, incorporate preferences for future sections |

## Tips for Effectiveness

- Be direct and procedural
- Explain rationale briefly when it affects behavior
- Don't try to "sell" the approach - just execute it
- Give user agency to adjust the process
- Quality over speed - each iteration should make meaningful improvements
