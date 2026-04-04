Implement subtask 6010: Implement GDPR deletion endpoint with R2 cleanup

## Objective
Build the DELETE /api/v1/gdpr/customer/:id endpoint that removes all photos, drafts, published posts, and R2 objects associated with a customer's events.

## Steps
1. Create `src/routes/gdpr.ts` with Elysia route.
2. `DELETE /api/v1/gdpr/customer/:id`:
   - Param: customer_id (string, the uploaded_by identifier).
   - Effect.gen pipeline:
     a. Find all uploads where uploaded_by = customer_id.
     b. For each upload: collect all photo r2_keys and all draft image_keys (cropped images).
     c. Collect all unique R2 keys (originals + crops).
     d. Call R2Service.deleteBatch(allKeys) to remove from R2.
     e. Delete published_posts (via draft_id cascade or explicit).
     f. Delete drafts (via upload_id cascade or explicit).
     g. Delete photos (via upload_id cascade or explicit).
     h. Delete uploads.
   - Use a database transaction (Effect.acquireRelease or sql transaction) to ensure atomicity of DB deletes.
   - Return { deleted: { uploads: number, photos: number, drafts: number, published_posts: number, r2_objects: number } }.
3. Handle case where customer has no data: return 200 with all counts = 0.
4. Log GDPR deletion request for audit trail (log customer_id, timestamp, counts).
5. Error handling: If R2 deletion partially fails, still proceed with DB deletion and log R2 failures for manual cleanup.

## Validation
Integration test: (1) Create upload with 5 photos, 4 drafts, 2 published_posts for customer 'test-customer'. Call DELETE /gdpr/customer/test-customer. Verify all DB records removed. Verify R2Service.deleteBatch called with all original + crop keys. (2) Test no-data case: DELETE for nonexistent customer returns 200 with zero counts. (3) Test R2 partial failure: mock one R2 delete failing, verify DB records still deleted and error logged.