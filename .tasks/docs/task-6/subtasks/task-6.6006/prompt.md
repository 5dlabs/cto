Implement subtask 6006: Implement platform-specific image cropping pipeline with sharp

## Objective
Build the image processing service that generates platform-specific crops from selected photos: Instagram square (1:1) and Story (9:16), LinkedIn landscape (1.91:1), Facebook (1.91:1), and TikTok (9:16).

## Steps
1. Create `src/services/ImageCropService.ts` as an Effect.Service:
   - Define crop profiles as constants:
     - INSTAGRAM_FEED: { width: 1080, height: 1080, ratio: '1:1' }
     - INSTAGRAM_STORY: { width: 1080, height: 1920, ratio: '9:16' }
     - LINKEDIN_POST: { width: 1200, height: 628, ratio: '1.91:1' }
     - FACEBOOK_POST: { width: 1200, height: 628, ratio: '1.91:1' }
     - TIKTOK_POST: { width: 1080, height: 1920, ratio: '9:16' }
   - Method `cropForPlatform(imageBuffer: Buffer, platform: Platform, originalWidth: number, originalHeight: number)`: Effect<Buffer, CropError>
     - Use `sharp` to: resize with cover strategy, apply crop to target dimensions.
     - Output as JPEG quality 90 for feed posts, WebP for story/TikTok.
   - Method `generateAllCrops(r2Key: string, platform: Platform)`: Effect<CroppedImage[], CropError>
     - Download original from R2 via R2Service.
     - Generate all crop variants for the given platform.
     - Upload each crop to R2 with key pattern: `social/crops/${platform}/${uuid}-${dimension}.jpg`.
     - Return array of { r2_key, width, height, format }.
   - Method `generateCropsForDrafts(selectedPhotos: Photo[], platforms: Platform[])`: Effect<Map<Platform, string[]>, CropError>
     - For each platform × each selected photo, generate crops.
     - Use Effect.forEach with concurrency: 4 for parallel processing.
     - Return map of platform → cropped image R2 keys.
2. Handle edge cases: images smaller than target crop (upscale with lanczos3), HEIC input (convert via sharp).
3. Create `src/services/ImageCropService.live.ts` — Effect Layer.

## Validation
Unit test: (1) Provide a 2000x3000 test image, crop for Instagram feed (1:1), verify output is 1080x1080 JPEG. (2) Crop for LinkedIn (1.91:1), verify output is 1200x628. (3) Test undersized image (500x500) for Instagram story — verify upscale to 1080x1920. (4) Test HEIC input conversion. (5) Verify R2 upload called with correct key patterns for each platform variant.