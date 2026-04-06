Implement subtask 7007: Implement finance, social-media, RMS, and admin skills

## Objective
Implement the remaining agent skills for finance operations, social media management, rental management, and administrative tasks.

## Steps
1. Implement 'finance' skill: handle invoice generation from accepted quotes (quote → invoice via finance tools), payment status inquiries, and payment reminders.
2. Implement 'social-media' skill: allow authorized users to draft social media posts via conversation, submit them for approval, check approval status, and trigger publishing.
3. Implement 'rms-*' skills: rental lifecycle management — create rentals, check rental status, schedule deliveries/pickups, handle rental modifications and cancellations.
4. Implement 'admin' skill: provide admin users with system status, reporting summaries, and ability to manage customer records or override vetting decisions.
5. Add role-based access control to skills: admin skills should only be available to authenticated admin users (identify via Signal number or web chat auth).
6. Ensure each skill has clear entry/exit conditions in the agent's routing logic.

## Validation
Test each skill independently: finance skill generates an invoice from an existing quote; social-media skill creates and submits a draft; RMS skill creates a rental and schedules delivery; admin skill returns system status. Verify role gating prevents non-admin access to admin skills.