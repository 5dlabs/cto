Implement subtask 2004: Seed database with 533+ products and 24 categories

## Objective
Create a seed script or migration that populates the catalog database with all 24 equipment categories and 533+ products with realistic rental equipment data.

## Steps
1. Create a seed migration or a standalone Rust binary (cargo workspace member) that inserts data. 2. Define 24 categories representing equipment rental segments (e.g., Aerial Lifts, Air Compressors, Compaction, Concrete & Masonry, Earth Moving, Generators, Heaters, HVAC, Ladders & Scaffolding, Lighting, Material Handling, Painting, Plumbing, Power Tools, Pressure Washers, Pumps, Safety Equipment, Saws, Surveying, Trailers, Trenchers, Trucks & Vehicles, Welding, Miscellaneous). 3. For each category, generate products with realistic SKUs (e.g., AL-001 through AL-030 for Aerial Lifts), names, descriptions, daily/weekly/monthly rates based on typical rental pricing. 4. Include image_url fields pointing to a placeholder path on the configured object storage (e.g., /catalog/images/{sku}.jpg). 5. Populate availability records with reasonable quantities (total_quantity between 1-20 per product, available_quantity <= total_quantity). 6. Use a transaction to ensure atomicity of the seed operation. 7. Add an idempotency check (skip if categories already exist) to allow re-running safely.

## Validation
After seeding, SELECT COUNT(*) FROM rms.categories returns 24. SELECT COUNT(*) FROM rms.products returns >= 533. SELECT COUNT(*) FROM rms.product_availability returns >= 533. All foreign key relationships are valid.