Implement subtask 6006: Implement sharp image cropping for platform-specific formats and R2 upload

## Objective
Implement the image cropping step within the curation pipeline: use sharp to generate Instagram 1:1, Instagram Story 9:16, LinkedIn 1.91:1, and TikTok 9:16 crops for each selected photo, then upload cropped versions to R2.

## Steps
After selecting top photos in runCurationPipeline, for each selected photo URL: download the image buffer using fetch. Use sharp(buffer).resize(1080,1080,{fit:'cover'}).toBuffer() for Instagram crop. Use sharp(buffer).resize(1080,1920,{fit:'cover'}).toBuffer() for Instagram Story and TikTok crops (same dimensions, different keys). Use sharp(buffer).resize(1200,628,{fit:'cover'}).toBuffer() for LinkedIn crop. Upload each buffer to R2: key=`events/{event_id}/crops/{photo_uuid}/{platform}.jpg`. Collect URLs. Update social_drafts: instagram_crop_url, linkedin_crop_url, tiktok_crop_url with the first selected photo's crop URLs (representative crop). Store all selected crop URLs in selected_photo_urls array. Handle sharp errors per-photo non-fatally: log warning and skip that photo's crop.

## Validation
Unit test: provide a 2000x1500 JPEG buffer, run the cropping logic, verify output buffers have correct dimensions (1080x1080, 1080x1920, 1200x628) using sharp(output).metadata(). Integration test: after curation pipeline completes for a test draft, verify instagram_crop_url, linkedin_crop_url, tiktok_crop_url are non-null and the R2 objects are accessible.