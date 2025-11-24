# Task 3: User Authentication Module

## Overview
Implement a secure user authentication system with JWT token handling and Argon2 password hashing for the e-commerce Rust API.

## Context
This is a **Level 0 task** (no dependencies) that provides:
- JWT-based stateless authentication
- Secure password hashing using Argon2
- Token creation and validation
- User model with password verification

This module is independent and can be developed in parallel with Tasks 1, 4, and 6.

## Objectives
1. Implement JWT token creation and validation
2. Set up Argon2 password hashing
3. Create user models with password verification
4. Establish authentication middleware foundation
5. Configure secure token management

## Dependencies
**None** - This is a foundational task that can run in parallel with Tasks 1, 4, and 6.

## Architecture Context
Refer to `.taskmaster/docs/architecture.md` sections:
- **User Authentication Module** (lines 203-230): JWT and password handling
- **Authentication Flow** (lines 398-440): Token-based auth sequence
- **Security Considerations** (lines 514-532): Best practices

## Implementation Plan

### Step 1: Add Authentication Dependencies
Update `Cargo.toml` with required authentication libraries:
```toml
[dependencies]
jsonwebtoken = "8.3.0"
argon2 = "0.5.0"
rand = "0.8.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Validation:** Run `cargo check` to verify dependency resolution.

### Step 2: Create Authentication Module Structure
Create `src/auth/mod.rs` to export module components:
```rust
pub mod jwt;
pub mod models;

pub use self::jwt::{create_token, validate_token, Claims};
pub use self::models::User;
```

### Step 3: Implement JWT Token Handling
Create `src/auth/jwt.rs` for JWT operations:
```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // Subject (user id)
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
}

pub fn create_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 24 * 3600; // 24 hours from now

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    };

    // In production, load from environment variable
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test_secret_key_change_in_production".to_string());

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test_secret_key_change_in_production".to_string());

    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    )?;

    Ok(token_data.claims)
}
```

### Step 4: Implement User Model with Password Hashing
Create `src/auth/models.rs` for user authentication logic:
```rust
use serde::{Serialize, Deserialize};
use argon2::{self, Config};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

impl User {
    /// Verify a password against the stored hash
    pub fn verify_password(&self, password: &str) -> bool {
        argon2::verify_encoded(&self.password_hash, password.as_bytes())
            .unwrap_or(false)
    }

    /// Hash a password using Argon2 with random salt
    pub fn hash_password(password: &str) -> String {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), &salt, &config)
            .expect("Failed to hash password")
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: i32,
    pub username: String,
}
```

### Step 5: Register Authentication Module
Update `src/main.rs` or `src/lib.rs`:
```rust
pub mod auth;
```

### Step 6: Add Environment Variable (Optional)
Update `.env` file with JWT secret:
```
JWT_SECRET=your_secure_random_secret_key_here
```

## Testing Strategy
1. **Unit Tests for JWT:**
   - Test token creation with valid user ID
   - Test token validation with valid token
   - Test token expiration handling
   - Test invalid token rejection

2. **Unit Tests for Password Hashing:**
   - Test password hashing generates unique hashes
   - Test password verification with correct password
   - Test password verification fails with wrong password
   - Test hash format is valid Argon2

3. **Integration Tests:**
   - Test complete auth flow (hash → verify → create token → validate token)
   - Test token expiration after 24 hours
   - Test multiple users don't interfere

## Security Considerations

### Password Security
- **Never store plaintext passwords** - Always use `hash_password()` before storage
- **Use random salt** - Argon2 generates unique salt for each password
- **Skip serialization** - `#[serde(skip_serializing)]` prevents password hash in JSON

### JWT Security
- **24-hour expiration** - Tokens expire after 1 day
- **Secure secret key** - Load from environment variable in production
- **Stateless tokens** - No server-side session storage required
- **Claims validation** - Always validate expiration and signature

### Best Practices
- Use HTTPS in production to protect tokens in transit
- Store tokens securely on client side (httpOnly cookies recommended)
- Implement token refresh mechanism for better UX
- Rate limit authentication endpoints to prevent brute force
- Log authentication attempts for security monitoring

## Risks and Considerations
- **Secret Key Management:** JWT_SECRET must be securely stored and rotated periodically
- **Token Expiration:** 24-hour expiration may need adjustment based on requirements
- **Password Complexity:** Consider enforcing password strength requirements
- **Timing Attacks:** Argon2 includes protection, but verify_password should use constant-time comparison
- **Concurrent Logins:** Stateless JWT allows multiple device logins

## Success Criteria
- [ ] All authentication dependencies added to `Cargo.toml`
- [ ] `src/auth/mod.rs` created with module exports
- [ ] `src/auth/jwt.rs` implements token creation and validation
- [ ] `src/auth/models.rs` implements User model with password methods
- [ ] JWT tokens expire after 24 hours
- [ ] Password hashing uses Argon2 with random salt
- [ ] Password hash never appears in serialized User
- [ ] `cargo check` passes without errors
- [ ] Unit tests verify JWT functionality
- [ ] Unit tests verify password hashing and verification
- [ ] Token validation rejects expired tokens
- [ ] Password verification fails for incorrect passwords

## Files Modified/Created
- `Cargo.toml` - Add authentication dependencies
- `src/auth/mod.rs` - Module exports
- `src/auth/jwt.rs` - JWT token handling
- `src/auth/models.rs` - User model and auth DTOs
- `.env` (optional) - JWT secret configuration

## Next Steps
After completion, this authentication module will be used by:
- **Task 5:** Shopping Cart API (requires JWT validation)
- **Task 7:** Integration Tests (tests auth flows)
- **Task 2:** API Endpoints (will add auth routes)

## Code Examples

### Creating a Token
```rust
use crate::auth::jwt::create_token;

let user_id = "123";
let token = create_token(user_id).expect("Failed to create token");
println!("JWT: {}", token);
```

### Validating a Token
```rust
use crate::auth::jwt::validate_token;

let token = "eyJ0eXAiOiJKV1QiLCJhbGc...";
match validate_token(token) {
    Ok(claims) => println!("Valid token for user: {}", claims.sub),
    Err(e) => println!("Invalid token: {}", e),
}
```

### Hashing and Verifying Password
```rust
use crate::auth::models::User;

// Hash a password
let password = "secure_password_123";
let hash = User::hash_password(password);

// Create user with hash
let user = User {
    id: 1,
    username: "john_doe".to_string(),
    email: "john@example.com".to_string(),
    password_hash: hash,
};

// Verify password
assert!(user.verify_password(password));
assert!(!user.verify_password("wrong_password"));
```

## Performance Notes
- **Argon2 is intentionally slow** - This is a security feature to resist brute force attacks
- **Token validation is fast** - JWT validation is cryptographically efficient
- **No database queries needed** - Stateless authentication reduces server load
- **Consider caching** - For high-traffic apps, cache validated tokens briefly

## Compliance and Standards
- **Password Hashing:** Follows OWASP recommendations using Argon2
- **JWT Standard:** Implements RFC 7519 (JSON Web Tokens)
- **Token Claims:** Uses standard claims (sub, exp, iat)
- **Cryptographic Standards:** Uses industry-standard algorithms
