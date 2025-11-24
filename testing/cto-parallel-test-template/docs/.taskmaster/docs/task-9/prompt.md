# Autonomous Agent Prompt: Integration - Level 1

## Mission
Validate that Level 1 tasks (2, 5) integrate correctly with Level 0 components and each other.

## Validation Steps

1. **Start server**
```bash
cargo run &
sleep 5  # Wait for startup
```

2. **Test API endpoints**
```bash
curl http://localhost:8080/api/health
# Should return {"status":"ok"}
```

3. **Test cart with authentication**
```bash
# Create JWT token (use test utilities)
# Test cart endpoints with valid/invalid tokens
```

4. **Run integration tests**
```bash
cargo test --test integration_tests
cargo test --test api_tests
```

5. **Verify service integration**
Check that CartService correctly uses ProductService for validation.

## Success Criteria
- Server starts without errors
- All endpoints respond correctly
- Cart requires authentication
- Integration tests pass
- No route conflicts

Report any integration issues.
