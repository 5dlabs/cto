Implement subtask 7004: Implement MCP tools for catalog search, availability check, and quote generation

## Objective
Build MCP tool-server tools that allow the Morgan agent to search the equipment catalog, check item availability, and generate/submit rental quotes by calling the appropriate backend APIs.

## Steps
1. In the tool-server, define the `catalog_search` MCP tool: accepts search query/filters, calls Equipment Catalog API search endpoint, returns formatted results.
2. Define the `availability_check` MCP tool: accepts equipment ID and date range, calls Equipment Catalog API availability endpoint, returns availability status.
3. Define the `quote_generate` MCP tool: accepts customer info, equipment list, dates, and delivery details; calls Quote Engine API to create a quote; returns quote summary with pricing.
4. Define the `quote_submit` MCP tool: accepts quote ID and customer confirmation; calls Quote Engine API to finalize the quote.
5. Use endpoint URLs from sigma1-infra-endpoints ConfigMap env vars.
6. Implement proper error handling and timeout for each tool (fail gracefully with user-friendly messages).
7. Register all tools with the OpenClaw agent's tool registry.

## Validation
Invoke each MCP tool individually with sample inputs; verify `catalog_search` returns equipment results from the backend; `availability_check` returns correct status; `quote_generate` returns a valid quote object; `quote_submit` finalizes the quote; error cases (invalid ID, service down) return graceful error messages.