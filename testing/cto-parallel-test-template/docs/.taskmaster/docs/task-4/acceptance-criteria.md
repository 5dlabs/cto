# Acceptance Criteria: Product Catalog Module

## Required Files Created

### 1. Dependencies
- [ ] `rust_decimal = { version = "1.30", features = ["serde"] }` in Cargo.toml
- [ ] serde and serde_json present

### 2. Module Structure
- [ ] `src/catalog/mod.rs` exists and exports models and service
- [ ] `src/catalog/models.rs` exists
- [ ] `src/catalog/service.rs` exists

### 3. Product Models
- [ ] `Product` struct with id, name, description, price (Decimal), inventory_count
- [ ] `NewProduct` struct for creation (no ID)
- [ ] `ProductFilter` struct with optional filters
- [ ] All structs derive Debug, Serialize, Deserialize
- [ ] Product derives Clone

### 4. ProductService Implementation
- [ ] Thread-safe storage with Arc<Mutex<Vec<Product>>>
- [ ] Auto-incrementing ID management
- [ ] `new()` method implemented
- [ ] `create()` method implemented
- [ ] `get_all()` method implemented
- [ ] `get_by_id()` method implemented
- [ ] `update_inventory()` method implemented
- [ ] `filter()` method implemented
- [ ] `delete()` method implemented (optional)

## Functional Requirements

### Product Creation
- [ ] Creates products with unique auto-incrementing IDs
- [ ] IDs start at 1 and increment sequentially
- [ ] Returns created product with assigned ID
- [ ] Handles concurrent creation correctly

### Product Retrieval
- [ ] `get_all()` returns all products
- [ ] `get_by_id()` returns Some(product) if found
- [ ] `get_by_id()` returns None if not found
- [ ] Clones products (doesn't expose internal state)

### Inventory Management
- [ ] Updates inventory count for existing products
- [ ] Returns updated product
- [ ] Returns None for non-existent products
- [ ] Handles negative inventory counts

### Product Filtering
- [ ] Name filter is case-insensitive
- [ ] Name filter uses substring matching
- [ ] Min price filter works correctly
- [ ] Max price filter works correctly
- [ ] In stock filter returns only products with inventory > 0
- [ ] Out of stock filter returns products with inventory = 0
- [ ] Multiple filters work together (AND logic)
- [ ] Empty filter returns all products

### Decimal Precision
- [ ] Prices use rust_decimal::Decimal type
- [ ] Decimal precision is maintained
- [ ] Price comparisons work correctly
- [ ] Serialization preserves decimal values

### Thread Safety
- [ ] Multiple threads can safely create products
- [ ] Multiple threads can safely read products
- [ ] No data races occur
- [ ] Lock contention is minimized

## Compilation and Tests

### Build Requirements
- [ ] `cargo check` passes
- [ ] `cargo build` succeeds
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt --check` passes

### Test Coverage
- [ ] Test product creation
- [ ] Test ID auto-increment
- [ ] Test get_all
- [ ] Test get_by_id (found and not found)
- [ ] Test update_inventory
- [ ] Test filtering by name
- [ ] Test filtering by price range
- [ ] Test filtering by stock status
- [ ] Test combined filters
- [ ] Test concurrent access
- [ ] Test decimal precision

## Definition of Done
1. All files created and compile
2. All functional requirements pass
3. All tests pass
4. Thread-safe concurrent access verified
5. Ready for integration with Task 5 (Shopping Cart)
