# Test Intake Files

This directory contains test files for validating the intake workflow functionality.

## Files

- **`prd.txt`**: Product Requirements Document for an Enterprise Task Management Platform
- **`architecture.md`**: System architecture document outlining the technical design

## Usage

### For MCP Intake Tool

When calling the intake tool, you can reference these files:

```javascript
intake({
  project_name: "enterprise-task-management",
  prd_content: "Content from prd.txt",  // or provide file path
  architecture_content: "Content from architecture.md"  // or provide file path
})
```

### For Manual Testing

You can also test by placing these files in a project directory:

```
my-project/
├── prd.txt           # Copy from docs/test-intake/prd.txt
├── architecture.md   # Copy from docs/test-intake/architecture.md
└── intake/           # Optional subdirectory location
    ├── prd.txt
    └── architecture.md
```

The intake workflow will automatically detect and process these files.

## Test Scenario

This test case covers:
- **Complex PRD**: Multi-phase enterprise software development
- **Comprehensive Architecture**: Multi-layer system design with scalability considerations
- **Integration Points**: Multiple external service integrations
- **Security Requirements**: Enterprise-grade security and compliance
- **Performance Requirements**: High-availability and scalability needs

## Expected Outcome

The intake workflow should generate:
- Multiple top-level tasks for different development phases
- Detailed subtasks for each major component
- Proper dependency relationships
- Realistic time estimates based on complexity
- Appropriate priority assignments

## Validation

After running intake, verify:
- ✅ Tasks are created with proper structure
- ✅ Dependencies are correctly identified
- ✅ Complexity analysis provides reasonable estimates
- ✅ No configuration transmission errors (with our new strict validation)

This test validates the end-to-end intake workflow including the new granular model configuration and strict parameter validation.
