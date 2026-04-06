Implement subtask 7006: Implement MCP tools for social media curation/publishing and RMS equipment lookup

## Objective
Build MCP tool-server tools for social media content curation and publishing via the Social Media Engine, and equipment data lookup via the RMS integration layer.

## Steps
1. Define `social_curate` MCP tool: accepts content theme or prompt, calls Social Media Engine curation endpoint, returns curated content suggestions.
2. Define `social_publish` MCP tool: accepts content (text, image refs, platform targets), calls Social Media Engine publish endpoint, returns publish confirmation and post URLs.
3. Define `equipment_rms_lookup` MCP tool: accepts equipment ID or search criteria, calls the RMS integration endpoint to retrieve detailed equipment data (specs, maintenance history, location).
4. Define `equipment_rms_status` MCP tool: accepts equipment ID, returns current RMS status (rented, available, maintenance).
5. Use endpoint URLs from sigma1-infra-endpoints ConfigMap env vars.
6. Handle authentication to each backend service as required.
7. Register all tools with the OpenClaw agent's tool registry.

## Validation
Invoke `social_curate` and verify content suggestions are returned; `social_publish` with test content confirms publishing; `equipment_rms_lookup` returns detailed equipment data; `equipment_rms_status` returns correct status; verify tools handle backend unavailability gracefully.