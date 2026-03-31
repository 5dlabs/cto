Implement subtask 7004: Implement MCP tool-server plugins for catalog and equipment tools

## Objective
Build MCP tool-server plugins for: catalog_search, check_availability, and equipment_lookup. Each plugin calls the corresponding Equipment Catalog backend service API and returns structured results to the agent.

## Steps
1. Implement the `catalog_search` MCP tool plugin:
   - Input schema: query string, optional category filter, optional price range.
   - Calls the Equipment Catalog service search endpoint (URL from ConfigMap).
   - Returns a list of matching equipment items with name, description, daily rate, and availability status.
2. Implement the `check_availability` MCP tool plugin:
   - Input schema: equipment_id, start_date, end_date.
   - Calls the Equipment Catalog service availability endpoint.
   - Returns availability boolean and any conflicting reservations.
3. Implement the `equipment_lookup` MCP tool plugin:
   - Input schema: equipment_id.
   - Calls the Equipment Catalog service detail endpoint.
   - Returns full equipment details (specs, images, pricing tiers, location).
4. Register all three tools with the OpenClaw MCP tool-server runtime.
5. Add input validation and meaningful error messages for each tool.
6. Test each tool independently by invoking it through the MCP tool interface.

## Validation
Invoke each tool via the MCP tool interface with valid inputs and verify correct structured responses from the Equipment Catalog service. Test with invalid inputs (missing fields, nonexistent equipment_id) and verify graceful error responses. Confirm all three tools appear in the agent's tool registry.