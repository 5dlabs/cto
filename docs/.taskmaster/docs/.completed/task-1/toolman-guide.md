# Toolman Guide: Analyze Existing CodeRun Controller Architecture

## Overview

This task requires deep code analysis and documentation capabilities. The selected tools focus on file exploration, documentation creation, and knowledge management for architectural discovery.

## Core Tools

### Filesystem Operations
The filesystem server provides essential tools for exploring the codebase:

#### `read_file`
- **Purpose**: Read complete source files for analysis
- **When to Use**: Examining controller code, CRD definitions, templates
- **Example Usage**: Reading `controller/src/crds/coderun.rs` to understand CRD structure
- **Best Practice**: Read files systematically, starting with entry points

#### `directory_tree`
- **Purpose**: Get hierarchical view of project structure
- **When to Use**: Understanding code organization and finding relevant files
- **Example Usage**: Mapping the `infra/charts/controller/claude-templates/` structure
- **Best Practice**: Use before deep diving to understand layout

#### `search_files`
- **Purpose**: Find files matching patterns across the codebase
- **When to Use**: Locating specific components or references
- **Example Usage**: Finding all Handlebars templates with `*.hbs` pattern
- **Best Practice**: Use case-insensitive search for flexibility

#### `list_directory`
- **Purpose**: Get detailed file listings with metadata
- **When to Use**: Inventorying template files or configuration
- **Example Usage**: Listing all files in controller source directory
- **Best Practice**: Distinguish between files and directories in analysis

#### `write_file`
- **Purpose**: Create documentation and analysis reports
- **When to Use**: Generating the discovery report and architecture documentation
- **Example Usage**: Writing `task-1-discovery-report.md`
- **Best Practice**: Structure documents with clear sections and cross-references

## Supporting Tools

### Knowledge Management

#### `memory_create_entities`
- **Purpose**: Create knowledge graph nodes for architectural components
- **When to Use**: Recording discovered components, patterns, and relationships
- **Example Usage**: Creating entities for "CodeRun Controller", "Template System", "Reconciliation Logic"
- **Best Practice**: Use consistent naming for related components

#### `memory_add_observations`
- **Purpose**: Add detailed findings to architectural entities
- **When to Use**: Recording specific discoveries about components
- **Example Usage**: Adding reconciliation pattern details to controller entity
- **Best Practice**: Include code references and examples in observations

### Research Tools

#### `brave_web_search`
- **Purpose**: Find external documentation and best practices
- **When to Use**: Researching Kubernetes patterns, Handlebars features, Argo Events
- **Example Usage**: Searching for "Kubernetes controller reconciliation patterns"
- **Best Practice**: Verify findings against actual code implementation

## Implementation Flow

### Phase 1: Initial Discovery
1. Use `directory_tree` to map project structure
2. Use `search_files` to locate key components:
   - Controller source files (`*.rs`)
   - Template files (`*.hbs`)
   - Configuration files (`*.yaml`)

### Phase 2: Deep Analysis
1. Use `read_file` to examine:
   - CRD definitions in `coderun.rs`
   - Controller logic in `controller.rs`
   - Resource creation in `resources.rs`
   - Template rendering in `templates.rs`
2. Create knowledge entities for major components
3. Add observations as patterns are discovered

### Phase 3: Template Investigation
1. Use `list_directory` on template directories
2. Read each template file to understand structure
3. Document Handlebars features and conditionals
4. Map variable substitution patterns

### Phase 4: Documentation Generation
1. Use `write_file` to create discovery report
2. Structure findings by architectural layer
3. Include code examples and diagrams
4. Cross-reference source files

### Phase 5: Knowledge Synthesis
1. Create relationship between entities
2. Add final observations and insights
3. Generate implementation recommendations
4. Document modification requirements

## Best Practices

### Systematic Exploration
- Start with high-level structure, then dive deep
- Follow code references to understand flow
- Document assumptions and validate with testing
- Keep notes in knowledge graph for persistence

### Documentation Quality
- Use clear technical language
- Include code snippets with context
- Provide file paths for all references
- Create diagrams for complex flows

### Analysis Depth
- Don't just describe - explain the "why"
- Identify patterns and anti-patterns
- Consider implications for multi-agent support
- Document edge cases and error handling

## Common Patterns

### Controller Analysis Pattern
1. Start with CRD definition
2. Trace through controller reconciliation
3. Follow resource creation flow
4. Document status management
5. Map cleanup and finalization

### Template System Pattern
1. Inventory all template files
2. Identify common structures
3. Document variable usage
4. Test conditional logic
5. Map rendering pipeline

## Troubleshooting

### Finding Hidden Dependencies
- Use `search_files` with partial names
- Check import statements in source files
- Follow function calls through codebase
- Look for configuration in unexpected places

### Understanding Complex Logic
- Break down into smaller functions
- Create sequence diagrams
- Test with actual CRD submissions
- Monitor controller logs for behavior

### Documenting Discoveries
- Use consistent formatting
- Include context for code snippets
- Explain technical decisions
- Provide examples for patterns

## Notes

This is primarily a discovery and documentation task. The tools selected emphasize:
- Deep file system exploration
- Knowledge capture and organization
- Documentation generation
- Research capabilities

Focus on understanding before proposing changes. The quality of this analysis directly impacts the success of subsequent multi-agent implementation tasks.