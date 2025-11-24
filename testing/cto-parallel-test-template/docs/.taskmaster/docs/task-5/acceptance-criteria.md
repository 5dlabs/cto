# Acceptance Criteria: Shopping Cart API

## Core Requirements
- [ ] `src/cart/mod.rs` exports CartService
- [ ] `src/cart/service.rs` implements all cart operations
- [ ] `src/api/cart_routes.rs` implements all API endpoints
- [ ] All endpoints require JWT authentication
- [ ] Returns 401 for missing/invalid tokens
- [ ] Inventory validated before adding items
- [ ] Cart isolated per user (user A can't access user B's cart)

## API Endpoints
- [ ] `GET /api/cart` - Get user's cart
- [ ] `POST /api/cart/add` - Add item (with inventory check)
- [ ] `DELETE /api/cart/remove/{id}` - Remove item
- [ ] `POST /api/cart/clear` - Clear cart

## Testing
```bash
# With valid JWT
TOKEN="valid_jwt_here"

# Get cart
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/cart

# Add item
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"product_id":1,"quantity":2}' \
  http://localhost:8080/api/cart/add

# Remove item
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/cart/remove/1

# Clear cart
curl -X POST -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/cart/clear
```

## Validation
- [ ] `cargo test` passes
- [ ] Cart operations work with valid JWT
- [ ] Returns 401 without JWT
- [ ] Returns 400 for insufficient inventory
- [ ] Returns 404 for invalid product_id
