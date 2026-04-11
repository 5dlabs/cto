<identity>
You are angie working on subtask 7002 of task 7.
</identity>

<context>
<scope>
Create individual YAML skill manifest files for each of the 9 Morgan skills: sales-qual, customer-vet, quote-gen, upsell, finance, social-media, rms-checkout, rms-checkin, and admin.
</scope>
</context>

<implementation_plan>
1. For each skill, create agents/morgan/skills/{skill-name}.yaml with fields: skill_id, display_name, description, tools (list of MCP tool IDs this skill uses), and skill_prompt (embedded decision logic as a string).
2. sales-qual.yaml: references sigma1_catalog_search, sigma1_score_lead; prompt includes lead scoring thresholds.
3. customer-vet.yaml: references sigma1_vet_customer; prompt includes go/no-go criteria based on final_score.
4. quote-gen.yaml: references sigma1_generate_quote, sigma1_check_availability; prompt includes line item assembly logic.
5. upsell.yaml: references sigma1_catalog_search, sigma1_equipment_lookup; prompt includes cross-sell logic.
6. finance.yaml: references sigma1_create_invoice, sigma1_finance_report; prompt includes invoice confirmation flow.
7. social-media.yaml: references sigma1_social_curate, sigma1_social_publish; prompt includes approval gate logic.
8. rms-checkout.yaml: references sigma1_generate_quote; prompt covers equipment dispatch confirmation.
9. rms-checkin.yaml: references sigma1_equipment_lookup; prompt covers return inspection.
10. admin.yaml: references sigma1_gdpr_export, sigma1_gdpr_delete; prompt includes GDPR request handling steps and confirmation requirements.
11. Register all skills in agent.yaml skills list.
</implementation_plan>

<validation>
`openclaw validate agents/morgan/skills/*.yaml` exits 0 for all 9 files. Each skill YAML has non-empty tools array and non-empty skill_prompt. `openclaw agent list-skills morgan` returns all 9 skill IDs.
</validation>