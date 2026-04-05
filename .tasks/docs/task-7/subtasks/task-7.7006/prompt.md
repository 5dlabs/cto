Implement subtask 7006: Implement finance, social-media, rms, and admin skills

## Objective
Build the remaining skills: finance (invoice creation, reports), social-media (photo curation pipeline with Mike approval), rms (project status, checkout/checkin, crew scheduling), and admin (calendar, email, documents).

## Steps
1. `finance` skill:
   a. Handle invoice creation: when a project is confirmed, call `sigma1_create_invoice` with opportunity data.
   b. Handle invoice queries: 'What invoices are outstanding?' — call `sigma1_finance_report` with type=aging.
   c. Revenue reporting: 'How much revenue this month?' — call `sigma1_finance_report` with type=revenue, period=current_month.
   d. Format financial data as clean, readable summaries.
2. `social-media` skill:
   a. Receive event photos (from Signal or web chat uploads).
   b. Call `sigma1_social_curate` with the photo and event context to generate curated content.
   c. Send curated draft to Mike for approval via Signal message with preview.
   d. When Mike approves (responds with 'approve' or similar), call `sigma1_social_publish` with draft ID.
   e. Report back published URLs.
3. `rms-*` skills (project management):
   a. Project status queries: 'What's the status of the Johnson wedding?' — query RMS for opportunity by name/ID.
   b. Checkout/checkin coordination: 'Mark the LED panels as checked out for project X' — call appropriate RMS endpoint.
   c. Crew scheduling: 'Who's available Saturday?' — query RMS crew/calendar endpoints.
4. `admin` skill:
   a. Calendar queries: 'What's on the schedule next week?' — query via RMS Google Calendar integration.
   b. Email drafting: compose professional emails for customer follow-ups (output text, don't actually send).
   c. Document references: answer questions about company policies, equipment specs from stored documents in workspace.

## Validation
Finance: verify invoice creation returns valid invoice ID and finance report returns formatted revenue data. Social-media: verify photo upload triggers curation and approval flow sends message to Mike. RMS: verify project status query returns formatted project info. Admin: verify calendar query returns upcoming events.