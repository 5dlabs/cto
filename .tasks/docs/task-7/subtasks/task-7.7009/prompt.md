<identity>
You are angie working on subtask 7009 of task 7.
</identity>

<context>
<scope>
Run one interactive CLI test request per MCP tool to verify every tool correctly reaches its backend service, receives a valid response, and is correctly parsed by Morgan.
</scope>
</context>

<implementation_plan>
1. For each of the 12 tools, run `kubectl exec -n openclaw deployment/morgan -- morgan tool-test <tool_id> [params]` and capture output.
2. sigma1_catalog_search: query=lights → expect array with name and day_rate fields.
3. sigma1_check_availability: product_id=<known_id> from={today} to={today+7} → expect {quantity_available: integer}.
4. sigma1_generate_quote: customer_id=test-cust-001, line_items=[{product_id: <id>, quantity: 1, days: 2}] → expect opportunity_id in response; verify via GET /api/v1/opportunities/:id returns status=pending.
5. sigma1_vet_customer: org_name='Test Co', contact_email='test@example.com', event_description='Corporate event' → expect {vetting_id, status}; poll until final_score appears within 60s.
6. sigma1_score_lead: id=<opportunity_id from step 4> → expect {score: number, tier: string}.
7. sigma1_create_invoice: use opportunity from step 4 → expect {invoice_id, total_cents > 0}; verify via GET /api/v1/invoices/:id.
8. sigma1_finance_report: period=2025-Q1 → expect {total_revenue_cents, invoice_count}.
9. sigma1_social_curate: upload test image file → expect {draft_id}.
10. sigma1_social_publish: draft_id=<from step 9> → expect {published_url}.
11. sigma1_equipment_lookup: no params → expect array of catalog items.
12. sigma1_gdpr_export: customer_id=test-cust-001 → expect aggregated JSON with keys from at least 3 services.
13. sigma1_gdpr_delete: customer_id=test-cust-gdpr-del-001 (dedicated test customer) → expect per-service status map; verify audit log entry via OpenClaw audit API.
14. Record pass/fail per tool in a test report file at agents/morgan/test-results/tool-connectivity.json.
</implementation_plan>

<validation>
All 12 tool tests return exit code 0 from morgan tool-test. tool-connectivity.json shows 12 entries all with status: pass. No tool times out (default 30s timeout). GDPR delete audit log entry is present and contains customer_id and per-service statuses.
</validation>