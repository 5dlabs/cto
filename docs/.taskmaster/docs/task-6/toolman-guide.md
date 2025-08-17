# Toolman Guide: Agent-Specific Handlebars Templates

## Overview

This task focuses on implementing specialized container script templates for multi-agent orchestration. You'll create agent-specific Handlebars templates and modify the controller's template selection logic.

## Tool Selection Strategy

### Primary Development Tools

**filesystem** - Essential for template creation and modification
- Create new template files (container-rex.sh.hbs, container-cleo.sh.hbs, container-tess.sh.hbs)
- Modify existing controller source code files
- Organize template directory structure
- Read existing templates for pattern consistency

**git** - Required for version control and change tracking
- Track template file additions and modifications
- Review existing template patterns and structure
- Commit incremental changes during development
- Branch management for template implementation

### Research and Documentation Tools

**rustdocs_query_rust_docs** - Critical for controller integration
- Research Handlebars template rendering in Rust
- Understand controller template loading mechanisms
- Query CRD field definitions and usage patterns
- Find template context building patterns

**memory_create_entities** - Store implementation knowledge
- Document agent-specific template requirements
- Track template selection logic implementation
- Remember controller modification patterns
- Store testing scenarios and validation approaches

**brave_web_search** - Supplemental research tool
- Research Handlebars conditional logic best practices
- Find Kubernetes template pattern examples
- Research multi-agent orchestration patterns
- Lookup Argo Workflows integration approaches

## Implementation Workflow

### Phase 1: Analysis and Planning
```
Tools: filesystem, git, rustdocs_query_rust_docs, memory_create_entities
```

1. **Examine Existing Templates**
   - Use `filesystem` to read current `container.sh.hbs`
   - Study existing Handlebars context and variables
   - Document current template patterns and structure

2. **Research Controller Template System**
   - Use `rustdocs_query_rust_docs` to understand template loading
   - Find template rendering functions in controller code
   - Identify where container scripts are generated

3. **Plan Agent-Specific Requirements**
   - Use `memory_create_entities` to document each agent's needs
   - Define environment variables and setup requirements
   - Plan template selection logic implementation

### Phase 2: Template Creation
```
Tools: filesystem, memory_create_entities
```

1. **Create Rex/Blaze Template**
   ```bash
   # Focus areas for container-rex.sh.hbs
   - Documentation-first workflow setup
   - MCP server integration preparation  
   - Task file access patterns
   - Implementation-focused environment
   ```

2. **Create Cleo Template**
   ```bash
   # Focus areas for container-cleo.sh.hbs
   - Code quality tools setup
   - GitHub API authentication
   - CI validation workflow
   - Ready-for-QA labeling logic
   ```

3. **Create Tess Template**
   ```bash
   # Focus areas for container-tess.sh.hbs
   - Admin access configuration
   - Testing infrastructure setup
   - Deployment validation tools
   - Comprehensive testing environment
   ```

### Phase 3: Controller Integration
```
Tools: filesystem, rustdocs_query_rust_docs, git
```

1. **Implement Template Selection Logic**
   - Modify `controller/src/tasks/code/templates.rs`
   - Add `get_container_template()` function
   - Implement agent-specific template mapping

2. **Update Template Loading**
   - Modify template rendering functions
   - Ensure proper fallback to default template
   - Add error handling for missing templates

3. **Test Integration Points**
   - Verify ConfigMap generation includes correct scripts
   - Test template variable resolution
   - Validate agent-specific context building

### Phase 4: Testing and Validation
```
Tools: filesystem, git, memory_create_entities
```

1. **Create Test Scenarios**
   - Test each agent template renders correctly
   - Verify template selection logic works
   - Test backward compatibility with existing workflows

2. **Document Implementation**
   - Use `memory_create_entities` to store validation results
   - Document any issues and solutions found
   - Create troubleshooting guidance

## Best Practices

### Template Development
- **Consistency**: Follow existing template naming and structure patterns
- **Modularity**: Create reusable template components where possible
- **Documentation**: Include comments in templates explaining agent-specific logic
- **Testing**: Test templates render without Handlebars errors

### Controller Integration  
- **Type Safety**: Ensure proper error handling for template operations
- **Performance**: Minimize template loading overhead
- **Maintainability**: Keep template selection logic simple and clear
- **Backward Compatibility**: Preserve existing workflow functionality

### Code Organization
- **File Structure**: Organize templates in logical directory structure
- **Naming Conventions**: Use clear, consistent naming patterns
- **Version Control**: Commit changes incrementally with clear messages
- **Code Review**: Ensure code follows project standards

## Tool Usage Examples

### Reading Existing Templates
```bash
# Use filesystem to examine current template structure
filesystem.read_file("infra/charts/controller/claude-templates/container.sh.hbs")
filesystem.list_directory("infra/charts/controller/claude-templates/")
```

### Researching Controller Code
```bash
# Use rustdocs to understand template system
rustdocs_query_rust_docs("How does Handlebars template rendering work in Rust?")
rustdocs_query_rust_docs("CodeRun CRD field definitions and usage")
```

### Creating New Templates
```bash
# Use filesystem to create agent-specific templates
filesystem.write_file("infra/charts/controller/claude-templates/container-rex.sh.hbs", template_content)
filesystem.write_file("infra/charts/controller/claude-templates/container-cleo.sh.hbs", template_content)
```

### Version Control Management
```bash
# Use git to track changes
git.status()  # Check current changes
git.diff()    # Review modifications
git.log()     # Review change history
```

## Common Pitfalls to Avoid

1. **Template Syntax Errors**: Test all Handlebars syntax before deployment
2. **Missing Variables**: Ensure all template variables are available in context
3. **Permission Issues**: Verify agents get appropriate access levels
4. **Breaking Changes**: Maintain backward compatibility with existing workflows
5. **Resource Conflicts**: Ensure agent workspaces remain isolated
6. **Error Handling**: Add proper fallbacks for template loading failures

## Success Validation

### Template Quality Checks
- [ ] All templates render without Handlebars errors
- [ ] Agent-specific environment variables are set correctly
- [ ] Required tools and credentials are available to each agent
- [ ] Template selection logic correctly maps GitHub Apps to templates

### Integration Quality Checks  
- [ ] Controller compiles and runs without errors
- [ ] Template loading performance is acceptable
- [ ] ConfigMaps contain correct container scripts
- [ ] Agent pods start successfully with new templates

### Workflow Quality Checks
- [ ] Rex/Blaze workflows continue working as before
- [ ] Cleo workflow has quality tools and GitHub API access
- [ ] Tess workflow has admin access and testing infrastructure
- [ ] Agent handoffs work correctly between workflow stages

This implementation requires careful attention to both template development and controller integration. Focus on creating clean, maintainable templates while ensuring the controller can reliably select and load the appropriate template for each agent type.