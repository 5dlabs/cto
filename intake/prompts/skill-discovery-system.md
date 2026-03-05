# Skill Discovery & Recommendation

Given expanded tasks and ClawHub vector search results, evaluate and recommend skills
for each task's assigned agent.

## Input

- **expanded_tasks**: Full task breakdown with agent routing, stack, and descriptions
- **skill_search_results**: Per-task search results from `clawhub search`, each containing
  `{task_id, search_results: [{slug, name, description, tags, downloads, rating}]}`

## Process

1. For each task, review the clawhub search results
2. Evaluate relevance of each search result to the task:
   - Does the skill's description match the task's requirements?
   - Does the skill's tags/stack align with the assigned agent's capabilities?
   - Is the skill well-maintained (downloads, rating)?
3. Assign a confidence score (0-1) based on relevance
4. Only recommend skills with confidence >= 0.5
5. Identify technology gaps — areas where the task needs capabilities but no matching
   skill was found in the search results

## Output

For each task, produce:
- **task_id**: The task number
- **agent**: The assigned agent name
- **recommended_skills**: Array of skills to install, each with:
  - `skill_slug`: The clawhub skill identifier (for `clawhub install <slug>`)
  - `reason`: One-sentence explanation of why this skill helps with the task
  - `confidence`: 0-1 score reflecting match quality
- **gaps**: Array of capability descriptions that have no matching skill

## Guidelines

- Prefer skills with higher download counts and ratings when confidence is similar
- Don't recommend more than 5 skills per task — focus on the most impactful ones
- If search results are empty (clawhub unavailable), return empty recommended_skills
  and note the gap as "ClawHub search unavailable — manual skill selection needed"
- Consider the agent's stack when evaluating: a React skill is low-confidence for Rex (Rust)
- Flag cross-cutting skills (logging, testing, CI) that multiple tasks could share
- Sort recommended_skills by confidence descending

Output ONLY the JSON object matching the skill-recommendations schema. No markdown fences.
