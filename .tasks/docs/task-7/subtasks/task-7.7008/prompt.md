Implement subtask 7008: Implement agent skills: upsell, finance, social-media, and admin

## Objective
Implement the upselling, finance management, social media, and administrative skill definitions within the Morgan agent.

## Steps
1. Implement upsell skill:
   - Analyze current rental/quote context for upsell opportunities.
   - Suggest complementary equipment, extended warranties, delivery services.
   - Use catalog tools to find related items.
   - Track upsell acceptance/rejection.
2. Implement finance skill:
   - Handle invoice inquiries (finance_get_invoice_status).
   - Process payment confirmations (finance_process_payment).
   - Generate and send invoices (finance_create_invoice).
   - Answer billing questions.
3. Implement social-media skill:
   - Publish project showcases via social_publish_content.
   - Retrieve portfolio items via social_get_portfolio.
   - Schedule content via social_schedule_post.
   - Handle content approval workflows.
4. Implement admin skill:
   - Provide system status summaries.
   - Handle internal team queries.
   - Manage agent configuration queries.
5. Wire all skills into the agent's routing logic alongside skills from 7007.

## Validation
Upsell skill correctly identifies and presents relevant upsell opportunities; finance skill handles invoice and payment queries accurately; social-media skill successfully orchestrates content publishing; admin skill returns system information; all skills integrate into the routing system without conflicts.