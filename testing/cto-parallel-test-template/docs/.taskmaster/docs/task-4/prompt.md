# Autonomous Agent Prompt: Product Catalog Module

## Mission
Implement a thread-safe product catalog system with inventory management and filtering capabilities for an e-commerce API. This is a foundational module with no dependencies.

## Goal
Create a complete product management system with:
- Thread-safe in-memory product storage
- CRUD operations for products
- Inventory tracking and updates
- Flexible product filtering (name, price, stock)
- Decimal precision for prices
- Auto-incrementing product IDs

## Prerequisites
- Rust toolchain installed
- Working directory: project root
- No external dependencies (Level 0 task)

## Step-by-Step Instructions

### 1. Add Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
rust_decimal = { version = "1.30", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. Create Module Structure
Create `src/catalog/mod.rs`:
```rust
pub mod models;
pub mod service;

pub use self::models::{Product, NewProduct, ProductFilter};
pub use self::service::ProductService;
```

### 3. Implement Product Models
Create `src/catalog/models.rs` with the Product, NewProduct, and ProductFilter structs using rust_decimal::Decimal for prices.

### 4. Implement ProductService
Create `src/catalog/service.rs` with thread-safe storage using Arc<Mutex<Vec<Product>>> and implement:
- `new()` - Initialize empty service
- `create(NewProduct)` - Add product with auto-incrementing ID
- `get_all()` - Return all products
- `get_by_id(i32)` - Find product by ID
- `update_inventory(i32, i32)` - Update stock count
- `filter(ProductFilter)` - Search products
- `delete(i32)` - Remove product

### 5. Register Module
Update `src/main.rs` or `src/lib.rs`:
```rust
pub mod catalog;
```

### 6. Write Tests
Test CRUD operations, filtering, concurrency, and decimal precision.

## Success Criteria
- [ ] All dependencies added
- [ ] Models implement Clone, Serialize, Deserialize
- [ ] ProductService is thread-safe
- [ ] Auto-incrementing IDs work
- [ ] Filtering works for all criteria
- [ ] Prices maintain decimal precision
- [ ] All tests pass

## Time Estimate
40 minutes for experienced Rust developer.
