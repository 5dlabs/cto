# Using Agent in CLI

## Prompting

Stating intent clearly is recommended for the best results. For example, you can use the prompt "do not write any code" to ensure that the agent won't edit any files. This is generally helpful when planning tasks before implementing them.

Agent currently has tools for file operations, searching, and running shell commands. More tools are being added, similar to the IDE agent.

## MCP

Agent supports [MCP (Model Context Protocol)](/docs/context/mcp/directory) for extended functionality and integrations. The CLI will automatically detect and respect your `mcp.json` configuration file, enabling the same MCP servers and tools that you've configured for the IDE.

## Rules

The CLI agent supports the same [rules system](/docs/context/rules) as the IDE. You can create rules in the `.cursor/rules` directory to provide context and guidance to the agent. These rules will be automatically loaded and applied based on their configuration, allowing you to customize the agent's behavior for different parts of your project or specific file types.

The CLI also reads `AGENTS.md` and `CLAUDE.md` at the project root (if
present) and applies them as rules alongside `.cursor/rules`.

## Working with Agent

### Navigation

Previous messages can be accessed using arrow up (ArrowUp) where you can cycle through them.

### Review

Review changes with Ctrl+R. Press i to add follow-up instructions. Use ArrowUp/ArrowDown to scroll, and ArrowLeft/ArrowRight to switch files.

### Selecting context

Select files and folders to include in context with @. Free up space in the context window by running `/compress`. See [Summarization](/docs/agent/chat/summarization) for details.

## History

Continue from an existing thread with `--resume [thread id]` to load prior context.

To resume the most recent conversation, use `cursor-agent resume`.

You can also run `cursor-agent ls` to see a list of previous conversations.

## Command approval

Before running terminal commands, CLI will ask you to approve (y) or reject (n) execution.

## Non-interactive mode

Use `-p` or `--print` to run Agent in non-interactive mode. This will print the response to the console.

With non-interactive mode, you can invoke Agent in a non-interactive way. This allows you to integrate it in scripts, CI pipelines, etc.

You can combine this with `--output-format` to control how the output is formatted. For example, use `--output-format json` for structured output that's easier to parse in scripts, or `--output-format text` for plain text output.

Cursor has full write access in non-interactive mode.