Implement subtask 2008: Integrate S3/R2 image URL generation in product responses

## Objective
Add a service layer that constructs fully qualified image URLs for product and category images using the configured object storage (S3 or R2) base URL.

## Steps
1. Add configuration for OBJECT_STORAGE_BASE_URL (e.g., https://assets.example.com or R2 public bucket URL) read from environment/ConfigMap. 2. Create src/services/image_service.rs with functions: resolve_image_url(relative_path: &str) -> String that prepends the base URL and optionally adds CDN transform parameters. resolve_thumbnail_url(relative_path: &str) -> String for thumbnail variants. 3. In the product service/handler layer, map stored relative image paths to full URLs before returning in API responses. 4. Handle the case where image_url is NULL in the database by returning a default placeholder URL. 5. If using R2, ensure the URL construction follows Cloudflare R2 public access patterns. If using S3, construct standard S3 URLs or presigned URLs depending on bucket policy.

## Validation
Product responses include fully qualified image URLs starting with the configured base URL. Products with no image return a placeholder URL. URLs are valid and correctly formed for the chosen object storage provider.