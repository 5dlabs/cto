# Cursor CLI

Cursor CLI lets you interact with AI agents directly from your terminal to write, review, and modify code. Whether you prefer an interactive terminal interface or print automation for scripts and CI pipelines, the CLI provides powerful coding assistance right where you work.

## Getting started

```
# Install
curl https://cursor.com/install -fsS | bash

# Run interactive session
cursor-agent
```

Cursor CLI is currently in beta, we'd love your feedback on it!
## Interactive mode

Start a conversational session with the agent to describe your goals, review proposed changes, and approve commands:

```
# Start interactive session
cursor-agent

# Start with initial prompt
cursor-agent "refactor the auth module to use JWT tokens"
```

## Non-interactive mode

Use print mode for non-interactive scenarios like scripts, CI pipelines, or automation:

```
# Run with specific prompt and model
cursor-agent -p "find and fix performance issues" --model "gpt-5"

# Use with git changes included for review
cursor-agent -p "review these changes for security issues" --output-format text
```

## Sessions

Resume previous conversations to maintain context across multiple interactions:

```
# List all previous chats
cursor-agent ls

# Resume latest conversation
cursor-agent resume

# Resume specific conversation
cursor-agent --resume="chat-id-here"
```