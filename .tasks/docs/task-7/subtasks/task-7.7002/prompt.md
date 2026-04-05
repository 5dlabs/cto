Implement subtask 7002: Implement MCP Tool Server — Equipment Catalog tools (catalog_search, check_availability, equipment_lookup)

## Objective
Define and implement MCP tool definitions for the three Equipment Catalog service tools: sigma1_catalog_search, sigma1_check_availability, and sigma1_equipment_lookup, including JSON input/output schemas, HTTP method/URL templates, and authorization header injection.

## Steps
1. Create MCP tool definition for `sigma1_catalog_search`:
   - HTTP: GET /api/v1/catalog/products
   - Input schema: { query: string, category?: string, limit?: number, offset?: number }
   - Output schema: { products: [{ id, name, description, daily_rate, category, image_url }], total: number }
   - Description for LLM: 'Search the equipment catalog by keyword, category, or browse all available rental equipment'
2. Create MCP tool definition for `sigma1_check_availability`:
   - HTTP: GET /api/v1/catalog/products/:id/availability?from={ISO8601}&to={ISO8601}
   - Input schema: { product_id: string, from_date: string (ISO8601), to_date: string (ISO8601) }
   - Output schema: { available: boolean, quantity_available: number, conflicts: [{ date, reason }] }
   - Description: 'Check if a specific piece of equipment is available for a given date range'
3. Create MCP tool definition for `sigma1_equipment_lookup`:
   - HTTP: GET /api/v1/equipment-api/catalog
   - Input schema: { filter?: string, format?: 'detailed'|'summary' }
   - Output schema: { equipment: [{ id, name, specs, category, condition, serial_number }] }
   - Description: 'Machine-readable equipment catalog lookup for detailed specs and inventory management'
4. Each tool definition must include Authorization header template: `Bearer ${MORGAN_SERVICE_JWT}`
5. Implement HTTP client wrapper that maps MCP tool calls to actual HTTP requests against Equipment Catalog service URL from sigma1-infra-endpoints.
6. Handle error responses: 404 → 'Equipment not found', 503 → 'Catalog service temporarily unavailable', map to user-friendly messages.

## Validation
For each tool: invoke with valid test parameters and verify correct HTTP request is formed (method, URL, headers). Mock the Equipment Catalog service and verify response parsing matches output schema. Test error handling for 404, 500, and timeout scenarios.