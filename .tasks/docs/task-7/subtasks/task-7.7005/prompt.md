Implement subtask 7005: Configure MCP tool-server with Equipment Catalog and RMS backend tools

## Objective
Set up the MCP tool-server and register tools for the Equipment Catalog service (search, lookup, availability) and Rental Management System (create rental, check status, manage returns).

## Steps
1. Initialize the MCP tool-server configuration within the OpenClaw agent.
2. Define and register Equipment Catalog tools:
   - catalog_search: Search equipment by category, specs, availability.
   - catalog_get_item: Get detailed info for a specific equipment item.
   - catalog_check_availability: Check rental availability for date range.
3. Define and register RMS tools:
   - rms_create_rental: Create a new rental agreement.
   - rms_get_rental_status: Check status of existing rental.
   - rms_process_return: Process equipment return.
   - rms_list_rentals: List rentals for a customer.
4. Configure each tool's endpoint URL from sigma1-infra-endpoints ConfigMap.
5. Define input/output schemas for each tool so the LLM can invoke them correctly.
6. Implement error handling for tool invocation failures (service unavailable, bad input).

## Validation
Agent can invoke each catalog tool and receive valid structured responses; agent can invoke each RMS tool and receive valid responses; tool schemas are correctly defined and the LLM selects appropriate tools for relevant queries; error cases return graceful error messages.