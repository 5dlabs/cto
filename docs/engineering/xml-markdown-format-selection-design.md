# XML/Markdown Format Selection Design



## Overview

This document outlines the design for supporting both XML and Markdown formats for task documentation and agent communication, with the ability to dynamically select the format through the MCP server when calling workflows.

## Problem Statement

While Markdown has been our primary format for task documentation and agent instructions, XML provides structured data advantages that can be beneficial for certain workflows. We need a flexible system that:



1. Supports both XML and Markdown formats


2. Allows format selection at runtime via MCP


3. Defaults to Markdown for backward compatibility


4. Handles format-specific processing in container scripts

## Design Goals

1. **Flexibility**: Support multiple documentation formats without code duplication
2. **Backward Compatibility**: Maintain Markdown as default to avoid breaking existing workflows
3. **Runtime Selection**: Allow format specification when submitting tasks
4. **Clean Abstraction**: Handle format differences transparently in the container layer
5. **Experimentation**: Enable A/B testing of formats for performance and accuracy comparison

## Proposed Solution

### 1. MCP Server Enhancement

Add a `format` parameter to relevant MCP tools:





```rust
// In mcp/src/tools.rs
Tool {
    name: "mcp_cto_code",
    parameters: json!({
        // ... existing parameters ...
        "format": {
            "type": "string",
            "description": "Documentation format to use (markdown or xml)",
            "enum": ["markdown", "xml"],
            "default": "markdown"
        }
    })
}

Tool {
    name: "mcp_cto_docs",
    parameters: json!({
        // ... existing parameters ...
        "format": {
            "type": "string",
            "description": "Documentation format to use (markdown or xml)",
            "enum": ["markdown", "xml"],
            "default": "markdown"
        }
    })
}

Tool {
    name: "mcp_cto_play",
    parameters: json!({
        // ... existing parameters ...
        "format": {
            "type": "string",
            "description": "Documentation format to use for all agents (markdown or xml)",
            "enum": ["markdown", "xml"],
            "default": "markdown"
        }
    })
}








```

### 2. Controller CRD Updates

Update the CRDs to include format specification:





```rust
// In controller/src/crds/code_run.rs
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "platform.5dlabs.com", version = "v1", kind = "CodeRun")]
pub struct CodeRunSpec {
    // ... existing fields ...

    /// Documentation format to use
    #[serde(default = "default_format")]
    pub format: DocumentFormat,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    Markdown,
    Xml,
}

fn default_format() -> DocumentFormat {
    DocumentFormat::Markdown
}








```

Similar updates for `DocsRunSpec` and `PlayRunSpec`.

### 3. Container Script Adaptation

Update container scripts to handle both formats:





```bash
# In container scripts (e.g., container-rex.sh.hbs)

# Detect format from environment or default to markdown
DOC_FORMAT="${DOC_FORMAT:-markdown}"

# Function to process documentation based on format
process_documentation() {
    local format="$1"
    local input_file="$2"
    local output_file="$3"

    case "$format" in
        xml)
            echo "📄 Processing XML documentation format"
            # XML-specific processing
            process_xml_documentation "$input_file" "$output_file"
            ;;
        markdown|*)
            echo "📝 Processing Markdown documentation format (default)"
            # Markdown processing (existing behavior)
            process_markdown_documentation "$input_file" "$output_file"
            ;;
    esac
}

# Function to generate prompt based on format
generate_prompt() {
    local format="$1"
    local task_id="$2"

    case "$format" in
        xml)
            cat > prompt.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<task>
    <id>${task_id}</id>
    <instructions>
        <!-- Task-specific instructions -->
    </instructions>
    <requirements>
        <!-- Requirements in XML structure -->
    </requirements>
</task>
EOF
            ;;
        markdown|*)
            cat > prompt.md << 'EOF'


# Task ${task_id}

## Instructions
<!-- Task-specific instructions -->

## Requirements
<!-- Requirements in Markdown -->
EOF
            ;;
    esac
}








```

### 4. Handlebars Template Updates

Update handlebars templates to support format selection:





```handlebars
{{!-- In agent-templates/code/claude/container-rex.sh.hbs --}}

# Set documentation format
export DOC_FORMAT="{{#if format}}{{format}}{{else}}markdown{{/if}}"

{{#if (eq format "xml")}}
# XML format specific configuration
echo "🔧 Configuring for XML documentation format"
export PROMPT_EXTENSION="xml"
export RESPONSE_FORMAT="xml"
{{else}}
# Markdown format configuration (default)
echo "🔧 Configuring for Markdown documentation format"
export PROMPT_EXTENSION="md"
export RESPONSE_FORMAT="markdown"
{{/if}}








```

### 5. Format-Specific Processing

#### XML Processing Benefits




```xml
<!-- Structured task definition -->
<task>
    <metadata>
        <id>42</id>
        <priority>high</priority>
        <agent>rex</agent>
    </metadata>
    <requirements>
        <requirement type="functional">
            <description>Implement user authentication</description>
            <acceptance_criteria>
                <criterion>JWT tokens are generated</criterion>
                <criterion>Refresh tokens are supported</criterion>
            </acceptance_criteria>
        </requirement>
    </requirements>
    <context>
        <files>
            <file path="src/auth.rs" action="modify"/>
            <file path="tests/auth_test.rs" action="create"/>
        </files>
    </context>
</task>








```

#### Markdown Processing (Default)




```markdown


# Task 42



## Metadata
- Priority: high
- Agent: rex

## Requirements
### Functional Requirements


- Implement user authentication


  - [ ] JWT tokens are generated


  - [ ] Refresh tokens are supported

## Context


### Files to Modify


- `src/auth.rs` (modify)


- `tests/auth_test.rs` (create)








```

### 6. Format Detection and Validation





```rust
// In controller/src/tasks/format.rs
pub trait DocumentFormatter {
    fn format_task(&self, task: &Task) -> Result<String>;
    fn parse_response(&self, response: &str) -> Result<TaskResponse>;
    fn validate(&self, content: &str) -> Result<()>;
}

pub struct MarkdownFormatter;
pub struct XmlFormatter;

impl DocumentFormatter for MarkdownFormatter {
    fn format_task(&self, task: &Task) -> Result<String> {
        // Generate Markdown representation
    }

    fn parse_response(&self, response: &str) -> Result<TaskResponse> {
        // Parse Markdown response
    }

    fn validate(&self, content: &str) -> Result<()> {
        // Validate Markdown structure
    }
}

impl DocumentFormatter for XmlFormatter {
    fn format_task(&self, task: &Task) -> Result<String> {
        // Generate XML representation
    }

    fn parse_response(&self, response: &str) -> Result<TaskResponse> {
        // Parse XML response
    }

    fn validate(&self, content: &str) -> Result<()> {
        // Validate XML structure (DTD/XSD)
    }
}








```

## Implementation Strategy

### Phase 1: MCP Parameter Addition (Day 1-2)


1. Add `format` parameter to MCP tools


2. Pass format through to controller


3. Default to markdown if not specified

### Phase 2: Controller Support (Day 3-4)


1. Update CRDs with format field


2. Add format to ConfigMap generation


3. Pass format to container environment

### Phase 3: Container Script Updates (Day 5-7)


1. Update container scripts to read format


2. Implement format-specific processing functions


3. Test both formats end-to-end

### Phase 4: Format Processing (Week 2)


1. Implement XML formatter


2. Update Markdown formatter


3. Add validation for both formats


4. Create conversion utilities



## Usage Examples

### Example 1: Default Markdown Usage




```typescript
// No format specified - uses markdown by default
await mcp.call("mcp_cto_code", {
    task_id: 42,
    agent: "rex"
});








```



### Example 2: Explicit XML Format




```typescript
// Specify XML format for structured data
await mcp.call("mcp_cto_code", {
    task_id: 42,
    agent: "rex",
    format: "xml"
});








```



### Example 3: Play Workflow with XML




```typescript
// Use XML for all agents in play workflow
await mcp.call("mcp_cto_play", {
    task_id: 42,
    implementation_agent: "rex",
    quality_agent: "cleo",
    testing_agent: "tess",
    format: "xml"  // All agents will use XML
});








```

### Example 4: A/B Testing Formats




```typescript
// Run same task with different formats for comparison
const markdownRun = await mcp.call("mcp_cto_code", {
    task_id: 42,
    agent: "rex",
    format: "markdown"
});

const xmlRun = await mcp.call("mcp_cto_code", {
    task_id: 42,
    agent: "rex",
    format: "xml"
});

// Compare performance, token usage, accuracy








```

## Environment Variables





```bash
# Container environment variables
DOC_FORMAT=xml|markdown          # Documentation format
PROMPT_EXTENSION=xml|md          # File extension for prompts
RESPONSE_FORMAT=xml|markdown     # Expected response format
VALIDATE_FORMAT=true|false       # Enable format validation








```

## Monitoring and Metrics

Track format usage and performance:





```rust
// Metrics to collect
static FORMAT_USAGE: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "agent_document_format_usage",
        "Document format usage by type",
        &["format", "agent"]
    ).unwrap()
});

static FORMAT_PROCESSING_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "agent_document_format_processing_seconds",
        "Time to process documents by format",
        &["format", "operation"]
    ).unwrap()
});

static TOKEN_USAGE_BY_FORMAT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "agent_tokens_by_format",
        "Token usage by document format",
        &["format", "agent"]
    ).unwrap()
});








```

## Migration Path

1. **Current State**: All workflows use Markdown
2. **Phase 1**: Add XML support, keep Markdown default
3. **Phase 2**: Test XML with specific tasks
4. **Phase 3**: Collect metrics, compare formats
5. **Phase 4**: Make format decision based on data
6. **Phase 5**: Potentially make XML default for certain task types

## Format Comparison

| Aspect | Markdown | XML |
|--------|----------|-----|
| **Readability** | High (human-friendly) | Medium (structured) |
| **Structure** | Flexible | Rigid (validated) |
| **Parsing** | Regex/Pattern | DOM/SAX parsers |
| **Token Usage** | Variable | Potentially lower |
| **Error Detection** | Runtime | Schema validation |
| **Tooling** | Wide support | XML-specific tools |
| **Agent Comprehension** | Good | Potentially better for structured data |

## Security Considerations

1. **XML Security**:


   - Disable external entity processing (XXE prevention)


   - Limit XML document size


   - Validate against schema/DTD

2. **Format Injection**:


   - Sanitize format parameter input


   - Validate format values against enum

3. **Resource Usage**:


   - Monitor memory usage for XML parsing


   - Set timeouts for format processing

## Testing Strategy

### Unit Tests




```rust


#[test]
fn test_markdown_format_default() {
    let spec = CodeRunSpec::default();
    assert_eq!(spec.format, DocumentFormat::Markdown);
}



#[test]
fn test_xml_format_selection() {
    let spec = CodeRunSpec {
        format: DocumentFormat::Xml,
        ..Default::default()
    };
    assert_eq!(spec.format, DocumentFormat::Xml);
}








```

### Integration Tests




```bash
# Test Markdown format (default)
mcp_cto_code --task-id 42



# Test XML format
mcp_cto_code --task-id 42 --format xml

# Verify outputs are equivalent
diff -u output_markdown.txt output_xml.txt








```

### Performance Tests


- Measure token usage for same task in both formats


- Compare completion times


- Analyze accuracy of responses


- Monitor resource consumption

## Future Enhancements

### 1. Format Auto-Selection




```rust
// Automatically choose format based on task complexity
fn auto_select_format(task: &Task) -> DocumentFormat {
    if task.has_complex_structure() || task.subtasks.len() > 10 {
        DocumentFormat::Xml  // Better for complex structured data
    } else {
        DocumentFormat::Markdown  // Better for simple tasks
    }
}








```



### 2. Custom Formats




```rust
enum DocumentFormat {
    Markdown,
    Xml,
    Json,
    Yaml,
    Custom(String),
}








```

### 3. Format Conversion




```rust
// Convert between formats
impl From<MarkdownDocument> for XmlDocument {
    fn from(md: MarkdownDocument) -> Self {
        // Conversion logic
    }
}








```



### 4. Format Templates




```yaml
# Format templates per agent/task type
format_templates:
  rex:
    backend_tasks: xml
    documentation: markdown
  cleo:
    quality_checks: xml
    reports: markdown








```

## Conclusion

This design provides a flexible, extensible system for supporting multiple documentation formats while maintaining backward compatibility. The ability to select formats at runtime enables experimentation and optimization based on real-world usage patterns. The default Markdown format ensures no disruption to existing workflows while XML support opens opportunities for more structured, validated task processing.
