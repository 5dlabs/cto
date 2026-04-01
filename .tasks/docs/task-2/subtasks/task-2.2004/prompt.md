Implement subtask 2004: Remove legacy agent:pending label code

## Objective
Find and remove all code paths that set or reference the 'agent:pending' label on Linear issues as a fallback assignment mechanism.

## Steps
1. Search the PM server codebase for all references to 'agent:pending' (string literals, label creation calls, label filtering logic).
2. Remove label creation/attachment in the issueCreate mutation payload and any post-creation label-setting calls.
3. Remove any helper functions or constants that define the 'agent:pending' label ID or name.
4. Check for any webhook handlers or scheduled jobs that react to the 'agent:pending' label and remove/update them.
5. If there are Linear labels already created in the workspace, note in a comment/README that existing 'agent:pending' labels on old issues can be cleaned up manually.

## Validation
Grep the entire codebase for 'agent:pending' — zero results expected. Run existing test suite to confirm no regressions from removed code paths.