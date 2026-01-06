---
name: evaluation
description: Agent evaluation methods including LLM-as-judge, rubrics, and quality gates.
agents: [tess, cleo]
triggers: [evaluate, test framework, measure quality, rubric, quality gate]
---

# Evaluation Methods for Agent Systems

Evaluation of agent systems requires different approaches than traditional software or even standard language model applications. Agents make dynamic decisions, are non-deterministic between runs, and often lack single correct answers. Effective evaluation must account for these characteristics while providing actionable feedback.

## When to Activate

Activate this skill when:
- Testing agent performance systematically
- Validating context engineering choices
- Measuring improvements over time
- Catching regressions before deployment
- Building quality gates for agent pipelines
- Comparing different agent configurations
- Evaluating production systems continuously

## Core Concepts

Agent evaluation requires outcome-focused approaches that account for non-determinism and multiple valid paths. Multi-dimensional rubrics capture various quality aspects: factual accuracy, completeness, citation accuracy, source quality, and tool efficiency. LLM-as-judge provides scalable evaluation while human evaluation catches edge cases.

The key insight is that agents may find alternative paths to goals—the evaluation should judge whether they achieve right outcomes while following reasonable processes.

## Performance Drivers: The 95% Finding

Research found that three factors explain 95% of performance variance:

| Factor | Variance Explained | Implication |
|--------|-------------------|-------------|
| Token usage | 80% | More tokens = better performance |
| Number of tool calls | ~10% | More exploration helps |
| Model choice | ~5% | Better models multiply efficiency |

## Evaluation Challenges

**Non-Determinism and Multiple Valid Paths**
Agents may take completely different valid paths to reach goals. The solution is outcome-focused evaluation that judges whether agents achieve right outcomes while following reasonable processes.

**Context-Dependent Failures**
Agent failures often depend on context in subtle ways. Evaluation must cover a range of complexity levels and test extended interactions, not just isolated queries.

**Composite Quality Dimensions**
Agent quality includes factual accuracy, completeness, coherence, tool efficiency, and process quality. Evaluation rubrics must capture multiple dimensions with appropriate weighting.

## Evaluation Rubric Design

### Multi-Dimensional Rubric

| Dimension | Excellent | Good | Failed |
|-----------|-----------|------|--------|
| Factual accuracy | Claims match ground truth | Minor errors | False claims |
| Completeness | Covers all aspects | Covers most | Missing key info |
| Citation accuracy | All citations match | Most match | Wrong citations |
| Source quality | Primary sources | Secondary OK | Poor sources |
| Tool efficiency | Optimal tool use | Some waste | Many wasted calls |

### Rubric Scoring

Convert dimension assessments to numeric scores (0.0 to 1.0) with appropriate weighting. Calculate weighted overall scores. Determine passing threshold based on use case requirements.

## Evaluation Methodologies

### LLM-as-Judge

LLM-based evaluation scales to large test sets and provides consistent judgments. The key is designing effective evaluation prompts that capture the dimensions of interest.

Provide clear task description, agent output, ground truth (if available), evaluation scale with level descriptions, and request structured judgment.

### Human Evaluation

Human evaluation catches what automation misses. Humans notice hallucinated answers on unusual queries, system failures, and subtle biases that automated evaluation misses.

### End-State Evaluation

For agents that mutate persistent state, end-state evaluation focuses on whether the final state matches expectations rather than how the agent got there.

## Test Set Design

### Complexity Stratification

Test sets should span complexity levels:
- **Simple**: Single tool call
- **Medium**: Multiple tool calls
- **Complex**: Many tool calls, significant ambiguity
- **Very Complex**: Extended interaction, deep reasoning

```python
test_set = [
    {
        "name": "simple_lookup",
        "input": "What is the capital of France?",
        "expected": {"type": "fact", "answer": "Paris"},
        "complexity": "simple"
    },
    {
        "name": "multi_step_reasoning",
        "input": "Analyze sales data from Q1-Q4 and create summary with trends",
        "complexity": "complex"
    }
]
```

## Context Engineering Evaluation

**Testing Context Strategies**
Context engineering choices should be validated through systematic evaluation. Run agents with different context strategies on the same test set. Compare quality scores, token usage, and efficiency metrics.

**Degradation Testing**
Test how context degradation affects performance by running agents at different context sizes. Identify performance cliffs. Establish safe operating limits.

## Guidelines

1. Use multi-dimensional rubrics, not single metrics
2. Evaluate outcomes, not specific execution paths
3. Cover complexity levels from simple to complex
4. Test with realistic context sizes and histories
5. Run evaluations continuously, not just before release
6. Supplement LLM evaluation with human review
7. Track metrics over time for trend detection
8. Set clear pass/fail thresholds based on use case
