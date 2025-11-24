# Autonomous Agent Prompt: Shopping Cart API

## Mission
Implement a complete shopping cart system with JWT-authenticated API endpoints, integrating with existing auth and catalog modules.

## Prerequisites
- Task 3 complete (JWT validation available)
- Task 4 complete (ProductService available)

## Implementation Steps

1. **Create cart module** with service layer
2. **Implement CartService** with thread-safe storage
3. **Create API routes** with JWT authentication
4. **Integrate services** in main.rs
5. **Test** all cart operations

## Key Implementation Details

### JWT Extraction Pattern
```rust
fn extract_user_id(req: &HttpRequest) -> Result<i32, ()> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if let Ok(claims) = validate_token(token) {
                    return Ok(claims.sub.parse::<i32>().unwrap_or(0));
                }
            }
        }
    }
    Err(())
}
```

### Inventory Validation
Before adding items:
```rust
if product.inventory_count >= quantity {
    // Add to cart
} else {
    // Return 400 Bad Request
}
```

## Success Criteria
- All endpoints return 401 without valid JWT
- Items added correctly with quantity tracking
- Inventory validated before adding
- Cart operations isolated per user
