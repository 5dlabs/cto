Implement subtask 2005: Implement S3/R2 image URL generation for product images

## Objective
Implement image URL construction and optional pre-signed URL generation for product images stored in S3/R2, integrated into product detail responses.

## Steps
1. Add the aws-sdk-s3 or rust-s3 crate to Cargo.toml. 2. Create src/services/s3.rs module. 3. Initialize an S3 client using S3_ENDPOINT, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY from environment/secrets. 4. Implement a function to construct public image URLs: '{S3_ENDPOINT}/{S3_PRODUCT_BUCKET}/{image_key}'. 5. Optionally implement pre-signed URL generation for private images with configurable expiry (e.g., 1 hour). 6. Integrate into the product detail endpoint (2003): when returning product data, resolve image_key to a full image URL. 7. Add error handling for missing images (return a default placeholder URL).

## Validation
Product detail responses include a valid, resolvable image_url field; the URL correctly points to the S3/R2 bucket; accessing the URL returns the image (or placeholder if image_key is null).