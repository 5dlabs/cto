# Task 1: Database Schema Setup

## Overview
Create the foundational database schema and configuration for the e-commerce Rust API project using Diesel ORM and PostgreSQL.

## Context
This is a **Level 0 task** (no dependencies) that establishes the data persistence layer for the entire application. The schema supports:
- User authentication and management
- Product catalog with inventory tracking
- Shopping cart functionality with multi-item support

## Objectives
1. Set up Diesel ORM with PostgreSQL support
2. Define database schema for all core tables
3. Create migration files for version-controlled schema changes
4. Configure database connection pooling
5. Implement ORM model structs with proper traits

## Dependencies
**None** - This is a foundational task that can run in parallel with Tasks 3, 4, and 6.

## Architecture Context
Refer to `.taskmaster/docs/architecture.md` sections:
- **Database Schema** (lines 108-148): Table structure and relationships
- **Entity Relationships** (lines 150-159): Foreign key constraints
- **Backend Architecture** (lines 73-105): Module organization

## Implementation Plan

### Step 1: Add Database Dependencies
Update `Cargo.toml` with required dependencies:
```toml
[dependencies]
diesel = { version = "2.1.0", features = ["postgres", "r2d2", "chrono"] }
r2d2 = "0.8.10"
dotenv = "0.15.0"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Validation:** Run `cargo check` to verify dependency resolution.

### Step 2: Create Database Schema
Create `src/schema.rs` with Diesel table definitions:
```rust
table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    products (id) {
        id -> Integer,
        name -> Varchar,
        description -> Text,
        price -> Numeric,
        inventory_count -> Integer,
    }
}

table! {
    carts (id) {
        id -> Integer,
        user_id -> Integer,
        created_at -> Timestamp,
    }
}

table! {
    cart_items (id) {
        id -> Integer,
        cart_id -> Integer,
        product_id -> Integer,
        quantity -> Integer,
    }
}

joinable!(cart_items -> carts (cart_id));
joinable!(cart_items -> products (product_id));
joinable!(carts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    users,
    products,
    carts,
    cart_items,
);
```

### Step 3: Configure Database Connection
Create `src/config/db.rs` for connection pooling:
```rust
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> Pool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}
```

Create `.env` in project root:
```
DATABASE_URL=postgres://username:password@localhost/ecommerce_db
```

### Step 4: Create Migration Files
Install Diesel CLI:
```bash
cargo install diesel_cli --no-default-features --features postgres
```

Initialize migrations:
```bash
diesel setup
diesel migration generate create_users
diesel migration generate create_products
diesel migration generate create_carts
diesel migration generate create_cart_items
```

**Migration: create_users/up.sql**
```sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL UNIQUE,
  email VARCHAR NOT NULL UNIQUE,
  password_hash VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Migration: create_users/down.sql**
```sql
DROP TABLE users;
```

**Migration: create_products/up.sql**
```sql
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  description TEXT,
  price NUMERIC NOT NULL,
  inventory_count INTEGER NOT NULL
);
```

**Migration: create_products/down.sql**
```sql
DROP TABLE products;
```

**Migration: create_carts/up.sql**
```sql
CREATE TABLE carts (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Migration: create_carts/down.sql**
```sql
DROP TABLE carts;
```

**Migration: create_cart_items/up.sql**
```sql
CREATE TABLE cart_items (
  id SERIAL PRIMARY KEY,
  cart_id INTEGER NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  quantity INTEGER NOT NULL
);
```

**Migration: create_cart_items/down.sql**
```sql
DROP TABLE cart_items;
```

### Step 5: Create ORM Models
Create `src/models.rs` with Diesel model structs:
```rust
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub inventory_count: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub inventory_count: i32,
}

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = carts)]
pub struct Cart {
    pub id: i32,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = carts)]
pub struct NewCart {
    pub user_id: i32,
}

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(Cart))]
#[diesel(belongs_to(Product))]
#[diesel(table_name = cart_items)]
pub struct CartItem {
    pub id: i32,
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Insertable)]
#[diesel(table_name = cart_items)]
pub struct NewCartItem {
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}
```

### Step 6: Register Modules
Update `src/lib.rs` or `src/main.rs`:
```rust
pub mod config;
pub mod models;
pub mod schema;
```

## Testing Strategy
1. **Dependency Check:** `cargo check` should compile without errors
2. **Migration Test:** `diesel migration run` should apply all migrations
3. **Rollback Test:** `diesel migration redo` should test up/down migrations
4. **Schema Validation:** Verify `src/schema.rs` matches database structure
5. **Connection Test:** Create a simple test that establishes a connection pool

## Risks and Considerations
- **PostgreSQL Requirement:** Ensure PostgreSQL is installed and running locally
- **Environment Variables:** The `.env` file must exist with valid `DATABASE_URL`
- **Migration Order:** Migrations must respect foreign key dependencies
- **Numeric Precision:** The `price` field uses `Numeric` for financial accuracy
- **Cascading Deletes:** Foreign keys use `ON DELETE CASCADE` for cleanup

## Success Criteria
- [ ] All database dependencies added to `Cargo.toml`
- [ ] `src/schema.rs` created with 4 table definitions
- [ ] `src/config/db.rs` created with connection pooling
- [ ] `src/models.rs` created with all model structs
- [ ] 4 migration pairs (up/down) created in `migrations/` directory
- [ ] `cargo check` passes without errors
- [ ] `diesel migration run` successfully creates all tables
- [ ] Database connection pool can be established

## Files Modified/Created
- `Cargo.toml` - Add database dependencies
- `src/schema.rs` - Diesel schema definitions
- `src/models.rs` - ORM model structs
- `src/config/db.rs` - Database configuration
- `migrations/*/up.sql` - Schema creation migrations
- `migrations/*/down.sql` - Schema rollback migrations
- `.env` - Database connection string

## Next Steps
After completion, this schema will be used by:
- **Task 2:** API Endpoints (depends on schema.rs)
- **Task 3:** User Authentication (uses users table)
- **Task 4:** Product Catalog (uses products table)
- **Task 5:** Shopping Cart (uses carts and cart_items tables)
