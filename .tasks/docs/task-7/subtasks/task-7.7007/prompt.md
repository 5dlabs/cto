Implement subtask 7007: Implement social media and admin skills with approval workflows

## Objective
Implement Morgan's social media skills (social-media) and admin skill using sigma1_social_curate, sigma1_social_publish MCP tools, including content curation, human approval workflows, and administrative command handling.

## Steps
Step 1: Implement the social-media skill — Morgan curates social media content (event photos, testimonials, promotional posts) via sigma1_social_curate, which suggests content from recent events and portfolio. Step 2: Implement the approval workflow — curated content is staged for human review before publishing; Morgan notifies admins via Signal with content previews and approve/reject options. Step 3: Implement publishing via sigma1_social_publish — once approved, Morgan triggers publishing to configured social media platforms. Step 4: Implement the admin skill — Morgan handles administrative commands (system status, configuration updates, user management queries) accessible only to authenticated admin users. Step 5: Implement approval state management — track pending approvals, send reminders, handle approval expiry.

## Validation
Trigger content curation and verify sigma1_social_curate returns content suggestions; verify approval notification is sent to admin via Signal; simulate admin approval and confirm sigma1_social_publish is invoked; admin commands are rejected for non-admin users.