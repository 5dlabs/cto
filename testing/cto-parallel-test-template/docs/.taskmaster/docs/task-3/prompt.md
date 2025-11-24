# Autonomous Agent Prompt: User Authentication Module

## Mission
You are a security-focused Rust backend developer tasked with implementing a production-grade authentication system using JWT tokens and Argon2 password hashing. This is a foundational security module with no dependencies.

## Goal
Implement a complete, secure authentication system with:
- JWT token creation and validation (24-hour expiration)
- Argon2 password hashing with random salt
- User model with password verification
- Request/Response DTOs for auth endpoints
- Secure token management

## Prerequisites
- Rust toolchain installed
- Working directory: project root
- Basic understanding of JWT and password hashing
- No external dependencies (this is a Level 0 task)

## Step-by-Step Instructions

### 1. Add Authentication Dependencies
Add these to the `[dependencies]` section in `Cargo.toml`:
```toml
jsonwebtoken = "8.3.0"
argon2 = "0.5.0"
rand = "0.8.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Verify:** Run `cargo check` to ensure dependencies resolve correctly.

### 2. Create Module Structure
Create `src/auth/mod.rs`:
```rust
pub mod jwt;
pub mod models;

pub use self::jwt::{create_token, validate_token, Claims};
pub use self::models::User;
```

### 3. Implement JWT Token Handling
Create `src/auth/jwt.rs` with the following implementation:

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

**Key Points:**
- Tokens expire after 24 hours
- Secret key loaded from environment (with fallback for development)
- Standard JWT claims: sub (subject/user ID), exp (expiration), iat (issued at)

### 4. Implement User Model and Password Hashing
Create `src/auth/models.rs`:

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

**Important Security Notes:**
- `#[serde(skip_serializing)]` prevents password hash from appearing in JSON responses
- Each password gets a unique random salt (32 bytes)
- Argon2 is intentionally slow to resist brute force attacks
- `verify_password` returns `false` on any error (constant-time comparison)

### 5. Register the Module
Update `src/main.rs` or `src/lib.rs` to include:
```rust
pub mod auth;
```

If the file doesn't exist, create `src/lib.rs` with:
```rust
pub mod auth;
```

### 6. Optional: Add JWT Secret to Environment
Create or update `.env` file:
```
JWT_SECRET=your_secure_random_secret_key_minimum_32_characters_long
```

**Note:** The code has a fallback for development, but production MUST use environment variable.

### 7. Write Unit Tests
Create `src/auth/tests.rs` (or add to existing test file):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash1 = User::hash_password(password);
        let hash2 = User::hash_password(password);

        // Hashes should be different (due to random salt)
        assert_ne!(hash1, hash2);

        // Both should verify correctly
        let user1 = User {
            id: 1,
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password_hash: hash1,
        };

        assert!(user1.verify_password(password));
        assert!(!user1.verify_password("wrong_password"));
    }

    #[test]
    fn test_jwt_creation_and_validation() {
        use crate::auth::jwt::{create_token, validate_token};

        let user_id = "123";
        let token = create_token(user_id).unwrap();

        let claims = validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);

        // Check expiration is set
        assert!(claims.exp > 0);
        assert!(claims.iat > 0);
    }

    #[test]
    fn test_invalid_token() {
        use crate::auth::jwt::validate_token;

        let invalid_token = "invalid.token.here";
        assert!(validate_token(invalid_token).is_err());
    }
}
```

### 8. Validate Implementation
Run these commands to verify correctness:

```bash
# Compile and check for errors
cargo check

# Run tests
cargo test

# Check for warnings
cargo clippy

# Format code
cargo fmt
```

## Success Criteria
You have succeeded when:
- [ ] `cargo check` passes without errors
- [ ] `cargo test` runs and all tests pass
- [ ] `src/auth/jwt.rs` implements token creation and validation
- [ ] `src/auth/models.rs` implements User with password methods
- [ ] JWT tokens include sub, exp, and iat claims
- [ ] Tokens expire after 24 hours
- [ ] Password hashing uses Argon2 with random salt
- [ ] Password hash is excluded from serialization
- [ ] Password verification works correctly
- [ ] Invalid tokens are rejected
- [ ] Wrong passwords fail verification

## Common Pitfalls to Avoid
- **Don't hardcode secrets** - Always use environment variables in production
- **Don't store plaintext passwords** - Always hash before storage
- **Don't skip serde(skip_serializing)** - Password hashes must not be in JSON
- **Don't use weak hashing** - Argon2 is required for security
- **Don't panic on verification** - Use `unwrap_or(false)` pattern
- **Don't forget token expiration** - Always set exp claim
- **Don't reuse salts** - Generate new random salt for each password

## Error Handling Guide

### "JWT_SECRET not set"
This is expected in development. The code has a fallback value. For production:
```bash
export JWT_SECRET="your_secure_secret_key_here"
```

### "Failed to hash password"
Usually indicates memory issues. Argon2 is memory-intensive. Check system resources.

### "Invalid token"
Expected behavior for:
- Expired tokens
- Tampered tokens
- Tokens signed with different secret
- Malformed tokens

### Compilation Errors
- Ensure jsonwebtoken = "8.3.0" (not 9.x which has different API)
- Ensure argon2 = "0.5.0" (not 0.4.x which has different API)
- Check all imports are correct

## Security Checklist
- [ ] Password hash uses Argon2 (not MD5, SHA1, or bcrypt)
- [ ] Each password gets unique random salt
- [ ] JWT secret is loaded from environment
- [ ] JWT tokens have expiration (24 hours)
- [ ] Password hash never serialized to JSON
- [ ] Verification failures return false (not panic)
- [ ] Token validation checks expiration
- [ ] Constant-time comparison used (Argon2 handles this)

## Integration Notes
This module provides the foundation for:
- **Authentication routes** (Task 2 will add /login and /register endpoints)
- **Protected routes** (Task 5 Shopping Cart will require authentication)
- **User management** (Future tasks may add profile, password reset, etc.)

The module is intentionally stateless - no database queries in this file. Database integration happens in API routes.

## Performance Considerations
- **Argon2 is slow by design** - This is a security feature (~100ms per hash)
- **JWT validation is fast** - Milliseconds per validation
- **No database queries** - All operations are CPU-bound
- **Consider async** - Password hashing can block; consider tokio::task::spawn_blocking

## Testing Commands
```bash
# Unit tests only
cargo test --lib auth

# With output
cargo test -- --nocapture

# Specific test
cargo test test_password_hashing

# Check coverage (requires cargo-tarpaulin)
cargo tarpaulin --lib
```

## Resources
- JWT Specification: https://datatracker.ietf.org/doc/html/rfc7519
- Argon2 Algorithm: https://en.wikipedia.org/wiki/Argon2
- jsonwebtoken crate: https://docs.rs/jsonwebtoken/
- argon2 crate: https://docs.rs/argon2/
- OWASP Password Storage: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html

## Deliverables
Submit the following files:
1. `Cargo.toml` (with auth dependencies)
2. `src/auth/mod.rs` (module exports)
3. `src/auth/jwt.rs` (token handling)
4. `src/auth/models.rs` (user model and DTOs)
5. Unit tests demonstrating functionality

## Time Estimate
45 minutes for an experienced Rust developer with security knowledge.

## Final Validation
Before marking this task complete:
1. Run `cargo test` - all tests pass
2. Create a test token and validate it manually
3. Hash a password and verify it works
4. Check that password_hash doesn't appear in serialized User
5. Verify token expires after 24 hours (can mock system time in tests)
6. Confirm no compiler warnings with `cargo clippy`
