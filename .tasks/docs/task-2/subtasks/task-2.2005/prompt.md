Implement subtask 2005: Implement repository layer for catalog queries

## Objective
Create repository modules with database query functions for categories, products (list, detail, search), and availability lookups using sqlx.

## Steps
1. Create src/repositories/category_repo.rs with functions: list_categories(pool, active_only) -> Vec<Category>, get_category_by_id(pool, id) -> Option<Category>, get_category_by_slug(pool, slug) -> Option<Category>. 2. Create src/repositories/product_repo.rs with functions: list_products(pool, filters: ProductFilter) -> (Vec<ProductListItem>, i64) supporting pagination (limit/offset), category_id filter, search by name/sku, and is_active filter. get_product_by_id(pool, id) -> Option<ProductDetail>. get_products_by_category(pool, category_id, pagination) -> (Vec<ProductListItem>, i64). 3. Create src/repositories/availability_repo.rs with: get_availability(pool, product_id) -> Option<AvailabilityResponse>, get_bulk_availability(pool, product_ids) -> Vec<AvailabilityResponse>. 4. Define a ProductFilter struct with optional fields: category_id, search_query, min_daily_rate, max_daily_rate, is_active, page, per_page. 5. Use sqlx::query_as! or query_as with named fields. Ensure all queries use parameterized inputs. 6. Add unit tests using sqlx::test with a test database for each repository function.

## Validation
Repository functions return correct results against a seeded test database. list_products with pagination returns expected page sizes and total counts. Search by name/SKU returns matching products. Availability query returns correct quantities.