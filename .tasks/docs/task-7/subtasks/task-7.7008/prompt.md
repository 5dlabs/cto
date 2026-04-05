Implement subtask 7008: Implement RMS and admin skills

## Objective
Develop skills for rental management system operations (equipment lookup, availability, status) and administrative functions.

## Steps
1. Implement RMS skills (rms-*): equipment lookup via sigma1_equipment_lookup, availability checks via sigma1_check_availability, and catalog search via sigma1_catalog_search for operational queries (e.g., 'What excavators are available next week?').
2. Implement the admin skill: handle administrative commands such as system status checks, user management queries, and configuration updates as supported by backend tools.
3. Define conversation patterns for RMS queries: natural language equipment searches, availability date range queries, equipment detail requests.
4. Ensure RMS skills are accessible from all channels (Signal, voice, web chat).
5. Add logging for all RMS and admin operations.

## Validation
Equipment lookup skill returns correct data via sigma1_equipment_lookup; availability skill returns accurate availability for date ranges; admin skill handles system status queries; all skills produce correct tool invocations and parse responses; logs capture operations across channels.