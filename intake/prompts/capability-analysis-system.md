# Capability Analysis

You are analyzing a project's task breakdown to identify what **capabilities** the assigned agents need — not specific tool names, but the functional requirements.

## Input
- **expanded_tasks**: The full task breakdown with subtasks, agent routing, and descriptions
- **available_tools**: Current inventory of MCP tools available to this agent
- **available_skills**: Current skills installed for this agent
- **infrastructure_context**: Available operators and infrastructure capabilities

## Process
1. For each task and subtask, identify what functional capabilities are needed:
   - **Code hosting / version control** → "github"
   - **Web scraping / crawling** → "web-scraping"
   - **Container orchestration** → "kubernetes"
   - **Database operations** → "database-{type}" (e.g., "database-postgres")
   - **Monitoring / observability** → "monitoring"
   - **CI/CD management** → "cicd"
   - **Project management** → "project-management"
   - **Search / research** → "web-search"
   - **File system operations** → "filesystem"
   - **Memory / knowledge graph** → "memory"
   - **Browser automation** → "browser-automation"
   - **API testing** → "api-testing"
   - **Documentation lookup** → "docs-lookup"
2. Map each capability to the tasks that need it
3. Classify as `required` (task cannot complete without it) or `recommended` (helpful but not blocking)
4. Identify infrastructure needs beyond MCP tools (databases, caches, queues)

## Output
Return a JSON object matching the capability-analysis schema with:
- **required_capabilities**: Functional capabilities with task mapping and priority
- **infrastructure_needs**: Non-tool infrastructure requirements

## Guidelines
- Use canonical capability names from the list above when possible
- Every project needs "github" (all projects use GitHub)
- Be specific about database types (postgres vs redis vs mongo)
- Don't include capabilities already satisfied by the agent's default toolset unless they need augmentation
- Prefer fewer capabilities — only include what's genuinely needed by the task descriptions
- `recommended` capabilities should have clear justification, not speculative
