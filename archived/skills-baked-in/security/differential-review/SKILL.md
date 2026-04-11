---
name: differential-review
description: Security-focused code review for PRs and diffs - risk classification, blast radius, attack scenarios.
---

# Differential Security Review

Security-focused code review for PRs, commits, and diffs.

## Core Principles

1. **Risk-First**: Focus on auth, crypto, value transfer, external calls
2. **Evidence-Based**: Every finding backed by git history, line numbers, attack scenarios
3. **Adaptive**: Scale to codebase size (SMALL/MEDIUM/LARGE)
4. **Honest**: Explicitly state coverage limits and confidence level
5. **Output-Driven**: Always generate comprehensive markdown report file

## Codebase Size Strategy

| Codebase Size | Strategy | Approach |
|---------------|----------|----------|
| SMALL (<20 files) | DEEP | Read all deps, full git blame |
| MEDIUM (20-200) | FOCUSED | 1-hop deps, priority files |
| LARGE (200+) | SURGICAL | Critical paths only |

## Risk Level Triggers

| Risk Level | Triggers |
|------------|----------|
| HIGH | Auth, crypto, external calls, value transfer, validation removal |
| MEDIUM | Business logic, state changes, new public APIs |
| LOW | Comments, tests, UI, logging |

## Workflow Overview

```
Pre-Analysis → Phase 0: Triage → Phase 1: Code Analysis → Phase 2: Test Coverage
      ↓              ↓                    ↓                      ↓
Phase 3: Blast Radius → Phase 4: Deep Context → Phase 5: Adversarial → Phase 6: Report
```

## Phase Summaries

### Phase 0: Triage
- Classify files by risk level
- Identify HIGH risk files for deep analysis

### Phase 1: Code Analysis
- Git blame on removed security code
- Analyze changes for security implications

### Phase 2: Test Coverage
- Check if security-critical changes have tests
- Flag missing tests as elevated risk

### Phase 3: Blast Radius
- Calculate how many callers are affected
- High blast radius + HIGH risk = immediate escalation

### Phase 4: Deep Context
- For HIGH risk changes, build full context
- Trace data flow, understand invariants

### Phase 5: Adversarial
- Model attacker perspective
- Develop concrete exploit scenarios
- Rate exploitability

### Phase 6: Report
- Generate comprehensive markdown report
- Include all findings with file:line references

## Red Flags (Stop and Investigate)

**Immediate escalation triggers:**
- Removed code from "security", "CVE", or "fix" commits
- Access control modifiers removed (onlyOwner, internal → external)
- Validation removed without replacement
- External calls added without checks
- High blast radius (50+ callers) + HIGH risk change

These patterns require adversarial analysis even in quick triage.

## Rationalizations (Do Not Skip)

| Rationalization | Why It's Wrong | Required Action |
|-----------------|----------------|-----------------|
| "Small PR, quick review" | Heartbleed was 2 lines | Classify by RISK, not size |
| "I know this codebase" | Familiarity breeds blind spots | Build explicit baseline context |
| "Git history takes too long" | History reveals regressions | Never skip Phase 1 |
| "Just a refactor, no security impact" | Refactors break invariants | Analyze as HIGH until proven LOW |

## Quality Checklist

Before delivering:
- [ ] All changed files analyzed
- [ ] Git blame on removed security code
- [ ] Blast radius calculated for HIGH risk
- [ ] Attack scenarios are concrete (not generic)
- [ ] Findings reference specific line numbers + commits
- [ ] Report file generated

## When NOT to Use

- Greenfield code (no baseline to compare)
- Documentation-only changes
- Formatting/linting changes
- User explicitly requests quick summary only

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) differential-review skill - 45+ installs.
