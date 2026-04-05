Implement subtask 7005: Register all MCP tools with tool-server mapping to backend service endpoints

## Objective
Register every MCP tool (sigma1_catalog_search, sigma1_check_availability, sigma1_customer_vet, sigma1_create_quote, sigma1_submit_invoice, sigma1_social_post, etc.) with the tool-server, mapping each to the correct backend service API endpoint.

## Steps
1. Define tool schemas for each MCP tool: name, description, input parameters (JSON Schema), output schema.
2. Tools to register:
   - sigma1_catalog_search → Equipment Catalog search endpoint
   - sigma1_check_availability → Equipment Catalog availability endpoint
   - sigma1_customer_vet → Customer Vetting service endpoint
   - sigma1_create_quote → Quoting/Invoicing quote creation endpoint
   - sigma1_get_quote → Quoting/Invoicing quote retrieval endpoint
   - sigma1_submit_invoice → Quoting/Invoicing invoice submission endpoint
   - sigma1_finance_report → Finance service reporting endpoint
   - sigma1_social_post → Social Media Engine post endpoint
   - sigma1_social_schedule → Social Media Engine scheduling endpoint
   - sigma1_rms_create_job → RMS job creation endpoint
   - sigma1_rms_update_status → RMS status update endpoint
   - sigma1_admin_users → Admin user management endpoint
3. Register each tool via tool-server's registration API or configuration file.
4. Verify each tool is listed in the agent's available tools.
5. Test each tool invocation with sample parameters to confirm end-to-end connectivity.

## Validation
Query tool-server for registered tools; all expected tools are listed. Invoke each tool with sample parameters; verify correct backend endpoint is called and response is returned. No tools return connection errors.