---
name: pr-review
description: Pull request review patterns including focus areas, feedback guidelines, and approval criteria.
agents: [stitch]
triggers: [pr, pull request, review, feedback, merge]
---

# Pull Request Review

Patterns for effective code review that maintains quality while supporting developers.

## Review Focus Areas

1. **Correctness** - Does the code do what it's supposed to?
2. **Security** - Are there any security vulnerabilities?
3. **Performance** - Are there performance concerns?
4. **Maintainability** - Is the code readable and maintainable?
5. **Testing** - Are there adequate tests?
6. **Style** - Does it follow project conventions?

## Review Process

1. **PR Details** - Fetch PR diff and description
2. **Context** - Understand the feature/fix being implemented
3. **Code Analysis** - Review each changed file
4. **Pattern Recognition** - Check for known issues with similar patterns
5. **Feedback** - Post review comments

## Review Guidelines

### Be Constructive

- Focus on the code, not the person
- Explain the "why" behind suggestions
- Offer alternatives, not just criticism
- Acknowledge good patterns when you see them

### Categorize Feedback

| Category | Action | Example |
|----------|--------|---------|
| **Blocking** | Must fix before merge | Security vulnerability, bug |
| **Suggestion** | Should consider | Performance improvement |
| **Nit** | Nice to have | Style preference |
| **Question** | Need clarification | Design decision |

### Common Review Points

**Code Quality:**
- Are function names clear and descriptive?
- Is the code DRY (Don't Repeat Yourself)?
- Are there any obvious bugs?
- Is error handling comprehensive?

**Security:**
- Are inputs validated?
- Are secrets properly managed?
- Are there any injection vulnerabilities?
- Is auth/authz properly enforced?

**Performance:**
- Are there N+1 query patterns?
- Is there unnecessary computation?
- Are there memory leaks?
- Is caching used appropriately?

**Testing:**
- Are new features tested?
- Are edge cases covered?
- Are tests readable and maintainable?
- Is test coverage adequate?

**Documentation:**
- Are public APIs documented?
- Are complex algorithms explained?
- Is the PR description clear?

## Approval Criteria

**Approve** if:
- Code is correct and addresses the requirements
- No security vulnerabilities
- Tests are adequate
- Style follows conventions
- Only nits or minor suggestions remain

**Request Changes** if:
- There are blocking issues
- Security vulnerabilities exist
- Critical functionality is untested
- Major design concerns

**Comment** if:
- Have questions but no blocking issues
- Want to discuss alternatives
- Providing information for future consideration

## Review Etiquette

- Review promptly (within 24 hours)
- Be respectful and professional
- Assume positive intent
- If it's not clear, ask
- Provide context for your suggestions
- Follow up on your own comments

## PR Description Template

Good PRs include:

```markdown
## Summary
Brief description of what this PR does

## Changes
- Change 1
- Change 2

## Testing
How this was tested

## Screenshots (if UI)
Before/after screenshots

## Related Issues
Closes #123
```
