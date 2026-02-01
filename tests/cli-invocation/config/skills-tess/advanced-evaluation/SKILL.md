---
name: advanced-evaluation
description: LLM-as-Judge techniques including direct scoring, pairwise comparison, rubric generation, and bias mitigation.
agents: [cleo, tess, morgan, atlas]
triggers: [LLM-as-judge, compare outputs, evaluation rubrics, mitigate bias, direct scoring, pairwise comparison]
---

# Advanced Evaluation

Production-grade techniques for evaluating LLM outputs using LLMs as judges.

## Evaluation Taxonomy

### Direct Scoring
Single LLM rates one response on a defined scale.

- **Best for:** Objective criteria (factual accuracy, instruction following)
- **Reliability:** Moderate to high for well-defined criteria
- **Failure mode:** Score calibration drift

### Pairwise Comparison
LLM compares two responses and selects the better one.

- **Best for:** Subjective preferences (tone, style, persuasiveness)
- **Reliability:** Higher than direct scoring for preferences
- **Failure mode:** Position bias, length bias

## The Bias Landscape

| Bias | Description | Mitigation |
|------|-------------|------------|
| Position | First-position responses favored | Swap positions, majority vote |
| Length | Longer = higher rating | Explicit prompting to ignore length |
| Self-Enhancement | Models rate own outputs higher | Use different model for evaluation |
| Verbosity | Detailed explanations favored | Criteria-specific rubrics |
| Authority | Confident tone rated higher | Require evidence citation |

## Direct Scoring Implementation

```markdown
You are an expert evaluator assessing response quality.

## Task
Evaluate the following response against each criterion.

## Original Prompt
{prompt}

## Response to Evaluate
{response}

## Criteria
{criteria with descriptions and weights}

## Instructions
For each criterion:
1. Find specific evidence in the response
2. Score according to the rubric (1-{max} scale)
3. Justify your score with evidence
4. Suggest one specific improvement

## Output Format
Respond with structured JSON containing scores, justifications, and summary.
```

**Critical:** Always require justification BEFORE the score. Improves reliability 15-25%.

## Pairwise Comparison Implementation

**Position Bias Mitigation Protocol:**
1. First pass: A in first position, B in second
2. Second pass: B in first position, A in second
3. Consistency check: If passes disagree, return TIE
4. Final verdict: Consistent winner with averaged confidence

```markdown
## Critical Instructions
- Do NOT prefer responses because they are longer
- Do NOT prefer responses based on position (first vs second)
- Focus ONLY on quality according to specified criteria
- Ties are acceptable when genuinely equivalent
```

## Rubric Generation

**Components:**
1. Level descriptions with clear boundaries
2. Observable characteristics for each level
3. Examples for each level
4. Edge case guidance
5. General scoring principles

**Strictness levels:**
- Lenient: Lower bar, encourages iteration
- Balanced: Typical production use
- Strict: High-stakes or safety-critical

## Decision Framework

```
Is there objective ground truth?
├── Yes → Direct Scoring
│   (factual accuracy, instruction following)
└── No → Is it a preference judgment?
    ├── Yes → Pairwise Comparison
    │   (tone, style, persuasiveness)
    └── No → Reference-based evaluation
        (summarization, translation)
```

## Scaling Evaluation

| Approach | Use Case | Trade-off |
|----------|----------|-----------|
| Panel of LLMs | High-stakes decisions | More expensive, more reliable |
| Hierarchical | Large volumes | Fast screening + careful edge cases |
| Human-in-loop | Critical applications | Best reliability, feedback loop |

## Guidelines

1. Always require justification before scores
2. Always swap positions in pairwise comparison
3. Match scale granularity to rubric specificity
4. Separate objective and subjective criteria
5. Include confidence scores calibrated to consistency
6. Define edge cases explicitly
7. Validate against human judgments
