Implement subtask 7014: Implement social media approval workflow (social-media skill)

## Objective
Build the social-media skill workflow: when social media drafts are created, send previews to Mike via Signal, parse Mike's approve/reject responses, and call the corresponding Social Engine endpoints.

## Steps
1. Implement social-media skill handler:
   - Activated when Morgan creates a social media draft via sigma1_social_curate
   - Or when Morgan is asked to post content to social media
2. Draft creation flow:
   - User or automated trigger provides content (photo, caption, platform selection)
   - Call sigma1_social_curate to create draft
   - Receive draft_id and preview_url
3. Mike approval notification:
   - Send Signal message to Mike's number with draft details:
   - 'Social media draft ready for review:\n{preview_url}\nCaption: {caption}\nPlatforms: {platforms}\nReply APPROVE to publish or REJECT with reason'
   - Store pending_approval state in Valkey: key `social_approval:{draft_id}` → { draft_id, preview_url, status: 'pending', requested_at }
4. Mike response parsing:
   - When Signal message arrives from Mike's number, check if it matches a pending approval
   - Parse response:
     - 'APPROVE' or 'approve' or 'yes' or '👍' → call sigma1_social_publish with draft_id
     - 'REJECT' + optional reason → mark draft as rejected, store reason
     - 'SCHEDULE {datetime}' → call sigma1_social_publish with schedule_time
5. Confirmation:
   - After publish: send Mike confirmation with published URLs
   - After reject: acknowledge and store feedback
6. Timeout: if no response from Mike within 24 hours, send a reminder. After 48 hours, mark as expired.

## Validation
Create a test social media draft via sigma1_social_curate → verify Signal message is sent to Mike with preview URL. Simulate Mike replying 'APPROVE' → verify sigma1_social_publish is called and confirmation is sent. Simulate Mike replying 'REJECT too dark' → verify draft is marked rejected with reason. Test timeout: create draft, wait (or mock time), verify reminder is sent after 24h.