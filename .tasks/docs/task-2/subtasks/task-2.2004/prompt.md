Implement subtask 2004: Implement machine-readable equipment API endpoints with S3/R2 image URL integration

## Objective
Implement the /equipment-api/catalog and /equipment-api/checkout endpoints designed for machine consumption by the Morgan agent, with full S3/R2 CDN image URL construction.

## Steps
1. Create src/handlers/equipment_api.rs.
2. GET /api/v1/equipment-api/catalog → equipment_catalog: return a complete, denormalized JSON document of all active products grouped by category, with full image CDN URLs, specifications, and pricing. This is optimized for a single fetch by the Morgan agent. Structure: { categories: [{ id, name, products: [{ id, name, description, daily_rate_cents, weekly_rate_cents, specifications, images: [{ url, alt_text }], availability_summary: { next_7_days_available: bool } }] }] }.
3. Construct image URLs by concatenating AppState.config.s3_cdn_base_url + '/' + image_key from the product_images table.
4. For availability_summary, query the next 7 days of availability and return a boolean indicating if any quantity is available on all 7 days.
5. POST /api/v1/equipment-api/checkout → equipment_checkout: accept a JSON body { items: [{ product_id, start_date, end_date, quantity }], customer_info: { name, email, phone } }. Validate all products exist, check availability for each item across the date range, and if all available, return a quote response: { quote_id (UUID), items: [...with pricing], total_cents, valid_until (now + 24h) }. Do NOT actually reserve inventory in v1 — this is a quote only.
6. Add input validation using serde with custom deserializers for dates and UUIDs.
7. Register routes under /api/v1/equipment-api prefix.
8. Add appropriate Content-Type headers and CORS for machine clients.

## Validation
equipment_catalog endpoint returns all active products with correct CDN image URLs; equipment_checkout with valid items returns a quote with correct pricing math; checkout with unavailable items returns a clear error; image URLs are well-formed and include the CDN base URL; JSON schema matches the documented structure.