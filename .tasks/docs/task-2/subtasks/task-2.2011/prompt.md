Implement subtask 2011: Create seed data script with 24 categories and sample products

## Objective
Write a SQL seed data file with 24 equipment rental categories and representative sample products with realistic specs, pricing, and availability data for development.

## Steps
1. Create `catalog/seed/seed_data.sql`.
2. Insert 24 categories representing equipment rental domains (e.g., Cameras, Lenses, Lighting, Audio, Grip, Drones, Monitors, Tripods, Power, Cable/Connectors, etc.) with appropriate parent-child hierarchy (e.g., 'Cameras' -> 'Cinema Cameras', 'DSLR/Mirrorless').
3. Insert at least 30 sample products across categories with:
   - Realistic names (e.g., 'ARRI ALEXA Mini LF', 'Sony FX6')
   - Populated JSONB specs (sensor size, resolution, weight, etc.)
   - Realistic day_rates
   - SKU and barcode values
   - Placeholder image_urls (e.g., `/images/products/{sku}.jpg`)
4. Insert availability data for the next 90 days for all products (e.g., quantity_total between 1-5).
5. Insert a few sample bookings to demonstrate availability reduction.
6. Add a `Makefile` target or script: `make seed` that runs `psql < seed_data.sql` or `sqlx` equivalent.

## Validation
Run the seed script against a fresh database (after migrations). Verify 24 categories exist with correct hierarchy. Verify 30+ products exist with non-null specs, day_rates, and SKUs. Verify availability rows cover the next 90 days. Query the availability endpoint for a seeded product and verify realistic data appears.