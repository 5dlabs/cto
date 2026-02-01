#!/bin/bash

# Rex Backend API Test Script
# Tests all API endpoints

API_URL="http://localhost:3000"
API_BASE="${API_URL}/api/v1"

echo "======================================"
echo "Rex Backend API Test Suite"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to test endpoint
test_endpoint() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local expected_status=$5

    echo -e "${BLUE}Testing: ${name}${NC}"

    if [ -z "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X ${method} "${endpoint}")
    else
        response=$(curl -s -w "\n%{http_code}" -X ${method} "${endpoint}" \
            -H "Content-Type: application/json" \
            -d "${data}")
    fi

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" -eq "$expected_status" ]; then
        echo -e "${GREEN}✓ PASSED${NC} (Status: ${http_code})"
        echo "$body" | jq '.' 2>/dev/null || echo "$body"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAILED${NC} (Expected: ${expected_status}, Got: ${http_code})"
        echo "$body" | jq '.' 2>/dev/null || echo "$body"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
}

# Wait for server to be ready
echo "Checking if server is running..."
for i in {1..10}; do
    if curl -s "${API_URL}/health" > /dev/null 2>&1; then
        echo "Server is ready!"
        echo ""
        break
    fi
    if [ $i -eq 10 ]; then
        echo "Server is not running. Please start it with: npm start"
        exit 1
    fi
    sleep 1
done

# Health Check Tests
echo "======================================"
echo "1. Health Check Tests"
echo "======================================"
test_endpoint "Health Check" "GET" "${API_URL}/health" "" 200
test_endpoint "Readiness Check" "GET" "${API_URL}/readiness" "" 200

# Task Creation Tests
echo "======================================"
echo "2. Task Creation Tests"
echo "======================================"
test_endpoint "Create Task 1" "POST" "${API_BASE}/tasks" \
    '{"title":"Implement authentication","description":"Add JWT auth","priority":"high","status":"todo"}' 201

test_endpoint "Create Task 2" "POST" "${API_BASE}/tasks" \
    '{"title":"Fix bug in login","priority":"urgent","status":"in_progress"}' 201

test_endpoint "Create Task 3" "POST" "${API_BASE}/tasks" \
    '{"title":"Update documentation","description":"Update API docs","priority":"low"}' 201

test_endpoint "Create Task - Invalid (No Title)" "POST" "${API_BASE}/tasks" \
    '{"description":"Test task"}' 400

test_endpoint "Create Task - Invalid (Bad Priority)" "POST" "${API_BASE}/tasks" \
    '{"title":"Test","priority":"invalid"}' 400

# Task Retrieval Tests
echo "======================================"
echo "3. Task Retrieval Tests"
echo "======================================"
test_endpoint "Get All Tasks" "GET" "${API_BASE}/tasks" "" 200
test_endpoint "Get Task by ID" "GET" "${API_BASE}/tasks/task-1" "" 200
test_endpoint "Get Task - Not Found" "GET" "${API_BASE}/tasks/task-999" "" 404
test_endpoint "Get Task - Invalid ID" "GET" "${API_BASE}/tasks/invalid@id" "" 400

# Task Filtering Tests
echo "======================================"
echo "4. Task Filtering Tests"
echo "======================================"
test_endpoint "Filter by Status" "GET" "${API_BASE}/tasks?status=todo" "" 200
test_endpoint "Filter by Priority" "GET" "${API_BASE}/tasks?priority=high" "" 200
test_endpoint "Search Tasks" "GET" "${API_BASE}/tasks?search=auth" "" 200

# Task Update Tests
echo "======================================"
echo "5. Task Update Tests"
echo "======================================"
test_endpoint "Update Task Status" "PATCH" "${API_BASE}/tasks/task-1" \
    '{"status":"in_progress"}' 200

test_endpoint "Update Task Title" "PATCH" "${API_BASE}/tasks/task-1" \
    '{"title":"Updated: Implement authentication"}' 200

test_endpoint "Update Task - Not Found" "PATCH" "${API_BASE}/tasks/task-999" \
    '{"status":"completed"}' 404

test_endpoint "Update Task - No Fields" "PATCH" "${API_BASE}/tasks/task-1" \
    '{}' 400

# Statistics Tests
echo "======================================"
echo "6. Statistics Tests"
echo "======================================"
test_endpoint "Get Task Statistics" "GET" "${API_BASE}/tasks/stats" "" 200

# Task Deletion Tests
echo "======================================"
echo "7. Task Deletion Tests"
echo "======================================"
test_endpoint "Delete Task" "DELETE" "${API_BASE}/tasks/task-1" "" 204
test_endpoint "Delete Task - Not Found" "DELETE" "${API_BASE}/tasks/task-999" "" 404

# 404 Tests
echo "======================================"
echo "8. Error Handling Tests"
echo "======================================"
test_endpoint "Invalid Endpoint" "GET" "${API_BASE}/invalid" "" 404

# Summary
echo "======================================"
echo "Test Summary"
echo "======================================"
echo -e "Tests Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests Failed: ${RED}${TESTS_FAILED}${NC}"
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed! ✗${NC}"
    exit 1
fi
