# Permissions

Configure what the agent is allowed to do using permission tokens in your CLI configuration. Permissions are set in `~/.cursor/cli-config.json` (global) or `<project>/.cursor/cli.json` (project-specific).

## Permission types

### Shell commands

**Format:** `Shell(commandBase)`

Controls access to shell commands. The `commandBase` is the first token in the command line.

ExampleDescription`Shell(ls)`Allow running `ls` commands`Shell(git)`Allow any `git` subcommand`Shell(npm)`Allow npm package manager commands`Shell(rm)`Deny destructive file removal (commonly in `deny`)
### File reads

**Format:** `Read(pathOrGlob)`

Controls read access to files and directories. Supports glob patterns.

ExampleDescription`Read(src/**/*.ts)`Allow reading TypeScript files in `src``Read(**/*.md)`Allow reading markdown files anywhere`Read(.env*)`Deny reading environment files`Read(/etc/passwd)`Deny reading system files
### File writes

**Format:** `Write(pathOrGlob)`

Controls write access to files and directories. Supports glob patterns. When using in print mode, `--force` is required to write files.

ExampleDescription`Write(src/**)`Allow writing to any file under `src``Write(package.json)`Allow modifying package.json`Write(**/*.key)`Deny writing private key files`Write(**/.env*)`Deny writing environment files
## Configuration

Add permissions to the `permissions` object in your CLI configuration file:

```
{
  "permissions": {
    "allow": [
      "Shell(ls)",
      "Shell(git)",
      "Read(src/**/*.ts)",
      "Write(package.json)"
    ],
    "deny": ["Shell(rm)", "Read(.env*)", "Write(**/*.key)"]
  }
}
```

## Pattern matching

- Glob patterns use `**`, `*`, and `?` wildcards
- Relative paths are scoped to the current workspace
- Absolute paths can target files outside the project
- Deny rules take precedence over allow rules