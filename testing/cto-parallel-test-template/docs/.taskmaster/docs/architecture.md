# E-Commerce Application Architecture

## Overview

This document describes the architecture of a full-stack e-commerce application designed to test parallel task execution in a multi-agent development environment. The application consists of a Rust backend API and a React frontend, implementing core e-commerce functionality including user authentication, product catalog, and shopping cart management.

## Technology Stack

### Backend
- **Language**: Rust
- **Web Framework**: Actix-web 4.3.1
- **Database ORM**: Diesel 2.1.0 with PostgreSQL
- **Authentication**: JWT (jsonwebtoken 8.3.0)
- **Password Hashing**: Argon2 0.5.0
- **Serialization**: Serde 1.0 with JSON support
- **Decimal Handling**: rust_decimal 1.30

### Frontend
- **Framework**: React 18.2.0
- **Routing**: React Router DOM 6.14.2
- **UI Library**: shadcn/ui (built on Radix UI)
- **HTTP Client**: Axios 1.4.0
- **Styling**: Tailwind CSS

### Database
- **RDBMS**: PostgreSQL
- **Migration Tool**: Diesel CLI
- **Connection Pooling**: r2d2 0.8.10

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                             │
│                  (React + shadcn/ui)                         │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │  Header  │  Product │   Cart   │  Login   │ Register │  │
│  │          │   List   │          │          │          │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │ HTTP/REST API
                          │ (JSON over HTTPS)
┌─────────────────────────▼───────────────────────────────────┐
│                      Backend API                             │
│                    (Actix-web + Rust)                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              API Routes Layer                        │  │
│  │  /api/health  /api/products  /api/cart  /api/auth   │  │
│  └────────────────────┬─────────────────────────────────┘  │
│  ┌────────────────────▼─────────────────────────────────┐  │
│  │           Business Logic Layer                       │  │
│  │  ┌──────────┬──────────┬──────────┬──────────────┐  │  │
│  │  │   Auth   │ Catalog  │   Cart   │  Integration │  │  │
│  │  │  Module  │  Module  │  Module  │    Tests     │  │  │
│  │  └──────────┴──────────┴──────────┴──────────────┘  │  │
│  └────────────────────┬─────────────────────────────────┘  │
│  ┌────────────────────▼─────────────────────────────────┐  │
│  │              Data Access Layer                       │  │
│  │              (Diesel ORM)                            │  │
│  └────────────────────┬─────────────────────────────────┘  │
└─────────────────────────┼───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    PostgreSQL Database                       │
│  ┌──────────┬──────────┬──────────┬──────────────────┐     │
│  │  users   │ products │  carts   │   cart_items     │     │
│  └──────────┴──────────┴──────────┴──────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Backend Architecture

### Module Structure

```
src/
├── main.rs                 # Application entry point
├── api/
│   ├── mod.rs             # API module exports
│   ├── routes.rs          # Main route configuration
│   ├── cart_routes.rs     # Cart-specific routes
│   └── errors.rs          # Error handling
├── auth/
│   ├── mod.rs             # Auth module exports
│   ├── jwt.rs             # JWT token handling
│   └── models.rs          # User models & password hashing
├── catalog/
│   ├── mod.rs             # Catalog module exports
│   ├── models.rs          # Product models
│   └── service.rs         # Product business logic
├── cart/
│   ├── mod.rs             # Cart module exports
│   └── service.rs         # Cart business logic
├── config/
│   └── db.rs              # Database configuration
├── models.rs              # Database entity models
└── schema.rs              # Database schema definitions

tests/
├── integration_tests.rs   # End-to-end integration tests
├── api_tests.rs           # API endpoint tests
└── auth_tests.rs          # Authentication tests
```

### Database Schema

#### Users Table
```sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL UNIQUE,
  email VARCHAR NOT NULL UNIQUE,
  password_hash VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### Products Table
```sql
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  description TEXT,
  price NUMERIC NOT NULL,
  inventory_count INTEGER NOT NULL
);
```

#### Carts Table
```sql
CREATE TABLE carts (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### Cart Items Table
```sql
CREATE TABLE cart_items (
  id SERIAL PRIMARY KEY,
  cart_id INTEGER NOT NULL REFERENCES carts(id),
  product_id INTEGER NOT NULL REFERENCES products(id),
  quantity INTEGER NOT NULL
);
```

### Entity Relationships

```
users (1) ──────< (N) carts
                       │
                       │ (1)
                       │
                       ▼
                    (N) cart_items (N) ──────> (1) products
```

## Core Modules

### 1. Database Schema Module (Task 1)
**Location**: `src/schema.rs`, `src/models.rs`, `migrations/`

**Responsibilities**:
- Define database table schemas using Diesel macros
- Implement ORM model structs for database entities
- Manage database migrations
- Handle database connection pooling

**Key Components**:
- Schema definitions for users, products, carts, and cart_items
- Model structs with Queryable and Insertable traits
- Database connection pool configuration
- Migration files for schema versioning

**Dependencies**: None (foundational module)

---

### 2. API Endpoints Module (Task 2)
**Location**: `src/api/`

**Responsibilities**:
- Configure HTTP routes and route handlers
- Handle HTTP request/response lifecycle
- Implement health check endpoint
- Set up route scopes for different resources
- Standardize error responses

**Key Components**:
- `configure_routes()` - Main route configuration
- Health check endpoint at `/api/health`
- Route scopes for `/api/users`, `/api/products`, `/api/cart`
- Error handling middleware
- Request/response serialization

**Dependencies**: Database Schema (Task 1)

---

### 3. User Authentication Module (Task 3)
**Location**: `src/auth/`

**Responsibilities**:
- JWT token creation and validation
- Password hashing and verification using Argon2
- User registration and login
- Authentication middleware for protected routes

**Key Components**:
- **JWT Handler** (`jwt.rs`):
  - `create_token()` - Generate JWT with 24-hour expiration
  - `validate_token()` - Verify and decode JWT claims
  - Claims struct with sub (user ID), exp, and iat fields

- **User Models** (`models.rs`):
  - User struct with password verification
  - `hash_password()` - Secure password hashing
  - `verify_password()` - Password validation

**Security Features**:
- Argon2 password hashing with random salt
- JWT-based stateless authentication
- Token expiration handling
- Secure password storage (never serialized)

**Dependencies**: None (independent module)

---

### 4. Product Catalog Module (Task 4)
**Location**: `src/catalog/`

**Responsibilities**:
- Product CRUD operations
- Inventory management
- Product filtering and search
- Price handling with decimal precision

**Key Components**:
- **Models** (`models.rs`):
  - `Product` - Full product entity
  - `NewProduct` - Product creation DTO
  - `ProductFilter` - Search/filter criteria

- **Service** (`service.rs`):
  - `create()` - Add new products
  - `get_all()` - List all products
  - `get_by_id()` - Retrieve single product
  - `update_inventory()` - Manage stock levels
  - `filter()` - Search products by criteria

**Features**:
- Thread-safe in-memory storage (Arc<Mutex>)
- Decimal price handling for financial accuracy
- Flexible filtering (name, price range, stock status)
- Auto-incrementing product IDs

**Dependencies**: None (independent module)

---

### 5. Shopping Cart Module (Task 5)
**Location**: `src/cart/`

**Responsibilities**:
- Cart creation and management
- Add/remove items from cart
- Cart item quantity updates
- Cart clearing and checkout preparation
- Integration with product catalog and authentication

**Key Components**:
- **Service** (`service.rs`):
  - `get_or_create_cart()` - Lazy cart initialization
  - `add_item()` - Add products to cart
  - `remove_item()` - Remove products from cart
  - `get_cart()` - Retrieve user's cart
  - `clear_cart()` - Empty cart contents

- **API Routes** (`api/cart_routes.rs`):
  - `GET /api/cart` - Get current cart
  - `POST /api/cart/add` - Add item to cart
  - `DELETE /api/cart/remove/{id}` - Remove item
  - `POST /api/cart/clear` - Clear cart

**Features**:
- User-specific cart isolation
- Automatic cart creation on first use
- Inventory validation before adding items
- JWT authentication required for all operations
- Thread-safe cart storage

**Dependencies**: 
- User Authentication (Task 3) - for JWT validation
- Product Catalog (Task 4) - for product validation

---

### 6. Frontend Components Module (Task 6)
**Location**: `frontend/src/`

**Responsibilities**:
- User interface for all application features
- Client-side routing and navigation
- API integration via Axios
- Responsive design with shadcn/ui and Tailwind CSS

**Key Components**:

- **App.js**: Main application component with routing
- **Header.js**: Navigation bar with cart badge and auth links
- **Footer.js**: Application footer
- **HomePage.js**: Landing page with call-to-action
- **ProductList.js**: Product grid with filtering
- **ProductDetail.js**: Individual product view
- **Cart.js**: Shopping cart with quantity controls
- **Login.js**: User login form
- **Register.js**: User registration form

**Features**:
- shadcn/ui components (built on Radix UI primitives)
- Tailwind CSS for styling and responsive design
- React Router for SPA navigation
- Responsive grid layouts
- Form validation
- Loading states and error handling
- JWT token management in localStorage

**Dependencies**: None (independent frontend)

---

### 7. Integration Tests Module (Task 7)
**Location**: `tests/`

**Responsibilities**:
- End-to-end testing of complete user flows
- API endpoint testing
- Authentication testing
- Component integration verification

**Test Suites**:

- **integration_tests.rs**:
  - Health check endpoint test
  - Full user shopping flow (browse → add to cart → checkout)
  - Multi-component integration tests

- **api_tests.rs**:
  - Product CRUD operations
  - Product filtering and search
  - Error handling and edge cases

- **auth_tests.rs**:
  - JWT creation and validation
  - Password hashing and verification
  - Protected route access control
  - Invalid token handling

**Testing Strategy**:
- Actix-web test utilities for HTTP testing
- Mock services for isolated testing
- Test data factories for consistent test setup
- Independent test execution (no shared state)

**Dependencies**: 
- API Endpoints (Task 2)
- Shopping Cart (Task 5)
- Frontend Components (Task 6)

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login and receive JWT
- `POST /api/auth/logout` - Logout (client-side token removal)

### Products
- `GET /api/products` - List all products
- `GET /api/products/:id` - Get product details
- `POST /api/products` - Create product (admin)
- `PUT /api/products/:id` - Update product (admin)
- `DELETE /api/products/:id` - Delete product (admin)
- `GET /api/products/search` - Search/filter products

### Shopping Cart
- `GET /api/cart` - Get user's cart (requires auth)
- `POST /api/cart/add` - Add item to cart (requires auth)
- `DELETE /api/cart/remove/:product_id` - Remove item (requires auth)
- `POST /api/cart/clear` - Clear cart (requires auth)

### Health Check
- `GET /api/health` - API health status

## Authentication Flow

```
┌─────────┐                                    ┌─────────┐
│ Client  │                                    │  Server │
└────┬────┘                                    └────┬────┘
     │                                              │
     │  POST /api/auth/register                    │
     │  { username, email, password }              │
     ├────────────────────────────────────────────>│
     │                                              │
     │                                    Hash password (Argon2)
     │                                    Store user in DB
     │                                              │
     │  201 Created                                 │
     │  { user_id, username, email }               │
     │<────────────────────────────────────────────┤
     │                                              │
     │  POST /api/auth/login                       │
     │  { username, password }                     │
     ├────────────────────────────────────────────>│
     │                                              │
     │                                    Verify password
     │                                    Create JWT token
     │                                              │
     │  200 OK                                      │
     │  { token, user_id, username }               │
     │<────────────────────────────────────────────┤
     │                                              │
     │  Store token in localStorage                │
     │                                              │
     │  GET /api/cart                              │
     │  Authorization: Bearer <token>              │
     ├────────────────────────────────────────────>│
     │                                              │
     │                                    Validate JWT
     │                                    Extract user_id
     │                                    Fetch cart
     │                                              │
     │  200 OK                                      │
     │  { cart_data }                              │
     │<────────────────────────────────────────────┤
     │                                              │
```

## Shopping Cart Flow

```
┌─────────┐                                    ┌─────────┐
│ Client  │                                    │  Server │
└────┬────┘                                    └────┬────┘
     │                                              │
     │  Browse products                            │
     │  GET /api/products                          │
     ├────────────────────────────────────────────>│
     │                                              │
     │  200 OK                                      │
     │  [ { product_list } ]                       │
     │<────────────────────────────────────────────┤
     │                                              │
     │  Add to cart                                │
     │  POST /api/cart/add                         │
     │  { product_id: 1, quantity: 2 }             │
     ├────────────────────────────────────────────>│
     │                                              │
     │                                    Validate JWT
     │                                    Check inventory
     │                                    Add to cart
     │                                              │
     │  200 OK                                      │
     │  { cart }                                   │
     │<────────────────────────────────────────────┤
     │                                              │
     │  View cart                                  │
     │  GET /api/cart                              │
     ├────────────────────────────────────────────>│
     │                                              │
     │  200 OK                                      │
     │  { cart_items, total }                      │
     │<────────────────────────────────────────────┤
     │                                              │
```

## Task Dependencies and Parallel Execution

The project is designed to test parallel task execution with the following dependency structure:

### Level 0 (No Dependencies - Can Run in Parallel)
- **Task 1**: Database Schema Setup
- **Task 3**: User Authentication Module
- **Task 4**: Product Catalog Module
- **Task 6**: Frontend Components

### Level 1 (Depends on Level 0)
- **Task 2**: API Endpoints (depends on Task 1)
- **Task 5**: Shopping Cart API (depends on Tasks 3 & 4)

### Level 2 (Depends on Level 1)
- **Task 7**: Integration Tests (depends on Tasks 2, 5, & 6)

### Execution Strategy
```
Time →
─────────────────────────────────────────────────────────────
Level 0:  [Task 1] [Task 3] [Task 4] [Task 6]
          ↓        ↓        ↓
Level 1:  [Task 2] [Task 5]
          ↓        ↓
Level 2:  [Task 7]
─────────────────────────────────────────────────────────────
```

This structure allows for:
- **4 tasks** to run in parallel initially (Level 0)
- **2 tasks** to run in parallel after Level 0 completes (Level 1)
- **1 final task** after all others complete (Level 2)

## Security Considerations

### Authentication
- JWT tokens with 24-hour expiration
- Secure password hashing with Argon2
- Random salt generation for each password
- Tokens transmitted via Authorization header

### Data Protection
- Password hashes never serialized in responses
- SQL injection prevention via Diesel ORM
- Input validation on all endpoints
- CORS configuration for production

### Best Practices
- Environment variables for sensitive configuration
- Database connection pooling for performance
- Proper error handling without information leakage
- HTTPS required in production

## Development and Testing

### Running the Backend
```bash
# Set up database
diesel setup
diesel migration run

# Run development server
cargo run

# Run tests
cargo test

# Run with coverage
cargo tarpaulin
```

### Running the Frontend
```bash
cd frontend
npm install
npm start

# Run tests
npm test

# Build for production
npm run build
```

### Environment Variables
```bash
# Backend (.env)
DATABASE_URL=postgres://username:password@localhost/dbname
JWT_SECRET=your-secret-key-here
RUST_LOG=info

# Frontend (.env)
REACT_APP_API_URL=http://localhost:8080/api
```

## Deployment Architecture

### Production Setup
```
┌─────────────────────────────────────────────────────────────┐
│                      Load Balancer                           │
│                        (HTTPS)                               │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
┌───────▼────────┐       ┌────────▼───────┐
│  Frontend      │       │   Backend API  │
│  (Static CDN)  │       │   (Rust)       │
│  React Build   │       │   Multiple     │
│                │       │   Instances    │
└────────────────┘       └────────┬───────┘
                                  │
                         ┌────────▼───────┐
                         │   PostgreSQL   │
                         │   (Primary +   │
                         │   Replicas)    │
                         └────────────────┘
```

### Scaling Considerations
- Stateless API design allows horizontal scaling
- Database connection pooling for efficient resource use
- JWT tokens eliminate need for session storage
- Frontend served from CDN for global distribution
- Database read replicas for query scaling

## Future Enhancements

### Potential Features
- Order management and checkout flow
- Payment processing integration
- Product reviews and ratings
- User profile management
- Admin dashboard
- Email notifications
- Product image uploads
- Advanced search with Elasticsearch
- Real-time inventory updates via WebSocket
- Recommendation engine

### Technical Improvements
- GraphQL API alternative
- Redis caching layer
- Message queue for async operations
- Microservices architecture
- Kubernetes deployment
- Monitoring and observability (Prometheus, Grafana)
- API rate limiting
- Advanced security (2FA, OAuth)

## Conclusion

This architecture provides a solid foundation for a modern e-commerce application while serving as an effective test platform for parallel task execution in multi-agent development environments. The modular design, clear separation of concerns, and well-defined dependencies enable efficient parallel development and testing.

