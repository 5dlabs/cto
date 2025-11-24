# Task 5: Shopping Cart API

## Overview
Implement shopping cart functionality with API endpoints, integrating user authentication and product catalog for a complete e-commerce cart experience.

## Context
**Level 1 task** depending on Task 3 (Authentication) and Task 4 (Product Catalog). Combines JWT-based user isolation with product validation for a secure shopping cart.

## Objectives
1. Create CartService with thread-safe in-memory storage
2. Implement cart CRUD operations (add, remove, clear, get)
3. Build API routes with JWT authentication
4. Integrate with Product Catalog for validation
5. Handle inventory checking before adding items

## Dependencies
- **Task 3:** User Authentication (JWT validation)
- **Task 4:** Product Catalog (product retrieval and validation)

## Implementation Plan

### Step 1: Create Cart Service Module
Create `src/cart/mod.rs`:
```rust
pub mod service;
pub use self::service::{CartService, Cart, CartItem};
```

### Step 2: Implement Cart Data Structures
Create `src/cart/service.rs` with models:
```rust
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: i32,
    pub quantity: i32,
    pub product_name: String,
    pub unit_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub id: i32,
    pub user_id: i32,
    pub items: Vec<CartItem>,
}
```

### Step 3: Implement CartService
Add to `src/cart/service.rs`:
```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::catalog::models::Product;

pub struct CartService {
    carts: Arc<Mutex<HashMap<i32, Cart>>>,
    next_id: Arc<Mutex<i32>>,
}

impl CartService {
    pub fn new() -> Self {
        CartService {
            carts: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn get_or_create_cart(&self, user_id: i32) -> Cart {
        // Implementation per task.txt
    }

    pub fn add_item(&self, user_id: i32, product: &Product, quantity: i32) -> Cart {
        // Adds item or increments quantity if already in cart
    }

    pub fn remove_item(&self, user_id: i32, product_id: i32) -> Option<Cart> {
        // Removes item from cart
    }

    pub fn get_cart(&self, user_id: i32) -> Option<Cart> {
        // Retrieves user's cart
    }

    pub fn clear_cart(&self, user_id: i32) -> Option<Cart> {
        // Empties cart
    }
}
```

### Step 4: Create Cart API Routes
Create `src/api/cart_routes.rs`:
```rust
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddItemRequest {
    pub product_id: i32,
    pub quantity: i32,
}

pub fn configure_cart_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_cart))
       .route("/add", web::post().to(add_item))
       .route("/remove/{product_id}", web::delete().to(remove_item))
       .route("/clear", web::post().to(clear_cart));
}

// Handler functions with JWT extraction and validation
```

### Step 5: Integrate with Main App
Update `src/main.rs`:
```rust
mod cart;
use cart::CartService;

// In HttpServer::new():
let cart_service = web::Data::new(CartService::new());

App::new()
    .app_data(cart_service.clone())
    .configure(api::configure_routes)
```

Update `src/api/mod.rs`:
```rust
pub mod cart_routes;
```

## Testing Strategy
- Unit tests for CartService operations
- Integration tests with JWT authentication
- Test inventory validation
- Test cart isolation per user

## Success Criteria
- [ ] Cart service thread-safe with Arc<Mutex>
- [ ] All cart operations work correctly
- [ ] JWT authentication required for all endpoints
- [ ] Inventory checked before adding items
- [ ] Cart properly isolated per user

## Files Created
- `src/cart/mod.rs`
- `src/cart/service.rs`
- `src/api/cart_routes.rs`
