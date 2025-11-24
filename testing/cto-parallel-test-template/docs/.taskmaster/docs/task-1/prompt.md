# Autonomous Agent Prompt: Database Schema Setup

## Mission
You are a Rust backend developer tasked with creating the database schema and configuration for an e-commerce API using Diesel ORM and PostgreSQL. This is a foundational task with no dependencies.

## Goal
Set up a complete, production-ready database layer with:
- Diesel ORM configuration
- PostgreSQL schema for users, products, carts, and cart items
- Migration files for version control
- ORM model structs with proper traits
- Connection pooling

## Prerequisites
- PostgreSQL must be installed and running
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`
- Working directory: project root

## Step-by-Step Instructions

### 1. Add Dependencies to Cargo.toml
Add these dependencies to the `[dependencies]` section:
```toml
diesel = { version = "2.1.0", features = ["postgres", "r2d2", "chrono"] }
r2d2 = "0.8.10"
dotenv = "0.15.0"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Verify:** Run `cargo check` to ensure dependencies resolve correctly.

### 2. Create Environment Configuration
Create `.env` in project root:
```
DATABASE_URL=postgres://username:password@localhost/ecommerce_db
```

**Note:** Replace credentials with actual PostgreSQL credentials.

### 3. Initialize Diesel
Run these commands in sequence:
```bash
diesel setup
diesel migration generate create_users
diesel migration generate create_products
diesel migration generate create_carts
diesel migration generate create_cart_items
```

This creates the `migrations/` directory structure.

### 4. Write Migration Files
For each migration, create both `up.sql` (create) and `down.sql` (drop) files:

**migrations/*/create_users/up.sql:**
```sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL UNIQUE,
  email VARCHAR NOT NULL UNIQUE,
  password_hash VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**migrations/*/create_users/down.sql:**
```sql
DROP TABLE users;
```

**migrations/*/create_products/up.sql:**
```sql
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  description TEXT,
  price NUMERIC NOT NULL,
  inventory_count INTEGER NOT NULL
);
```

**migrations/*/create_products/down.sql:**
```sql
DROP TABLE products;
```

**migrations/*/create_carts/up.sql:**
```sql
CREATE TABLE carts (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**migrations/*/create_carts/down.sql:**
```sql
DROP TABLE carts;
```

**migrations/*/create_cart_items/up.sql:**
```sql
CREATE TABLE cart_items (
  id SERIAL PRIMARY KEY,
  cart_id INTEGER NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  quantity INTEGER NOT NULL
);
```

**migrations/*/create_cart_items/down.sql:**
```sql
DROP TABLE cart_items;
```

### 5. Run Migrations
Apply all migrations:
```bash
diesel migration run
```

This will auto-generate `src/schema.rs`. Verify it contains all table definitions.

### 6. Create Database Configuration Module
Create `src/config/db.rs`:
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

Create `src/config/mod.rs`:
```rust
pub mod db;
```

### 7. Create ORM Models
Create `src/models.rs` with all model structs:
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

### 8. Register Modules
Update `src/main.rs` or `src/lib.rs`:
```rust
pub mod config;
pub mod models;
pub mod schema;
```

### 9. Validate Implementation
Run these commands to verify correctness:
```bash
cargo check              # Verify compilation
diesel migration redo    # Test up/down migrations
cargo test               # Run any tests
```

## Success Criteria
You have succeeded when:
- [ ] `cargo check` passes without errors
- [ ] All 4 tables exist in PostgreSQL database
- [ ] `src/schema.rs` is auto-generated and contains all tables
- [ ] `src/models.rs` defines all model structs
- [ ] `src/config/db.rs` provides connection pooling
- [ ] `diesel migration redo` works without errors
- [ ] Foreign key relationships are correctly defined

## Error Handling
- **"DATABASE_URL not set"**: Create `.env` file with valid credentials
- **"connection refused"**: Ensure PostgreSQL is running
- **"relation already exists"**: Run `diesel migration revert` first
- **Compilation errors**: Check Diesel version compatibility (2.1.0)

## Key Constraints
- Use `NUMERIC` type for price fields (financial accuracy)
- Include `ON DELETE CASCADE` for foreign keys
- Add `UNIQUE` constraints on username and email
- Use `SERIAL` for auto-incrementing primary keys
- Include `created_at` timestamps where appropriate

## Resources
- Diesel Documentation: https://diesel.rs/guides/getting-started
- PostgreSQL Docs: https://www.postgresql.org/docs/
- See `.taskmaster/docs/architecture.md` for schema details

## Deliverables
Submit the following files:
1. `Cargo.toml` (with database dependencies)
2. `src/schema.rs` (auto-generated by Diesel)
3. `src/models.rs` (ORM models)
4. `src/config/db.rs` (connection pooling)
5. `migrations/*/up.sql` (4 migration files)
6. `migrations/*/down.sql` (4 rollback files)
7. `.env` (database connection string)

## Time Estimate
30 minutes for an experienced Rust developer with Diesel knowledge.
