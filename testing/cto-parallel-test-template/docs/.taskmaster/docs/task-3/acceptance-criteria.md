# Acceptance Criteria: User Authentication Module

## Required Files Created

### 1. Dependencies in Cargo.toml
- [ ] `jsonwebtoken = "8.3.0"` added
- [ ] `argon2 = "0.5.0"` added
- [ ] `rand = "0.8.5"` added
- [ ] `serde` with derive feature present
- [ ] `serde_json` present

### 2. Module Structure
- [ ] `src/auth/mod.rs` exists
- [ ] Exports `jwt` module
- [ ] Exports `models` module
- [ ] Re-exports `create_token`, `validate_token`, `Claims`
- [ ] Re-exports `User` type

### 3. JWT Implementation (src/auth/jwt.rs)
- [ ] File exists and compiles
- [ ] `Claims` struct defined with `sub`, `exp`, `iat` fields
- [ ] `Claims` derives `Debug`, `Serialize`, `Deserialize`, `Clone`
- [ ] `create_token(user_id: &str)` function implemented
- [ ] `validate_token(token: &str)` function implemented
- [ ] Token expiration set to 24 hours
- [ ] JWT secret loaded from environment with fallback
- [ ] Uses `jsonwebtoken` crate correctly
- [ ] Returns proper Result types with error handling

### 4. User Model (src/auth/models.rs)
- [ ] File exists and compiles
- [ ] `User` struct defined with id, username, email, password_hash
- [ ] `User` derives `Debug`, `Clone`, `Serialize`, `Deserialize`
- [ ] `password_hash` field has `#[serde(skip_serializing)]` attribute
- [ ] `User::verify_password(&self, password: &str)` method implemented
- [ ] `User::hash_password(password: &str)` static method implemented
- [ ] `LoginRequest` struct defined
- [ ] `RegisterRequest` struct defined
- [ ] `AuthResponse` struct defined

### 5. Module Registration
- [ ] `src/main.rs` or `src/lib.rs` declares `pub mod auth;`
- [ ] Module is accessible from crate root

## Functional Requirements

### JWT Token Creation
- [ ] Tokens are valid JWT format
- [ ] Tokens contain `sub` claim with user ID
- [ ] Tokens contain `exp` claim set to now + 24 hours
- [ ] Tokens contain `iat` claim set to current timestamp
- [ ] Tokens can be decoded successfully
- [ ] Same user ID produces different tokens (due to timestamps)
- [ ] Token creation returns `Result<String, Error>`

### JWT Token Validation
- [ ] Valid tokens are accepted and claims extracted
- [ ] Invalid tokens are rejected with error
- [ ] Expired tokens are rejected
- [ ] Tampered tokens are rejected
- [ ] Tokens with wrong signature are rejected
- [ ] Returns `Result<Claims, Error>`
- [ ] Claims can be accessed after validation

### Password Hashing
- [ ] Uses Argon2 algorithm
- [ ] Generates random 32-byte salt for each password
- [ ] Same password produces different hashes (due to random salt)
- [ ] Hash is in Argon2 encoded format
- [ ] Hash can be verified with original password
- [ ] Function handles UTF-8 passwords correctly

### Password Verification
- [ ] Correct password returns `true`
- [ ] Incorrect password returns `false`
- [ ] Empty password is handled correctly
- [ ] Very long passwords are handled correctly
- [ ] Invalid hash format returns `false` (not panic)
- [ ] Special characters in passwords work correctly

### Serialization
- [ ] `User` can be serialized to JSON
- [ ] Serialized User does NOT include `password_hash` field
- [ ] `User` can be deserialized from JSON
- [ ] `LoginRequest` can be deserialized from JSON
- [ ] `RegisterRequest` can be deserialized from JSON
- [ ] `AuthResponse` can be serialized to JSON

## Compilation and Build

### Code Quality
- [ ] `cargo check` completes without errors
- [ ] `cargo build` completes successfully
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] No unused imports
- [ ] No dead code warnings

### Type Safety
- [ ] All public functions have explicit types
- [ ] Error types are properly propagated
- [ ] No `unwrap()` calls in production code paths
- [ ] Proper use of `Result` and `Option` types

## Testing Requirements

### Unit Tests - Password Hashing
- [ ] Test: Same password produces different hashes
```rust
let hash1 = User::hash_password("password");
let hash2 = User::hash_password("password");
assert_ne!(hash1, hash2);
```

- [ ] Test: Correct password verifies successfully
```rust
let user = User {
    password_hash: User::hash_password("password"),
    ..
};
assert!(user.verify_password("password"));
```

- [ ] Test: Incorrect password fails verification
```rust
let user = User {
    password_hash: User::hash_password("password"),
    ..
};
assert!(!user.verify_password("wrong"));
```

- [ ] Test: Empty password is handled
- [ ] Test: Special characters in password work

### Unit Tests - JWT Tokens
- [ ] Test: Token creation succeeds
```rust
let token = create_token("123").unwrap();
assert!(!token.is_empty());
```

- [ ] Test: Token validation succeeds with valid token
```rust
let token = create_token("123").unwrap();
let claims = validate_token(&token).unwrap();
assert_eq!(claims.sub, "123");
```

- [ ] Test: Invalid token is rejected
```rust
assert!(validate_token("invalid").is_err());
```

- [ ] Test: Token contains correct claims
```rust
let token = create_token("123").unwrap();
let claims = validate_token(&token).unwrap();
assert!(claims.exp > claims.iat);
assert_eq!(claims.sub, "123");
```

- [ ] Test: Expiration is ~24 hours in future
```rust
let token = create_token("123").unwrap();
let claims = validate_token(&token).unwrap();
let expected_exp = now + 86400; // 24 hours
assert!((claims.exp as i64 - expected_exp).abs() < 10);
```

### Unit Tests - Serialization
- [ ] Test: User serialization excludes password_hash
```rust
let user = User {
    password_hash: "secret".to_string(),
    ..
};
let json = serde_json::to_string(&user).unwrap();
assert!(!json.contains("password_hash"));
assert!(!json.contains("secret"));
```

- [ ] Test: LoginRequest deserialization works
- [ ] Test: RegisterRequest deserialization works
- [ ] Test: AuthResponse serialization works

### Integration Tests
- [ ] Test: Complete auth flow (hash → verify → create token → validate)
```rust
// Hash password
let hash = User::hash_password("mypassword");

// Create user
let user = User {
    id: 1,
    password_hash: hash,
    ..
};

// Verify password
assert!(user.verify_password("mypassword"));

// Create token
let token = create_token(&user.id.to_string()).unwrap();

// Validate token
let claims = validate_token(&token).unwrap();
assert_eq!(claims.sub, "1");
```

## Security Requirements

### Password Security
- [ ] Passwords are never stored in plaintext
- [ ] Argon2 algorithm is used (not MD5, SHA1, bcrypt)
- [ ] Random salt is generated for each password (32 bytes)
- [ ] Salt is different for each password hash
- [ ] Password hash is never logged or exposed
- [ ] Password hash is excluded from JSON serialization
- [ ] Timing attacks are mitigated (Argon2 provides this)

### JWT Security
- [ ] Tokens have expiration (24 hours maximum)
- [ ] Secret key is loaded from environment in production
- [ ] Secret key fallback is only for development
- [ ] Tokens are properly signed
- [ ] Token signature is validated on decode
- [ ] Expired tokens are rejected
- [ ] Invalid tokens don't cause panics

### Error Handling
- [ ] Verification errors return `false` (not panic)
- [ ] Invalid tokens return `Err` (not panic)
- [ ] Malformed data is handled gracefully
- [ ] No sensitive data in error messages
- [ ] Errors are logged appropriately

## Performance Requirements

### Password Hashing
- [ ] Hashing completes within reasonable time (<500ms)
- [ ] Verification completes within reasonable time (<500ms)
- [ ] Memory usage is acceptable (Argon2 uses ~64MB)

### JWT Operations
- [ ] Token creation is fast (<10ms)
- [ ] Token validation is fast (<10ms)
- [ ] No database queries needed for token operations

## Code Quality Standards

### Documentation
- [ ] Public functions have doc comments
- [ ] Complex logic is explained with comments
- [ ] Security considerations are documented
- [ ] Module-level documentation exists

### Error Messages
- [ ] Errors are descriptive
- [ ] No sensitive information in error messages
- [ ] Errors indicate what went wrong and how to fix

### Code Organization
- [ ] Clear separation: jwt.rs for tokens, models.rs for users
- [ ] No business logic in model structs (pure data + methods)
- [ ] Proper module visibility (pub where needed)
- [ ] Consistent naming conventions

## Edge Cases Handled

### Password Edge Cases
- [ ] Empty password: handled (should hash and verify)
- [ ] Very long password (>1000 chars): handled
- [ ] Unicode/emoji in password: handled correctly
- [ ] Whitespace in password: preserved
- [ ] Special characters: handled correctly

### Token Edge Cases
- [ ] Empty user ID: handled (creates valid token)
- [ ] Very long user ID: handled
- [ ] Special characters in user ID: handled
- [ ] Whitespace in user ID: preserved

### Error Conditions
- [ ] Missing JWT_SECRET env var: uses fallback
- [ ] Invalid hash format: returns false
- [ ] Malformed token: returns error
- [ ] Network issues: N/A (no network calls)
- [ ] Memory allocation failure: propagates error

## Environment Configuration

### Required Environment Variables
- [ ] `JWT_SECRET` is optional in development
- [ ] `JWT_SECRET` has reasonable fallback
- [ ] Documentation mentions production should set JWT_SECRET

### Optional Configuration
- [ ] Token expiration is configurable (optional enhancement)
- [ ] Argon2 config is customizable (optional enhancement)

## Validation Commands

Run these commands to validate acceptance criteria:

```bash
# 1. Check compilation
cargo check

# 2. Run all tests
cargo test

# 3. Run specific test module
cargo test auth::tests

# 4. Check for warnings
cargo clippy

# 5. Format check
cargo fmt --check

# 6. Test coverage (if available)
cargo tarpaulin --lib
```

## Manual Testing Scenarios

### Scenario 1: Create and Validate Token
```rust
use your_crate::auth::jwt::{create_token, validate_token};

let token = create_token("user_123").expect("Failed to create token");
println!("Token: {}", token);

let claims = validate_token(&token).expect("Failed to validate token");
println!("User ID: {}", claims.sub);
assert_eq!(claims.sub, "user_123");
```

### Scenario 2: Hash and Verify Password
```rust
use your_crate::auth::models::User;

let password = "SecureP@ssw0rd";
let hash = User::hash_password(password);

let user = User {
    id: 1,
    username: "testuser".to_string(),
    email: "test@example.com".to_string(),
    password_hash: hash,
};

assert!(user.verify_password(password));
assert!(!user.verify_password("WrongPassword"));
```

### Scenario 3: Serialization Safety
```rust
use your_crate::auth::models::User;

let user = User {
    id: 1,
    username: "testuser".to_string(),
    email: "test@example.com".to_string(),
    password_hash: "sensitive_hash_value".to_string(),
};

let json = serde_json::to_string(&user).unwrap();
println!("Serialized: {}", json);

// Verify password_hash is not in JSON
assert!(!json.contains("password_hash"));
assert!(!json.contains("sensitive_hash_value"));
```

## Definition of Done

The task is complete when:
1. All required files are created and compile
2. All functional requirements pass
3. All unit tests pass
4. All security requirements are met
5. `cargo check` succeeds with no warnings
6. `cargo test` passes all tests
7. Password hashes are secure and never exposed
8. JWT tokens work correctly with proper expiration
9. Code is documented and follows Rust best practices
10. Ready for integration with Task 5 (Shopping Cart) and Task 7 (Tests)

## Sign-Off Checklist

### Code Completeness
- [ ] All functions implemented
- [ ] All structs defined
- [ ] All traits derived
- [ ] Module exports correct

### Security Audit
- [ ] Password hashing reviewed and approved
- [ ] JWT implementation reviewed and approved
- [ ] No security vulnerabilities identified
- [ ] Error handling doesn't leak sensitive data

### Testing Coverage
- [ ] Unit tests written and passing
- [ ] Edge cases covered
- [ ] Integration tests work
- [ ] Manual testing scenarios verified

### Documentation
- [ ] Code is well-documented
- [ ] Security considerations noted
- [ ] Usage examples provided
- [ ] Integration points documented

### Quality Assurance
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Code is formatted
- [ ] Follows Rust conventions

### Readiness
- [ ] Task 5 can integrate this module
- [ ] Task 7 can test this module
- [ ] No blocking issues identified
- [ ] All acceptance criteria met
