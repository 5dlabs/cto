# MCP Server Development Guide

## Overview

Create MCP (Model Context Protocol) servers that enable LLMs to interact with external services through well-designed tools. The quality of an MCP server is measured by how well it enables LLMs to accomplish real-world tasks.

## High-Level Workflow

Creating a high-quality MCP server involves four main phases:

### Phase 1: Deep Research and Planning

#### Understand Modern MCP Design

**API Coverage vs. Workflow Tools:**
Balance comprehensive API endpoint coverage with specialized workflow tools. Workflow tools can be more convenient for specific tasks, while comprehensive coverage gives agents flexibility to compose operations. Performance varies by client—some clients benefit from code execution that combines basic tools, while others work better with higher-level workflows. When uncertain, prioritize comprehensive API coverage.

**Tool Naming and Discoverability:**
Clear, descriptive tool names help agents find the right tools quickly. Use consistent prefixes (e.g., `github_create_issue`, `github_list_repos`) and action-oriented naming.

**Context Management:**
Agents benefit from concise tool descriptions and the ability to filter/paginate results. Design tools that return focused, relevant data. Some clients support code execution which can help agents filter and process data efficiently.

**Actionable Error Messages:**
Error messages should guide agents toward solutions with specific suggestions and next steps.

#### Study MCP Protocol Documentation

**Navigate the MCP specification:**

Start with the sitemap to find relevant pages: `https://modelcontextprotocol.io/sitemap.xml`

Then fetch specific pages with `.md` suffix for markdown format (e.g., `https://modelcontextprotocol.io/specification/draft.md`).

Key pages to review:

- Specification overview and architecture
- Transport mechanisms (streamable HTTP, stdio)
- Tool, resource, and prompt definitions

#### Recommended Stack

- **Language**: TypeScript (high-quality SDK support and good compatibility in many execution environments)
- **Transport**: Streamable HTTP for remote servers (simpler to scale), stdio for local servers

### Phase 2: Implementation

#### Set Up Project Structure

```text
src/
├── index.ts           # Entry point
├── server.ts          # MCP server setup
├── tools/
│   ├── index.ts       # Tool registration
│   └── [tool-name].ts # Individual tools
├── utils/
│   ├── api-client.ts  # API wrapper
│   └── errors.ts      # Error handling
└── types.ts           # TypeScript types
```

#### Implement Tools

For each tool:

**Input Schema:**

- Use Zod (TypeScript) or Pydantic (Python)
- Include constraints and clear descriptions
- Add examples in field descriptions

**Output Schema:**

- Define `outputSchema` where possible for structured data
- Use `structuredContent` in tool responses (TypeScript SDK feature)
- Helps clients understand and process tool outputs

**Tool Description:**

- Concise summary of functionality
- Parameter descriptions
- Return type schema

**Implementation:**

- Async/await for I/O operations
- Proper error handling with actionable messages
- Support pagination where applicable
- Return both text content and structured data when using modern SDKs

**Annotations:**

- `readOnlyHint`: true/false
- `destructiveHint`: true/false
- `idempotentHint`: true/false
- `openWorldHint`: true/false

### Phase 3: Review and Test

#### Code Quality

Review for:

- No duplicated code (DRY principle)
- Consistent error handling
- Full type coverage
- Clear tool descriptions

#### Build and Test

**TypeScript:**

- Run `npm run build` to verify compilation
- Test with MCP Inspector: `npx @modelcontextprotocol/inspector`

**Python:**

- Verify syntax: `python -m py_compile your_server.py`
- Test with MCP Inspector

### Phase 4: Create Evaluations

After implementing your MCP server, create comprehensive evaluations to test its effectiveness.

#### Create 10 Evaluation Questions

To create effective evaluations, follow this process:

1. **Tool Inspection**: List available tools and understand their capabilities
2. **Content Exploration**: Use READ-ONLY operations to explore available data
3. **Question Generation**: Create 10 complex, realistic questions
4. **Answer Verification**: Solve each question yourself to verify answers

#### Evaluation Requirements

Ensure each question is:

- **Independent**: Not dependent on other questions
- **Read-only**: Only non-destructive operations required
- **Complex**: Requiring multiple tool calls and deep exploration
- **Realistic**: Based on real use cases humans would care about
- **Verifiable**: Single, clear answer that can be verified by string comparison
- **Stable**: Answer won't change over time

#### Output Format

Create an XML file with this structure:

```xml
<evaluation>
  <qa_pair>
    <question>Find discussions about AI model launches with animal codenames...</question>
    <answer>3</answer>
  </qa_pair>
  <!-- More qa_pairs... -->
</evaluation>
```

## TypeScript Example

```typescript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const server = new Server({
  name: "my-mcp-server",
  version: "1.0.0",
});

// Register a tool
server.registerTool({
  name: "get_weather",
  description: "Get current weather for a city",
  inputSchema: z.object({
    city: z.string().describe("City name"),
    units: z.enum(["metric", "imperial"]).default("metric"),
  }),
  outputSchema: z.object({
    temperature: z.number(),
    conditions: z.string(),
  }),
  annotations: {
    readOnlyHint: true,
  },
  handler: async ({ city, units }) => {
    const weather = await fetchWeather(city, units);
    return {
      content: [{ type: "text", text: JSON.stringify(weather) }],
      structuredContent: weather,
    };
  },
});

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);
```

## Best Practices

1. **Comprehensive API coverage** over specialized workflow tools
2. **Consistent naming** with prefixes (e.g., `github_`, `slack_`)
3. **Pagination support** for list operations
4. **Actionable errors** with specific suggestions
5. **Type safety** with Zod schemas
6. **Thorough testing** with MCP Inspector

## References

- [MCP Protocol Specification](https://modelcontextprotocol.io)
- [TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk)
- [Python SDK](https://github.com/modelcontextprotocol/python-sdk)
