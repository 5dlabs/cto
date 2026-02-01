---
name: ask-questions
description: Ask clarifying questions when requests are underspecified - minimum questions to avoid wrong work.
---

# Ask Questions If Underspecified

## When to Use

Use this skill when a request has multiple plausible interpretations or key details (objective, scope, constraints, environment, or safety) are unclear.

## When NOT to Use

Do not use this skill when the request is already clear, or when a quick, low-risk discovery read can answer the missing details.

## Goal

Ask the minimum set of clarifying questions needed to avoid wrong work; do not start implementing until the must-have questions are answered (or the user explicitly approves proceeding with stated assumptions).

## Workflow

### 1. Decide whether the request is underspecified

Treat a request as underspecified if after exploring how to perform the work, some or all of the following are not clear:

- Define the objective (what should change vs stay the same)
- Define "done" (acceptance criteria, examples, edge cases)
- Define scope (which files/components/users are in/out)
- Define constraints (compatibility, performance, style, deps, time)
- Identify environment (language/runtime versions, OS, build/test runner)
- Clarify safety/reversibility (data migration, rollout/rollback, risk)

If multiple plausible interpretations exist, assume it is underspecified.

### 2. Ask must-have questions first (keep it small)

Ask 1-5 questions in the first pass. Prefer questions that eliminate whole branches of work.

Make questions easy to answer:
- Optimize for scannability (short, numbered questions; avoid paragraphs)
- Offer multiple-choice options when possible
- Suggest reasonable defaults when appropriate
- Include a fast-path response (e.g., reply `defaults` to accept all defaults)
- Include a low-friction "not sure" option when helpful
- Separate "Need to know" from "Nice to know" if that reduces friction
- Structure options so the user can respond with compact decisions (e.g., `1b 2a 3c`)

### 3. Pause before acting

Until must-have answers arrive:
- Do not run commands, edit files, or produce a detailed plan that depends on unknowns
- Do perform a clearly labeled, low-risk discovery step only if it does not commit you to a direction

If the user explicitly asks you to proceed without answers:
- State your assumptions as a short numbered list
- Ask for confirmation; proceed only after they confirm or correct them

### 4. Confirm interpretation, then proceed

Once you have answers, restate the requirements in 1-3 sentences (including key constraints and what success looks like), then start work.

## Question Templates

```
Before I start, I need: (1) ..., (2) ..., (3) ....
If you don't care about (2), I will assume ....
```

```
Which of these should it be?
A) ...
B) ...
C) ... (pick one)
```

```
1) Scope?
   a) Minimal change (default)
   b) Refactor while touching the area
   c) Not sure - use default

2) Compatibility target?
   a) Current project defaults (default)
   b) Also support older versions: <specify>
   c) Not sure - use default

Reply with: defaults (or 1a 2a)
```

## Anti-patterns

- Don't ask questions you can answer with a quick, low-risk discovery read (e.g., configs, existing patterns, docs)
- Don't ask open-ended questions if a tight multiple-choice or yes/no would eliminate ambiguity faster
- Don't ask more than 5 questions in the first pass
- Don't hold up work for "nice to know" information

## Examples

### Good: Minimal, Actionable Questions

```
Before implementing the API endpoint, I need to clarify:

1) Response format?
   a) JSON (default)
   b) JSON + pagination
   c) Other: ___

2) Auth required?
   a) Yes - use existing middleware (default)
   b) No - public endpoint
   c) Not sure

Reply: defaults to use (1a, 2a)
```

### Bad: Open-Ended, Verbose

```
I have some questions about this feature. First, I was wondering
what you had in mind for the response format - there are several
options we could consider. Also, should this be authenticated?
There are pros and cons to each approach. Additionally, have you
thought about pagination? What about rate limiting? And caching?
```

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) ask-questions-if-underspecified skill.
