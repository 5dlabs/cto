Implement subtask 7007: Implement skills: finance, social-media, rms-*, admin

## Objective
Implement the operational skill set for Morgan: finance reporting and invoicing (finance), social media management (social-media), rental management system operations (rms-*), and administrative functions (admin).

## Steps
1. **finance skill**: Handle invoice generation (sigma1_submit_invoice), payment status queries, and finance report requests (sigma1_finance_report). Format financial data clearly.
2. **social-media skill**: Allow authorized users to create social media posts (sigma1_social_post) and schedule content (sigma1_social_schedule). Confirm post content before publishing.
3. **rms-* skills**: Implement rental management operations: create rental jobs (sigma1_rms_create_job), update job status (sigma1_rms_update_status), query job details. Handle multi-step workflows (create → assign → dispatch → complete).
4. **admin skill**: User management operations (sigma1_admin_users). List users, update roles, manage permissions. Require elevated authorization for admin actions.
5. Implement permission checks: certain skills (admin, finance) should only be accessible to authorized users.
6. Define skill activation triggers and conversation context switching.

## Validation
Request an invoice; verify sigma1_submit_invoice is called with correct parameters. Request a finance report; verify data is returned and formatted. Create a social media post via conversation; verify sigma1_social_post is called. Create and update a rental job; verify correct RMS tools are called in sequence. Attempt admin action as unauthorized user; verify rejection.