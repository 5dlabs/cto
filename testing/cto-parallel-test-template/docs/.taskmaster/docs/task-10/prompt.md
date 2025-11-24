# Autonomous Agent Prompt: Final Integration & Deployment Verification

## Mission
Perform comprehensive end-to-end validation of the complete system and verify deployment readiness.

## Validation Steps

### 1. Build Everything
```bash
cargo build --release
cd frontend && npm run build
```

### 2. Database Setup
```bash
diesel migration run
```

### 3. Start System
```bash
cargo run --release &
```

### 4. Run All Tests
```bash
cargo test --release
cd frontend && npm test
```

### 5. Manual E2E Testing
1. Open http://localhost:3000
2. Navigate through all pages
3. Test user registration/login
4. Add products to cart
5. Verify cart operations

### 6. Smoke Tests
```bash
curl http://localhost:8080/api/health
# Test other critical endpoints
```

### 7. Performance Check
```bash
hey -n 1000 http://localhost:8080/api/health
```

## Success Criteria
- All builds succeed
- All tests pass
- E2E flows work
- Performance acceptable
- No errors in logs
- System ready for deployment

Create a deployment readiness report with any issues found.
