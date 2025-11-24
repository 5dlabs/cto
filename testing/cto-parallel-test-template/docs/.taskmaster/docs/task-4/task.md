# Task 4: Product Catalog Module

## Overview
Implement a comprehensive product catalog system with inventory management, filtering capabilities, and thread-safe in-memory storage for the e-commerce API.

## Context
This is a **Level 0 task** (no dependencies) that provides:
- Product CRUD operations
- Inventory tracking and management
- Product filtering and search
- Thread-safe concurrent access
- Decimal precision for financial calculations

This module is independent and can be developed in parallel with Tasks 1, 3, and 6.

## Objectives
1. Implement thread-safe ProductService with in-memory storage
2. Create Product and NewProduct models
3. Implement product filtering by name, price, and stock status
4. Handle decimal prices with rust_decimal
5. Support concurrent access with Arc<Mutex>
6. Provide auto-incrementing product IDs

## Dependencies
**None** - This is a foundational task that can run in parallel with Tasks 1, 3, and 6.

## Architecture Context
Refer to `.taskmaster/docs/architecture.md` sections:
- **Product Catalog Module** (lines 234-263): Service implementation
- **Backend Architecture** (lines 73-105): Module structure
- **API Endpoints** (lines 374-396): Product routes

## Implementation Plan

### Step 1: Add Product Catalog Dependencies
Update `Cargo.toml`:
```toml
[dependencies]
rust_decimal = { version = "1.30", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Step 2: Create Catalog Module Structure
Create `src/catalog/mod.rs`:
```rust
pub mod models;
pub mod service;

pub use self::models::{Product, NewProduct, ProductFilter};
pub use self::service::ProductService;
```

### Step 3: Implement Product Models
Create `src/catalog/models.rs`:
```rust
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: Decimal,
    pub inventory_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub price: Decimal,
    pub inventory_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFilter {
    pub name_contains: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub in_stock: Option<bool>,
}

impl ProductFilter {
    pub fn new() -> Self {
        ProductFilter {
            name_contains: None,
            min_price: None,
            max_price: None,
            in_stock: None,
        }
    }
}
```

### Step 4: Implement ProductService
Create `src/catalog/service.rs`:
```rust
use crate::catalog::models::{Product, NewProduct, ProductFilter};
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};

pub struct ProductService {
    products: Arc<Mutex<Vec<Product>>>,
    next_id: Arc<Mutex<i32>>,
}

impl ProductService {
    pub fn new() -> Self {
        ProductService {
            products: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn create(&self, new_product: NewProduct) -> Product {
        let mut products = self.products.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        let product = Product {
            id: *next_id,
            name: new_product.name,
            description: new_product.description,
            price: new_product.price,
            inventory_count: new_product.inventory_count,
        };

        *next_id += 1;
        products.push(product.clone());
        product
    }

    pub fn get_all(&self) -> Vec<Product> {
        let products = self.products.lock().unwrap();
        products.clone()
    }

    pub fn get_by_id(&self, id: i32) -> Option<Product> {
        let products = self.products.lock().unwrap();
        products.iter().find(|p| p.id == id).cloned()
    }

    pub fn update_inventory(&self, id: i32, new_count: i32) -> Option<Product> {
        let mut products = self.products.lock().unwrap();
        if let Some(product) = products.iter_mut().find(|p| p.id == id) {
            product.inventory_count = new_count;
            Some(product.clone())
        } else {
            None
        }
    }

    pub fn filter(&self, filter: ProductFilter) -> Vec<Product> {
        let products = self.products.lock().unwrap();
        products
            .iter()
            .filter(|p| {
                let name_match = filter.name_contains
                    .as_ref()
                    .map_or(true, |name| {
                        p.name.to_lowercase().contains(&name.to_lowercase())
                    });

                let min_price_match = filter.min_price
                    .map_or(true, |min| p.price >= min);

                let max_price_match = filter.max_price
                    .map_or(true, |max| p.price <= max);

                let in_stock_match = filter.in_stock
                    .map_or(true, |in_stock| (p.inventory_count > 0) == in_stock);

                name_match && min_price_match && max_price_match && in_stock_match
            })
            .cloned()
            .collect()
    }

    pub fn delete(&self, id: i32) -> bool {
        let mut products = self.products.lock().unwrap();
        let initial_len = products.len();
        products.retain(|p| p.id != id);
        products.len() < initial_len
    }
}

impl Default for ProductService {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 5: Register Module
Update `src/main.rs` or `src/lib.rs`:
```rust
pub mod catalog;
```

## Testing Strategy
1. **Unit Tests for ProductService:**
   - Test product creation with auto-incrementing IDs
   - Test retrieving all products
   - Test getting product by ID
   - Test inventory updates
   - Test product filtering with various criteria
   - Test concurrent access (multiple threads)

2. **Unit Tests for Filtering:**
   - Test name filtering (case-insensitive)
   - Test price range filtering
   - Test stock status filtering
   - Test combined filters

3. **Decimal Price Tests:**
   - Test decimal precision is maintained
   - Test price comparisons work correctly

## Success Criteria
- [ ] All catalog dependencies added to `Cargo.toml`
- [ ] `src/catalog/mod.rs` created with module exports
- [ ] `src/catalog/models.rs` defines Product, NewProduct, ProductFilter
- [ ] `src/catalog/service.rs` implements ProductService
- [ ] Products use Decimal type for prices
- [ ] ProductService is thread-safe with Arc<Mutex>
- [ ] Auto-incrementing IDs work correctly
- [ ] CRUD operations implemented
- [ ] Filtering works for all criteria
- [ ] `cargo check` passes without errors
- [ ] Unit tests verify all operations

## Files Modified/Created
- `Cargo.toml` - Add rust_decimal dependency
- `src/catalog/mod.rs` - Module exports
- `src/catalog/models.rs` - Product data models
- `src/catalog/service.rs` - Business logic

## Next Steps
After completion, this module will be used by:
- **Task 5:** Shopping Cart API (validates products before adding to cart)
- **Task 2:** API Endpoints (will add product routes)
- **Task 7:** Integration Tests (tests product operations)
