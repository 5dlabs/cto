Implement subtask 2006: Implement admin CRUD endpoints with RBAC authorization

## Objective
Build admin-only endpoints for creating and updating products and categories, protected by role-based access control middleware.

## Steps
1. Implement `POST /api/v1/catalog/products`:
   - Accept JSON body with all product fields (name, sku, description, category_id, rates, image_key, specs).
   - Validate required fields and uniqueness constraints (sku).
   - Insert into DB and return the created product with 201 status.
2. Implement `PATCH /api/v1/catalog/products/:id`:
   - Accept partial JSON body; only update provided fields.
   - Return the updated product.
3. Implement `POST /api/v1/catalog/categories` and `PATCH /api/v1/catalog/categories/:id` similarly.
4. Implement RBAC middleware:
   - Extract JWT or API key from Authorization header.
   - Verify the token and extract the user's role.
   - Admin endpoints require `role: admin`.
   - Return HTTP 401 for missing auth, 403 for insufficient role.
5. On successful mutation, invalidate relevant Redis cache keys.
6. Add request logging for all admin operations (who, what, when).

## Validation
Verify admin can create and update products and categories with valid auth. Verify non-admin users receive 403. Verify missing auth returns 401. Verify SKU uniqueness constraint returns 409. Verify Redis cache is invalidated after mutations. Verify audit log entries are created for admin operations.