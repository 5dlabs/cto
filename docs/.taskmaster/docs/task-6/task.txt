# Task ID: 6
# Title: Develop Agent-Specific Handlebars Templates
# Status: pending
# Dependencies: 1, 4
# Priority: medium
# Description: Create specialized container scripts for Rex/Blaze, Cleo, and Tess agents with template selection logic based on github_app parameter
# Details:
Implement simplified architecture using agent-specific container scripts instead of complex template conditionals. Create in infra/charts/controller/claude-templates/: container-rex.sh.hbs (documentation workflow - pulls docs, copies task files), container-cleo.sh.hbs (code quality workflow - focuses on formatting, Clippy, PR labeling with 'ready-for-qa'), container-tess.sh.hbs (testing workflow - deployment validation, test coverage, PR approval). Implement template selection logic in the controller to choose correct container script based on github_app field ('5DLabs-Rex', '5DLabs-Cleo', '5DLabs-Tess'). Keep system prompts simple and focused on each agent's core responsibilities. This approach avoids complex Rust controller modifications while maintaining clean separation of agent workflows.

# Test Strategy:
Test template selection logic with different github_app values. Verify correct container script is selected for each agent. Test each container script executes its specific workflow correctly. Validate agent-specific behaviors: Rex pulls documentation, Cleo runs formatting/Clippy, Tess focuses on testing. Ensure clean handoff between agents through workflow stages.

# Subtasks:
## 1. Create container-rex.sh.hbs template [pending]
### Dependencies: None
### Description: Implement Rex/Blaze container script for documentation workflow
### Details:


## 2. Create container-cleo.sh.hbs template [pending]
### Dependencies: None
### Description: Implement Cleo container script for code quality workflow
### Details:


## 3. Create container-tess.sh.hbs template [pending]
### Dependencies: None
### Description: Implement Tess container script for testing workflow
### Details:


## 4. Implement template selection logic [pending]
### Dependencies: None
### Description: Add logic to select correct container script based on github_app field
### Details:


## 5. Update template loading mechanism [pending]
### Dependencies: None
### Description: Modify controller to load agent-specific container scripts
### Details:


