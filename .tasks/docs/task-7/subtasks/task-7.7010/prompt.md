Implement subtask 7010: Document Morgan agent skills, MCP tool usage, and conversation flows

## Objective
Create comprehensive documentation covering Morgan's skills, MCP tool mappings, conversation flow diagrams, configuration reference, and operational runbook.

## Steps
Step 1: Document each skill (sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-*, admin) with: purpose, trigger phrases, MCP tools used, input/output data, and example conversation snippets. Step 2: Document each MCP tool with: endpoint, request/response schema, error codes, and retry behavior. Step 3: Create conversation flow diagrams showing the decision tree for common scenarios: new customer inquiry → qualification → quote → vetting → invoice. Step 4: Write a configuration reference documenting all environment variables, secrets, and tunable parameters. Step 5: Write an operational runbook covering: startup procedures, common failure modes, troubleshooting steps, and escalation paths.

## Validation
Documentation covers all 10 skills and 10 MCP tools; conversation flow diagrams are accurate against implementation; configuration reference lists all environment variables used in the codebase; runbook is reviewed by at least one other team member for clarity.