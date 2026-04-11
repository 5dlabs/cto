---
name: code-maturity
description: Assess codebase maturity using 9-category framework - arithmetic, auditing, access controls, complexity, and more.
---

# Code Maturity Assessor

Systematically assess a codebase's maturity using Trail of Bits' 9-category framework.

## Purpose

Provide evidence-based ratings and actionable recommendations by analyzing code against established criteria.

## Assessment Process

### Phase 1: Discovery

Explore the codebase to understand:
- Project structure and platform
- Module/component files
- Test coverage
- Documentation availability

### Phase 2: Analysis

For each of 9 categories:
- **Search the code** for relevant patterns
- **Read key files** to assess implementation
- **Present findings** with file references
- **Ask clarifying questions** about processes not visible in code
- **Determine rating** based on criteria

### Phase 3: Report

Generate:
- Executive summary
- Maturity scorecard (ratings for all 9 categories)
- Detailed analysis with evidence
- Priority-ordered improvement roadmap

## Rating System

| Rating | Score | Description |
|--------|-------|-------------|
| Missing | 0 | Not present/not implemented |
| Weak | 1 | Several significant improvements needed |
| Moderate | 2 | Adequate, can be improved |
| Satisfactory | 3 | Above average, minor improvements |
| Strong | 4 | Exceptional, only small improvements possible |

**Rating Logic:**
- ANY "Weak" criteria → **Weak**
- NO "Weak" + SOME "Moderate" unmet → **Moderate**
- ALL "Moderate" + SOME "Satisfactory" met → **Satisfactory**
- ALL "Satisfactory" + exceptional practices → **Strong**

## The 9 Categories

### 1. ARITHMETIC
- Overflow protection mechanisms
- Precision handling and rounding
- Formula specifications
- Edge case testing

### 2. AUDITING
- Event definitions and coverage
- Logging comprehensiveness
- Monitoring infrastructure
- Incident response planning

### 3. AUTHENTICATION / ACCESS CONTROLS
- Privilege management
- Role separation
- Access control testing
- Key compromise scenarios

### 4. COMPLEXITY MANAGEMENT
- Function scope and clarity
- Cyclomatic complexity
- Inheritance hierarchies
- Code duplication

### 5. DECENTRALIZATION
- Centralization risks
- Upgrade control mechanisms
- User opt-out paths
- Timelock/multisig patterns

### 6. DOCUMENTATION
- Specifications and architecture
- Inline code documentation
- User stories
- Domain glossaries

### 7. TRANSACTION ORDERING RISKS
- Race condition vulnerabilities
- Front-running protections
- Timing-dependent operations
- Oracle security

### 8. LOW-LEVEL MANIPULATION
- Unsafe code sections
- Low-level calls
- Pointer/memory operations
- Justification and testing

### 9. TESTING & VERIFICATION
- Test coverage
- Fuzzing and property-based testing
- CI/CD integration
- Test quality and maintainability

## Report Structure

### 1. Executive Summary
- Project name and platform
- Overall maturity (average rating)
- Top 3 strengths
- Top 3 critical gaps
- Priority recommendations

### 2. Maturity Scorecard

| Category | Rating | Score | Notes |
|----------|--------|-------|-------|
| Arithmetic | Satisfactory | 3 | Good overflow handling |
| Auditing | Moderate | 2 | Events present, monitoring gaps |
| ... | ... | ... | ... |

### 3. Detailed Analysis

For each category:
- Evidence with file:line references
- Gaps identified
- Improvement actions

### 4. Improvement Roadmap

**CRITICAL (Immediate)**
- Item with effort estimate and impact

**HIGH (1-2 months)**
- Item with effort estimate and impact

**MEDIUM (2-4 months)**
- Item with effort estimate and impact

## Common Rationalizations (Avoid)

| Rationalization | Why It's Wrong |
|-----------------|----------------|
| "Found some findings, assessment complete" | Must evaluate ALL 9 categories |
| "I see events, auditing looks good" | Events alone don't equal maturity |
| "Code looks simple, complexity is low" | Visual simplicity masks composition complexity |
| "No assembly found, low-level is N/A" | Low-level risks include external calls, unsafe blocks |
| "I can rate without evidence" | Ratings without file:line references = unsubstantiated |

## Estimated Time

30-40 minutes for a full assessment

## Requirements

- Access to full codebase
- Knowledge of processes (monitoring, incident response, team practices)
- Context about the project type

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) code-maturity-assessor skill.
