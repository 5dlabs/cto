Implement subtask 6006: Implement CropService with sharp for platform-specific image crops

## Objective
Create the CropService as an Effect service that generates platform-specific image crops using the sharp library: Instagram square (1:1), Instagram Story (9:16), LinkedIn (1.91:1), and TikTok (9:16).

## Steps
1. Install `sharp`.
2. Create `src/services/CropService.ts` as an Effect.Service.
3. Define platform crop specifications:
   - Instagram square: 1080x1080 (1:1)
   - Instagram Story: 1080x1920 (9:16)
   - LinkedIn: 1200x628 (1.91:1)
   - TikTok: 1080x1920 (9:16)
4. Implement `generateCrops(imageKey: string, platforms: string[]): Effect.Effect<PlatformCrops, CropError>`:
   - Download the original image from R2 via R2StorageService.
   - For each requested platform, use sharp to resize/crop to the target dimensions using `cover` fit with center gravity.
   - Upload each cropped image to R2 under `social/crops/{platform}/{originalId}_{platform}.jpg`.
   - Return `PlatformCrops` object: `{ instagram?: { square: { url, width, height }, story: { url, width, height } }, linkedin?: { url, width, height }, tiktok?: { url, width, height } }`.
5. Define `CropError` as a tagged Effect error.
6. Create `CropServiceLive` layer depending on R2StorageService.
7. Handle edge cases: very small images (upscale with sharp), images with unusual aspect ratios.

## Validation
Given a 4000x3000 test image, verify CropService produces: Instagram square at 1080x1080, Instagram Story at 1080x1920, LinkedIn at 1200x628, TikTok at 1080x1920. Verify each crop is uploaded to R2 with correct key pattern. Test with a portrait-oriented image (3000x4000) to verify correct cropping behavior.