---
name: brainstorming
description: Turn ideas into fully formed designs through collaborative dialogue - one question at a time.
---

# Brainstorming Ideas Into Designs

## Overview

Help turn ideas into fully formed designs and specs through natural collaborative dialogue.

Start by understanding the current project context, then ask questions one at a time to refine the idea. Once you understand what you're building, present the design in small sections (200-300 words), checking after each section whether it looks right so far.

## The Process

### Understanding the Idea

- Check out the current project state first (files, docs, recent commits)
- Ask questions one at a time to refine the idea
- Prefer multiple choice questions when possible, but open-ended is fine too
- Only one question per message
- Focus on understanding: purpose, constraints, success criteria

### Exploring Approaches

- Propose 2-3 different approaches with trade-offs
- Present options conversationally with your recommendation and reasoning
- Lead with your recommended option and explain why

### Presenting the Design

- Once you believe you understand what you're building, present the design
- Break it into sections of 200-300 words
- Ask after each section whether it looks right so far
- Cover: architecture, components, data flow, error handling, testing
- Be ready to go back and clarify if something doesn't make sense

## After the Design

### Documentation

- Write the validated design to `docs/plans/YYYY-MM-DD-<topic>-design.md`
- Commit the design document to git

### Implementation (if continuing)

- Ask: "Ready to set up for implementation?"
- Use git-worktrees skill to create isolated workspace
- Use writing-plans skill to create detailed implementation plan

## Key Principles

1. **One question at a time** - Don't overwhelm with multiple questions
2. **Multiple choice preferred** - Easier to answer than open-ended when possible
3. **YAGNI ruthlessly** - Remove unnecessary features from all designs
4. **Explore alternatives** - Always propose 2-3 approaches before settling
5. **Incremental validation** - Present design in sections, validate each
6. **Be flexible** - Go back and clarify when something doesn't make sense

## Example Flow

```
User: "I want to add caching to our API"

Agent: "Before diving in, let me understand the current setup.

What's the primary goal?
a) Reduce database load
b) Improve response times for users
c) Both equally important
d) Something else"

User: "b"

Agent: "Got it - response times are the priority.

Where are the slowest endpoints?
a) Complex queries (aggregations, joins)
b) External API calls
c) Not sure yet - need to profile
d) Specific endpoints: ___"
```

## Anti-Patterns

- Don't ask multiple questions in one message
- Don't present a complete design without validation checkpoints
- Don't skip the alternatives exploration phase
- Don't add features the user didn't ask for (YAGNI)

## Attribution

Based on [obra/superpowers](https://github.com/obra/superpowers) brainstorming skill - 220+ installs.
