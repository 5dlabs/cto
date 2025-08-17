
# CRITICAL: Complete Documentation Generation Task


**🚨 MANDATORY COMPLETION REQUIREMENT 🚨**
You MUST complete ALL steps in this process. Partial completion is not acceptable. This is especially critical for large projects with many tasks.

## Task Overview
Generate comprehensive documentation for **ALL Task Master tasks**.

**If this is a large project with many tasks (10+ tasks), you MUST:**
- Process ALL tasks without stopping
- Show progress updates as you work through each task
- Complete the ENTIRE git workflow including PR creation
- Do NOT stop partway through - finish everything

## Required Process

### Step 1: Context Analysis (REQUIRED)
1. Read CLAUDE.md for project context and standards
2. **Use individual task files:**
   - Individual task files have been pre-copied to `.taskmaster/docs/task-{id}/task.txt`
   - Each `task.txt` contains complete task information including subtasks and implementation details
3. Review architecture.md and prd.txt for context
4. **For large projects: Announce total task count and confirm you will process ALL of them**

**🎯 IMPORTANT: Use individual `task.txt` files for each task**
- Individual task files are available at: `.taskmaster/docs/task-{id}/task.txt`
- Example: task 1 → `.taskmaster/docs/task-1/task.txt`, task 15 → `.taskmaster/docs/task-15/task.txt`
- These files contain complete task information including subtasks and implementation details
- This approach allows efficient processing of large projects with many tasks

### Step 2: Documentation Generation (MANDATORY FOR ALL TASKS)
**IMPORTANT: SKIP TASKS THAT ALREADY HAVE COMPLETE DOCUMENTATION**

Before processing any task, check if ALL required files already exist:
- `task.md` 
- `prompt.md`
- `acceptance-criteria.md`
- `client-config.json`
- `toolman-guide.md`

If ALL five files exist and have substantial content (not just stubs), SKIP that task to save tokens and time.

**YOU MUST CREATE DOCUMENTATION FOR EVERY TASK THAT NEEDS IT. DO NOT SKIP ANY INCOMPLETE TASKS.**

For each task that needs documentation (process ALL incomplete tasks, no exceptions):
- `task.md` - Comprehensive task overview and implementation guide
- `prompt.md` - Autonomous prompt for AI agents
- `acceptance-criteria.md` - Clear acceptance criteria and test cases
- `client-config.json` - MCP client configuration for code implementation agents
- `toolman-guide.md` - Task-specific guide explaining which tools to use and when

**Progress Requirements:**
- Announce each task as you start it: "📝 Processing Task [ID]: [Title]"
- Confirm completion of each task: "✅ Completed Task [ID]"
- **For large projects: Provide periodic updates (every 5 tasks): "Progress: [X] of [Y] tasks completed"**

Place all documentation in `.taskmaster/docs/task-{id}/` directories.

### Step 2.1: Toolman Configuration Generation (MANDATORY)

**🔧 CRITICAL: Generate Simple MCP Client Configuration**

For each task, you MUST create a `client-config.json` file that is a **SIMPLE CLIENT CONFIGURATION** - NOT a task specification document.

**🚨 IMPORTANT: This is a CLIENT CONFIG FILE, not a detailed task document!**

**REQUIRED FORMAT (EXACT STRUCTURE):**
```json
{
  "remoteTools": [
    "tool_name_1",
    "tool_name_2"
  ],
  "localServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "tools": ["read_file", "write_file", "list_directory", "create_directory", "edit_file"],
      "workingDirectory": "project_root"
    }
  }
}
```

**❌ DO NOT GENERATE:** 
- Task specifications with `task_id`, `task_name`, `task_type` 
- Complex metadata like `priority`, `status`, `estimated_duration`
- Detailed planning documents with `required_tools`, `recommended_tools`
- Nested configuration objects with `tool_configuration`, `technical_requirements`
- Any fields other than `remoteTools` and `localServers`

**✅ CORRECT APPROACH:**
1. **Read the Catalog Below**: Examine available tools and capabilities
2. **Select Remote Tools**: Choose specific remote tool names from the catalog for `remoteTools` array
3. **Configure Local Servers**: Use exact command/args from catalog for local servers (usually just filesystem)
4. **Keep It Simple**: Only include `remoteTools` and `localServers` fields - nothing else!

**Example for Infrastructure Task:**
```json
{
  "remoteTools": [
    "kubernetes_listResources",
    "kubernetes_getResource",
    "kubernetes_createResource"
  ],
  "localServers": {
    "filesystem": {
      "command": "npx", 
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "tools": ["read_file", "write_file", "edit_file", "list_directory"],
      "workingDirectory": "project_root"
    }
  }
}
```

**Available Tool Catalog:**
The complete toolman catalog is provided below. Use ONLY tool names that appear in this catalog.

### Step 2.2: Toolman Guide Generation (MANDATORY)

**📖 CRITICAL: Generate Task-Specific Tool Usage Guide**

For each task, you MUST create a comprehensive `toolman-guide.md` file that explains how and when to use the selected tools based on the catalog information.

**Guide Generation Requirements:**
1. **Use Catalog Descriptions**: Leverage the detailed tool descriptions, use cases, and categories from the catalog below
2. **Document Tool Arguments**: Clearly document all arguments, parameters, and configuration options for each selected tool based on catalog information
3. **Task-Specific Context**: Explain how each selected tool applies specifically to this task's requirements
4. **When to Use**: Provide clear guidance on when to use each tool during task implementation
5. **How to Use**: Include practical examples and best practices based on catalog information
6. **Tool Relationships**: Explain how tools work together and their sequence of use
7. **Common Patterns**: Reference the selection guidelines and patterns from the catalog

**Guide Structure:**
- **Overview**: Brief explanation of the tool selection for this task
- **Core Tools**: Primary tools with detailed usage instructions
- **Supporting Tools**: Secondary tools and their specific use cases
- **Implementation Flow**: Suggested order and workflow for using the tools
- **Best Practices**: Task-specific tips and recommendations
- **Troubleshooting**: Common issues and solutions based on tool capabilities

**Content Source**: Use the rich catalog information below as your primary source for tool descriptions, capabilities, use cases, and implementation guidance. The catalog contains everything you need to create comprehensive, actionable tool guides.

---



# Available Tools Catalog

**Total Tools Available**: 62  
**Generated**: 1755398736

Use this catalog to select appropriate tools for each task based on implementation requirements.

## Local Servers

Local servers run within the agent's environment and provide file system and local operations.

### filesystem Server

**Description**: File system operations for reading, writing, and managing files  
**Command**: `npx`  
**Arguments**: `-y` `@modelcontextprotocol/server-filesystem` `/tmp`  
**Working Directory**: /tmp

**Available Tools**:
- **`read_file`** (file-operations)
  - **Description**: Read the complete contents of a file as text. DEPRECATED: Use read_text_file instead.
  - **Use Cases**: read the complete contents of a file as text. deprecated: use read_text_file instead.
- **`list_directory_with_sizes`** (file-operations)
  - **Description**: Get a detailed listing of all files and directories in a specified path, including sizes. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is useful for understanding directory structure and finding specific files within a directory. Only works within allowed directories.
  - **Use Cases**: retrieving information
- **`edit_file`** (version-control)
  - **Description**: Make line-based edits to a text file. Each edit replaces exact line sequences with new content. Returns a git-style diff showing the changes made. Only works within allowed directories.
  - **Use Cases**: make line-based edits to a text file. each edit replaces exact line sequences with new content. returns a git-style diff showing the changes made. only works within allowed directories.
- **`search_files`** (search)
  - **Description**: Recursively search for files and directories matching a pattern. Searches through all subdirectories from the starting path. The search is case-insensitive and matches partial names. Returns full paths to all matching items. Great for finding files when you don&#x27;t know their exact location. Only searches within allowed directories.
  - **Use Cases**: finding information
- **`list_allowed_directories`** (file-operations)
  - **Description**: Returns the list of root directories that this server is allowed to access. Use this to understand which directories are available before trying to access files. 
  - **Use Cases**: retrieving information
- **`read_media_file`** (file-operations)
  - **Description**: Read an image or audio file. Returns the base64 encoded data and MIME type. Only works within allowed directories.
  - **Use Cases**: read an image or audio file. returns the base64 encoded data and mime type. only works within allowed directories.
- **`read_multiple_files`** (file-operations)
  - **Description**: Read the contents of multiple files simultaneously. This is more efficient than reading files one by one when you need to analyze or compare multiple files. Each file&#x27;s content is returned with its path as a reference. Failed reads for individual files won&#x27;t stop the entire operation. Only works within allowed directories.
  - **Use Cases**: read the contents of multiple files simultaneously. this is more efficient than reading files one by one when you need to analyze or compare multiple files. each file&#x27;s content is returned with its path as a reference. failed reads for individual files won&#x27;t stop the entire operation. only works within allowed directories.
- **`directory_tree`** (file-operations)
  - **Description**: Get a recursive tree view of files and directories as a JSON structure. Each entry includes &#x27;name&#x27;, &#x27;type&#x27; (file/directory), and &#x27;children&#x27; for directories. Files have no children array, while directories always have a children array (which may be empty). The output is formatted with 2-space indentation for readability. Only works within allowed directories.
  - **Use Cases**: retrieving information
- **`read_text_file`** (file-operations)
  - **Description**: Read the complete contents of a file from the file system as text. Handles various text encodings and provides detailed error messages if the file cannot be read. Use this tool when you need to examine the contents of a single file. Use the &#x27;head&#x27; parameter to read only the first N lines of a file, or the &#x27;tail&#x27; parameter to read only the last N lines of a file. Operates on the file as text regardless of extension. Only works within allowed directories.
  - **Use Cases**: read the complete contents of a file from the file system as text. handles various text encodings and provides detailed error messages if the file cannot be read. use this tool when you need to examine the contents of a single file. use the &#x27;head&#x27; parameter to read only the first n lines of a file, or the &#x27;tail&#x27; parameter to read only the last n lines of a file. operates on the file as text regardless of extension. only works within allowed directories.
- **`list_directory`** (file-operations)
  - **Description**: Get a detailed listing of all files and directories in a specified path. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is essential for understanding directory structure and finding specific files within a directory. Only works within allowed directories.
  - **Use Cases**: retrieving information
- **`move_file`** (file-operations)
  - **Description**: Move or rename files and directories. Can move files between directories and rename them in a single operation. If the destination exists, the operation will fail. Works across different directories and can be used for simple renaming within the same directory. Both source and destination must be within allowed directories.
  - **Use Cases**: move or rename files and directories. can move files between directories and rename them in a single operation. if the destination exists, the operation will fail. works across different directories and can be used for simple renaming within the same directory. both source and destination must be within allowed directories.
- **`write_file`** (file-operations)
  - **Description**: Create a new file or completely overwrite an existing file with new content. Use with caution as it will overwrite existing files without warning. Handles text content with proper encoding. Only works within allowed directories.
  - **Use Cases**: creating resources
- **`create_directory`** (file-operations)
  - **Description**: Create a new directory or ensure a directory exists. Can create multiple nested directories in one operation. If the directory already exists, this operation will succeed silently. Perfect for setting up directory structures for projects or ensuring required paths exist. Only works within allowed directories.
  - **Use Cases**: creating resources
- **`get_file_info`** (file-operations)
  - **Description**: Retrieve detailed metadata about a file or directory. Returns comprehensive information including size, creation time, last modified time, permissions, and type. This tool is perfect for understanding file characteristics without reading the actual content. Only works within allowed directories.
  - **Use Cases**: retrieving information


## Remote Tools

Remote tools are available via MCP servers running in the cluster.

### brave-search Server

**Description**: Web search using Brave Search API  
**Endpoint**: stdio

**Available Tools**:
- **`brave_web_search`** (search)
  - **Description**: Performs a web search using the Brave Search API, ideal for general queries, news, articles, and online content. Use this for broad information gathering, recent events, or when you need diverse web sources. Supports pagination, content filtering, and freshness controls. Maximum 20 results per request, with offset for pagination. 
  - **Use Cases**: finding information
- **`brave_local_search`** (search)
  - **Description**: Searches for local businesses and places using Brave&#x27;s Local Search API. Best for queries related to physical locations, businesses, restaurants, services, etc. Returns detailed information including:
- Business names and addresses
- Ratings and review counts
- Phone numbers and opening hours
Use this when the query implies &#x27;near me&#x27; or mentions specific locations. Automatically falls back to web search if no local results are found.
  - **Use Cases**: creating resources, finding information

### context7 Server

**Description**: Up-to-date library documentation and code examples  
**Endpoint**: stdio

**Available Tools**:
- **`resolve-library-id`** (memory)
  - **Description**: Resolves a package/product name to a Context7-compatible library ID and returns a list of matching libraries.

You MUST call this function before &#x27;get-library-docs&#x27; to obtain a valid Context7-compatible library ID UNLESS the user explicitly provides a library ID in the format &#x27;/org/project&#x27; or &#x27;/org/project/version&#x27; in their query.

Selection Process:
1. Analyze the query to understand what library/package the user is looking for
2. Return the most relevant match based on:
- Name similarity to the query (exact matches prioritized)
- Description relevance to the query&#x27;s intent
- Documentation coverage (prioritize libraries with higher Code Snippet counts)
- Trust score (consider libraries with scores of 7-10 more authoritative)

Response Format:
- Return the selected library ID in a clearly marked section
- Provide a brief explanation for why this library was chosen
- If multiple good matches exist, acknowledge this but proceed with the most relevant one
- If no good matches exist, clearly state this and suggest query refinements

For ambiguous queries, request clarification before proceeding with a best-guess match.
  - **Use Cases**: retrieving information
- **`get-library-docs`** (version-control)
  - **Description**: Fetches up-to-date documentation for a library. You must call &#x27;resolve-library-id&#x27; first to obtain the exact Context7-compatible library ID required to use this tool, UNLESS the user explicitly provides a library ID in the format &#x27;/org/project&#x27; or &#x27;/org/project/version&#x27; in their query.
  - **Use Cases**: retrieving information

### kubernetes Server

**Description**: Kubernetes cluster management and Helm operations  
**Endpoint**: http://k8s-mcp-k8s-mcp-server.mcp.svc.cluster.local:8080/sse

**Available Tools**:
- **`describeResource`** (kubernetes)
  - **Description**: Describe a resource in the Kubernetes cluster based on given kind and name
  - **Use Cases**: describe a resource in the kubernetes cluster based on given kind and name
- **`helmInstall`** (kubernetes)
  - **Description**: Install a Helm chart to the Kubernetes cluster
  - **Use Cases**: install a helm chart to the kubernetes cluster
- **`getEvents`** (kubernetes)
  - **Description**: Get events in the Kubernetes cluster
  - **Use Cases**: retrieving information
- **`helmList`** (kubernetes)
  - **Description**: List all Helm releases in the cluster or a specific namespace
  - **Use Cases**: retrieving information
- **`helmUpgrade`** (kubernetes)
  - **Description**: Upgrade an existing Helm release
  - **Use Cases**: upgrade an existing helm release
- **`getAPIResources`** (kubernetes)
  - **Description**: Get all API resources in the Kubernetes cluster
CreateGetAPIResourcesTool creates a tool for getting API resources
GetAPIResourcesHandler handles the getAPIResources tool
It retrieves the API resources from the Kubernetes cluster
and returns them as a response.
e.g. &#x27;beta&#x27; or &#x27;prod&#x27;.
The function returns a mcp.CallToolResult containing the API resources
or an error if the operation fails.
The function also handles the inclusion of namespace scoped
and cluster scoped resources based on the provided parameters.
The function is designed to be used as a handler for the mcp tool
  - **Use Cases**: retrieving information, creating resources
- **`helmRollback`** (kubernetes)
  - **Description**: Rollback a Helm release to a previous revision
  - **Use Cases**: rollback a helm release to a previous revision
- **`createResource`** (kubernetes)
  - **Description**: Create a resource in the Kubernetes cluster
  - **Use Cases**: creating resources
- **`getPodsLogs`** (kubernetes)
  - **Description**: Get logs of a specific pod in the Kubernetes cluster
  - **Use Cases**: retrieving information
- **`listResources`** (kubernetes)
  - **Description**: List all resources in the Kubernetes cluster of a specific type
  - **Use Cases**: retrieving information
- **`helmRepoAdd`** (kubernetes)
  - **Description**: Add a Helm repository
  - **Use Cases**: creating resources
- **`getResource`** (kubernetes)
  - **Description**: Get a specific resource in the Kubernetes cluster
  - **Use Cases**: retrieving information
- **`helmUninstall`** (kubernetes)
  - **Description**: Uninstall a Helm release from the Kubernetes cluster
  - **Use Cases**: uninstall a helm release from the kubernetes cluster
- **`getPodMetrics`** (memory)
  - **Description**: Get CPU and Memory metrics for a specific pod
  - **Use Cases**: retrieving information
- **`helmGet`** (kubernetes)
  - **Description**: Get details of a specific Helm release
  - **Use Cases**: retrieving information
- **`getNodeMetrics`** (kubernetes)
  - **Description**: Get resource usage of a specific node in the Kubernetes cluster
  - **Use Cases**: retrieving information
- **`helmRepoList`** (kubernetes)
  - **Description**: List all Helm repositories
  - **Use Cases**: retrieving information
- **`helmHistory`** (kubernetes)
  - **Description**: Get the history of a Helm release
  - **Use Cases**: retrieving information

### memory Server

**Description**: Persistent memory and knowledge graph for long-term information retention  
**Endpoint**: stdio

**Available Tools**:
- **`delete_entities`** (memory)
  - **Description**: Delete multiple entities and their associated relations from the knowledge graph
  - **Use Cases**: removing resources
- **`delete_relations`** (memory)
  - **Description**: Delete multiple relations from the knowledge graph
  - **Use Cases**: removing resources
- **`delete_observations`** (memory)
  - **Description**: Delete specific observations from entities in the knowledge graph
  - **Use Cases**: removing resources
- **`create_relations`** (memory)
  - **Description**: Create multiple new relations between entities in the knowledge graph. Relations should be in active voice
  - **Use Cases**: creating resources
- **`add_observations`** (memory)
  - **Description**: Add new observations to existing entities in the knowledge graph
  - **Use Cases**: creating resources
- **`open_nodes`** (memory)
  - **Description**: Open specific nodes in the knowledge graph by their names
  - **Use Cases**: open specific nodes in the knowledge graph by their names
- **`search_nodes`** (search)
  - **Description**: Search for nodes in the knowledge graph based on a query
  - **Use Cases**: finding information
- **`create_entities`** (memory)
  - **Description**: Create multiple new entities in the knowledge graph
  - **Use Cases**: creating resources
- **`read_graph`** (memory)
  - **Description**: Read the entire knowledge graph
  - **Use Cases**: read the entire knowledge graph

### rustdocs Server

**Description**: Rust documentation MCP server  
**Endpoint**: http://rustdocs-mcp-rust-docs-mcp-server.mcp.svc.cluster.local:3000/sse

**Available Tools**:
- **`add_crate`** (general)
  - **Description**: Add or update a crate configuration
  - **Use Cases**: creating resources, updating resources
- **`check_crate_status`** (general)
  - **Description**: Check the status of crate population jobs
  - **Use Cases**: check the status of crate population jobs
- **`query_rust_docs`** (search)
  - **Description**: Query documentation for a specific Rust crate using semantic search and LLM summarization.
  - **Use Cases**: finding information
- **`add_crates`** (general)
  - **Description**: Add or update multiple crate configurations
  - **Use Cases**: creating resources, updating resources
- **`list_crates`** (general)
  - **Description**: List all configured crates
  - **Use Cases**: retrieving information
- **`remove_crate`** (general)
  - **Description**: Remove a crate configuration
  - **Use Cases**: removing resources

### solana Server

**Description**: Solana blockchain development tools  
**Endpoint**: https://mcp.solana.com/mcp

**Available Tools**:
- **`Solana_Documentation_Search`** (search)
  - **Description**: Search documentation across the Solana ecosystem to get the most up to date information.
  - **Use Cases**: retrieving information, finding information
- **`Ask_Solana_Anchor_Framework_Expert`** (general)
  - **Description**: Ask questions about developing on Solana with the Anchor Framework.
  - **Use Cases**: ask questions about developing on solana with the anchor framework.
- **`Solana_Expert__Ask_For_Help`** (general)
  - **Description**: A Solana expert that can answer questions about Solana development.
  - **Use Cases**: a solana expert that can answer questions about solana development.

### terraform Server

**Description**: Terraform Registry API integration  
**Endpoint**: stdio

**Available Tools**:
- **`get_provider_details`** (search)
  - **Description**: Fetches up-to-date documentation for a specific service from a Terraform provider. 
You must call &#x27;search_providers&#x27; tool first to obtain the exact tfprovider-compatible provider_doc_id required to use this tool.
  - **Use Cases**: retrieving information, finding information
- **`search_modules`** (search)
  - **Description**: Resolves a Terraform module name to obtain a compatible module_id for the get_module_details tool and returns a list of matching Terraform modules.
You MUST call this function before &#x27;get_module_details&#x27; to obtain a valid and compatible module_id.
When selecting the best match, consider the following:
	- Name similarity to the query
	- Description relevance
	- Verification status (verified)
	- Download counts (popularity)
Return the selected module_id and explain your choice. If there are multiple good matches, mention this but proceed with the most relevant one.
If no modules were found, reattempt the search with a new moduleName query.
  - **Use Cases**: retrieving information, finding information
- **`get_policy_details`** (search)
  - **Description**: Fetches up-to-date documentation for a specific policy from the Terraform registry. You must call &#x27;search_policies&#x27; first to obtain the exact terraform_policy_id required to use this tool.
  - **Use Cases**: retrieving information, finding information
- **`search_policies`** (search)
  - **Description**: Searches for Terraform policies based on a query string.
This tool returns a list of matching policies, which can be used to retrieve detailed policy information using the &#x27;get_policy_details&#x27; tool.
You MUST call this function before &#x27;get_policy_details&#x27; to obtain a valid terraform_policy_id.
When selecting the best match, consider the following:
	- Name similarity to the query
	- Title relevance
	- Verification status (verified)
	- Download counts (popularity)
Return the selected policyID and explain your choice. If there are multiple good matches, mention this but proceed with the most relevant one.
If no policies were found, reattempt the search with a new policy_query.
  - **Use Cases**: retrieving information, finding information
- **`search_providers`** (search)
  - **Description**: This tool retrieves a list of potential documents based on the service_slug and provider_data_type provided.
You MUST call this function before &#x27;get_provider_details&#x27; to obtain a valid tfprovider-compatible provider_doc_id.
Use the most relevant single word as the search query for service_slug, if unsure about the service_slug, use the provider_name for its value.
When selecting the best match, consider the following:
	- Title similarity to the query
	- Category relevance
Return the selected provider_doc_id and explain your choice.
If there are multiple good matches, mention this but proceed with the most relevant one.
  - **Use Cases**: retrieving information, finding information
- **`get_latest_module_version`** (version-control)
  - **Description**: Fetches the latest version of a Terraform module from the public registry
  - **Use Cases**: retrieving information
- **`get_latest_provider_version`** (version-control)
  - **Description**: Fetches the latest version of a Terraform provider from the public registry
  - **Use Cases**: retrieving information
- **`get_module_details`** (search)
  - **Description**: Fetches up-to-date documentation on how to use a Terraform module. You must call &#x27;search_modules&#x27; first to obtain the exact valid and compatible module_id required to use this tool.
  - **Use Cases**: retrieving information, finding information


## Tool Selection Guidelines

### How to Select Tools

1. **Analyze Task Requirements**: Read the task description and implementation details carefully
2. **Match Categories**: Look for tools whose categories align with your task needs:
   - **file-operations**: Read the complete contents of a file as text. DEPRECATED: Use read_text_file instead.
   - **file-operations**: Get a detailed listing of all files and directories in a specified path, including sizes. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is useful for understanding directory structure and finding specific files within a directory. Only works within allowed directories.
   - **version-control**: Make line-based edits to a text file. Each edit replaces exact line sequences with new content. Returns a git-style diff showing the changes made. Only works within allowed directories.
   - **search**: Recursively search for files and directories matching a pattern. Searches through all subdirectories from the starting path. The search is case-insensitive and matches partial names. Returns full paths to all matching items. Great for finding files when you don&#x27;t know their exact location. Only searches within allowed directories.
   - **file-operations**: Returns the list of root directories that this server is allowed to access. Use this to understand which directories are available before trying to access files. 
   - **file-operations**: Read an image or audio file. Returns the base64 encoded data and MIME type. Only works within allowed directories.
   - **file-operations**: Read the contents of multiple files simultaneously. This is more efficient than reading files one by one when you need to analyze or compare multiple files. Each file&#x27;s content is returned with its path as a reference. Failed reads for individual files won&#x27;t stop the entire operation. Only works within allowed directories.
   - **file-operations**: Get a recursive tree view of files and directories as a JSON structure. Each entry includes &#x27;name&#x27;, &#x27;type&#x27; (file/directory), and &#x27;children&#x27; for directories. Files have no children array, while directories always have a children array (which may be empty). The output is formatted with 2-space indentation for readability. Only works within allowed directories.
   - **file-operations**: Read the complete contents of a file from the file system as text. Handles various text encodings and provides detailed error messages if the file cannot be read. Use this tool when you need to examine the contents of a single file. Use the &#x27;head&#x27; parameter to read only the first N lines of a file, or the &#x27;tail&#x27; parameter to read only the last N lines of a file. Operates on the file as text regardless of extension. Only works within allowed directories.
   - **file-operations**: Get a detailed listing of all files and directories in a specified path. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is essential for understanding directory structure and finding specific files within a directory. Only works within allowed directories.
   - **file-operations**: Move or rename files and directories. Can move files between directories and rename them in a single operation. If the destination exists, the operation will fail. Works across different directories and can be used for simple renaming within the same directory. Both source and destination must be within allowed directories.
   - **file-operations**: Create a new file or completely overwrite an existing file with new content. Use with caution as it will overwrite existing files without warning. Handles text content with proper encoding. Only works within allowed directories.
   - **file-operations**: Create a new directory or ensure a directory exists. Can create multiple nested directories in one operation. If the directory already exists, this operation will succeed silently. Perfect for setting up directory structures for projects or ensuring required paths exist. Only works within allowed directories.
   - **file-operations**: Retrieve detailed metadata about a file or directory. Returns comprehensive information including size, creation time, last modified time, permissions, and type. This tool is perfect for understanding file characteristics without reading the actual content. Only works within allowed directories.
   - **search**: Performs a web search using the Brave Search API, ideal for general queries, news, articles, and online content. Use this for broad information gathering, recent events, or when you need diverse web sources. Supports pagination, content filtering, and freshness controls. Maximum 20 results per request, with offset for pagination. 
   - **search**: Searches for local businesses and places using Brave&#x27;s Local Search API. Best for queries related to physical locations, businesses, restaurants, services, etc. Returns detailed information including:
- Business names and addresses
- Ratings and review counts
- Phone numbers and opening hours
Use this when the query implies &#x27;near me&#x27; or mentions specific locations. Automatically falls back to web search if no local results are found.
   - **memory**: Resolves a package/product name to a Context7-compatible library ID and returns a list of matching libraries.

You MUST call this function before &#x27;get-library-docs&#x27; to obtain a valid Context7-compatible library ID UNLESS the user explicitly provides a library ID in the format &#x27;/org/project&#x27; or &#x27;/org/project/version&#x27; in their query.

Selection Process:
1. Analyze the query to understand what library/package the user is looking for
2. Return the most relevant match based on:
- Name similarity to the query (exact matches prioritized)
- Description relevance to the query&#x27;s intent
- Documentation coverage (prioritize libraries with higher Code Snippet counts)
- Trust score (consider libraries with scores of 7-10 more authoritative)

Response Format:
- Return the selected library ID in a clearly marked section
- Provide a brief explanation for why this library was chosen
- If multiple good matches exist, acknowledge this but proceed with the most relevant one
- If no good matches exist, clearly state this and suggest query refinements

For ambiguous queries, request clarification before proceeding with a best-guess match.
   - **version-control**: Fetches up-to-date documentation for a library. You must call &#x27;resolve-library-id&#x27; first to obtain the exact Context7-compatible library ID required to use this tool, UNLESS the user explicitly provides a library ID in the format &#x27;/org/project&#x27; or &#x27;/org/project/version&#x27; in their query.
   - **kubernetes**: Describe a resource in the Kubernetes cluster based on given kind and name
   - **kubernetes**: Install a Helm chart to the Kubernetes cluster
   - **kubernetes**: Get events in the Kubernetes cluster
   - **kubernetes**: List all Helm releases in the cluster or a specific namespace
   - **kubernetes**: Upgrade an existing Helm release
   - **kubernetes**: Get all API resources in the Kubernetes cluster
CreateGetAPIResourcesTool creates a tool for getting API resources
GetAPIResourcesHandler handles the getAPIResources tool
It retrieves the API resources from the Kubernetes cluster
and returns them as a response.
e.g. &#x27;beta&#x27; or &#x27;prod&#x27;.
The function returns a mcp.CallToolResult containing the API resources
or an error if the operation fails.
The function also handles the inclusion of namespace scoped
and cluster scoped resources based on the provided parameters.
The function is designed to be used as a handler for the mcp tool
   - **kubernetes**: Rollback a Helm release to a previous revision
   - **kubernetes**: Create a resource in the Kubernetes cluster
   - **kubernetes**: Get logs of a specific pod in the Kubernetes cluster
   - **kubernetes**: List all resources in the Kubernetes cluster of a specific type
   - **kubernetes**: Add a Helm repository
   - **kubernetes**: Get a specific resource in the Kubernetes cluster
   - **kubernetes**: Uninstall a Helm release from the Kubernetes cluster
   - **memory**: Get CPU and Memory metrics for a specific pod
   - **kubernetes**: Get details of a specific Helm release
   - **kubernetes**: Get resource usage of a specific node in the Kubernetes cluster
   - **kubernetes**: List all Helm repositories
   - **kubernetes**: Get the history of a Helm release
   - **memory**: Delete multiple entities and their associated relations from the knowledge graph
   - **memory**: Delete multiple relations from the knowledge graph
   - **memory**: Delete specific observations from entities in the knowledge graph
   - **memory**: Create multiple new relations between entities in the knowledge graph. Relations should be in active voice
   - **memory**: Add new observations to existing entities in the knowledge graph
   - **memory**: Open specific nodes in the knowledge graph by their names
   - **search**: Search for nodes in the knowledge graph based on a query
   - **memory**: Create multiple new entities in the knowledge graph
   - **memory**: Read the entire knowledge graph
   - **general**: Add or update a crate configuration
   - **general**: Check the status of crate population jobs
   - **search**: Query documentation for a specific Rust crate using semantic search and LLM summarization.
   - **general**: Add or update multiple crate configurations
   - **general**: List all configured crates
   - **general**: Remove a crate configuration
   - **search**: Search documentation across the Solana ecosystem to get the most up to date information.
   - **general**: Ask questions about developing on Solana with the Anchor Framework.
   - **general**: A Solana expert that can answer questions about Solana development.
   - **search**: Fetches up-to-date documentation for a specific service from a Terraform provider. 
You must call &#x27;search_providers&#x27; tool first to obtain the exact tfprovider-compatible provider_doc_id required to use this tool.
   - **search**: Resolves a Terraform module name to obtain a compatible module_id for the get_module_details tool and returns a list of matching Terraform modules.
You MUST call this function before &#x27;get_module_details&#x27; to obtain a valid and compatible module_id.
When selecting the best match, consider the following:
	- Name similarity to the query
	- Description relevance
	- Verification status (verified)
	- Download counts (popularity)
Return the selected module_id and explain your choice. If there are multiple good matches, mention this but proceed with the most relevant one.
If no modules were found, reattempt the search with a new moduleName query.
   - **search**: Fetches up-to-date documentation for a specific policy from the Terraform registry. You must call &#x27;search_policies&#x27; first to obtain the exact terraform_policy_id required to use this tool.
   - **search**: Searches for Terraform policies based on a query string.
This tool returns a list of matching policies, which can be used to retrieve detailed policy information using the &#x27;get_policy_details&#x27; tool.
You MUST call this function before &#x27;get_policy_details&#x27; to obtain a valid terraform_policy_id.
When selecting the best match, consider the following:
	- Name similarity to the query
	- Title relevance
	- Verification status (verified)
	- Download counts (popularity)
Return the selected policyID and explain your choice. If there are multiple good matches, mention this but proceed with the most relevant one.
If no policies were found, reattempt the search with a new policy_query.
   - **search**: This tool retrieves a list of potential documents based on the service_slug and provider_data_type provided.
You MUST call this function before &#x27;get_provider_details&#x27; to obtain a valid tfprovider-compatible provider_doc_id.
Use the most relevant single word as the search query for service_slug, if unsure about the service_slug, use the provider_name for its value.
When selecting the best match, consider the following:
	- Title similarity to the query
	- Category relevance
Return the selected provider_doc_id and explain your choice.
If there are multiple good matches, mention this but proceed with the most relevant one.
   - **version-control**: Fetches the latest version of a Terraform module from the public registry
   - **version-control**: Fetches the latest version of a Terraform provider from the public registry
   - **search**: Fetches up-to-date documentation on how to use a Terraform module. You must call &#x27;search_modules&#x27; first to obtain the exact valid and compatible module_id required to use this tool.
3. **Check Use Cases**: Match tool use cases to your specific task requirements
4. **Be Selective**: Only include tools relevant to the specific task

### Configuration Format

**🚨 CRITICAL: client-config.json MUST use this EXACT format - nothing else!**

```json
{
  "remoteTools": [
    "actual_tool_name_from_catalog",
    "another_tool_name_from_catalog"
  ],
  "localServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "tools": ["read_file", "write_file", "list_directory", "create_directory", "edit_file"],
      "workingDirectory": "project_root"
    }
  }
}
```

**❌ NEVER generate task specification documents with fields like:**
- `task_id`, `task_name`, `task_type`, `priority`, `status`
- `required_tools`, `recommended_tools`, `optional_tools`
- `tool_configuration`, `technical_requirements`, `testing_requirements`
- Complex nested objects or metadata

**✅ ONLY generate simple MCP client configuration with:**
- `remoteTools` array (tool names from this catalog)
- `localServers` object (exact command/args shown above)

### Example Tool Selections

**For file operations**: Include tools from local filesystem server like `read_file`, `write_file`, `list_directory`

**For research/documentation**: Include remote tools like search and documentation tools

**For infrastructure tasks**: Include Kubernetes and system management tools

**For development tasks**: Include relevant language-specific documentation tools

### Validation Requirements

- ✅ Use specific tool names from this catalog (never wildcards)
- ✅ Include complete server configuration for local servers
- ✅ Only select tools relevant to the specific task
- ✅ Use `workingDirectory: "project_root"` for local filesystem servers
- ✅ Verify all tool names exist in this catalog


---

**Note**: All configuration format details, selection guidelines, examples, and validation requirements are provided in the tool catalog section above. Use that information to generate appropriate `client-config.json` files for each task.

### Step 3: Complete Documentation Generation
**⚠️ CRITICAL: You MUST generate ALL documentation files as specified above.**

**Git Workflow:** The orchestrator post-completion hook will automatically handle:
- Creating and checking out the feature branch: `docs-gen-main`
- Staging all documentation files
- Committing with proper message
- Pushing to origin
- Creating pull request to target branch: `main`

**Your Job:** Focus ONLY on generating the documentation files. Do NOT run any git commands.

✅ **Final Confirmation Required:** "✅ DOCUMENTATION FILES GENERATED - Hook will handle git workflow and PR creation"

## Error Handling
- If documentation generation fails, report the error and retry
- If file creation fails, check permissions and retry
- **DO NOT give up - complete the documentation generation**
- **Note:** Git workflow and PR creation are handled by the hook, not the agent

## Quality Standards
- Well-structured and comprehensive content
- Actionable implementation guidance
- Proper markdown formatting
- Code examples where relevant
- Clear cross-references between documents
- Maintain consistency across ALL documents

## Final Confirmation Required
**YOU MUST END WITH THIS EXACT MESSAGE:**
```
🎉 DOCUMENTATION GENERATION COMPLETE 🎉
✅ Generated documentation for ALL tasks
✅ Created all required documentation files (task.md, prompt.md, acceptance-criteria.md, client-config.json, toolman-guide.md)
📋 Total files created: [COUNT]
🔧 Generated task-specific Toolman configurations for code implementation
🔗 Git workflow and pull request will be handled automatically by orchestrator hook
```

**If you cannot provide this final confirmation, the task is NOT complete and you must continue working until it is done.**

