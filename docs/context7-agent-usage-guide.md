# Context7 Usage Guide for Agents

## Overview

Context7 provides **up-to-date, version-specific documentation** for libraries and frameworks. Unlike static documentation or training data, Context7 queries live documentation sources to ensure you always have the latest information.

**When to Use Context7:**
- ✅ When you need current library/framework documentation
- ✅ When checking for breaking changes or new features
- ✅ When looking for code examples and best practices
- ✅ When verifying API signatures and usage patterns
- ✅ When exploring unfamiliar libraries

**When NOT to Use Context7:**
- ❌ For general programming concepts (use your knowledge)
- ❌ For project-specific code (use codebase search)
- ❌ For debugging existing code (analyze the code directly)

## Tool Name

```
context7_get_library_docs
```

## How to Use

### Basic Query Structure

Context7 works best with **specific, language-aware queries**. Always include:
1. The library/framework name
2. The programming language
3. What you want to know

**Good Query Pattern:**
```
"[library name] [specific feature] in [language]"
```

### Examples by Language

#### Rust Queries

```javascript
// Async runtime
context7_get_library_docs({
  query: "tokio async runtime setup and basic usage in Rust"
})

// Serialization
context7_get_library_docs({
  query: "serde derive macros for JSON serialization in Rust"
})

// Web framework
context7_get_library_docs({
  query: "axum web framework routing and middleware in Rust"
})

// Error handling
context7_get_library_docs({
  query: "anyhow error handling patterns in Rust"
})

// Database
context7_get_library_docs({
  query: "sqlx async PostgreSQL queries in Rust"
})
```

#### TypeScript/JavaScript Queries

```javascript
// React
context7_get_library_docs({
  query: "React hooks useState and useEffect with TypeScript"
})

// Next.js
context7_get_library_docs({
  query: "Next.js 14 app router server components with TypeScript"
})

// UI Components
context7_get_library_docs({
  query: "shadcn/ui form components with react-hook-form TypeScript"
})

// State Management
context7_get_library_docs({
  query: "Zustand state management with TypeScript patterns"
})

// API Client
context7_get_library_docs({
  query: "React Query data fetching and caching with TypeScript"
})
```

#### Python Queries

```javascript
// Web Framework
context7_get_library_docs({
  query: "FastAPI async endpoints and dependency injection in Python"
})

// Data Processing
context7_get_library_docs({
  query: "pandas DataFrame operations and transformations in Python"
})

// Testing
context7_get_library_docs({
  query: "pytest fixtures and parametrize decorators in Python"
})
```

## Query Best Practices

### ✅ DO: Be Specific

**Good:**
```javascript
context7_get_library_docs({
  query: "tokio async runtime with tracing instrumentation in Rust"
})
```

**Why:** Specific queries return targeted, relevant documentation.

### ❌ DON'T: Be Too Vague

**Bad:**
```javascript
context7_get_library_docs({
  query: "async runtime"  // Which language? Which library?
})
```

**Why:** Vague queries may return irrelevant or generic results.

### ✅ DO: Include Context

**Good:**
```javascript
context7_get_library_docs({
  query: "React Server Components data fetching patterns in Next.js 14 with TypeScript"
})
```

**Why:** Context helps Context7 find the most relevant documentation sections.

### ❌ DON'T: Ask Multiple Questions

**Bad:**
```javascript
context7_get_library_docs({
  query: "tokio runtime and serde serialization and axum routing in Rust"
})
```

**Why:** Make separate queries for different topics to get better results.

### ✅ DO: Specify Versions When Relevant

**Good:**
```javascript
context7_get_library_docs({
  query: "Next.js 14 app router middleware patterns with TypeScript"
})
```

**Why:** Version-specific queries ensure you get the right documentation.

## Common Use Cases

### 1. Starting a New Feature

**Scenario:** You need to implement JWT authentication in Rust

```javascript
// First, understand the library
context7_get_library_docs({
  query: "jsonwebtoken JWT encoding and decoding in Rust"
})

// Then, check integration patterns
context7_get_library_docs({
  query: "axum JWT authentication middleware in Rust"
})
```

### 2. Debugging an Error

**Scenario:** You're getting a type error with React Query

```javascript
// Check the correct types
context7_get_library_docs({
  query: "React Query useQuery TypeScript types and generics"
})
```

### 3. Updating Dependencies

**Scenario:** You're upgrading from Next.js 13 to 14

```javascript
// Check for breaking changes
context7_get_library_docs({
  query: "Next.js 14 migration guide from Next.js 13 breaking changes"
})
```

### 4. Learning a New Library

**Scenario:** You need to use a library you haven't used before

```javascript
// Start with basics
context7_get_library_docs({
  query: "sqlx getting started async PostgreSQL in Rust"
})

// Then dive deeper
context7_get_library_docs({
  query: "sqlx query macros and compile-time verification in Rust"
})
```

## Integration with Your Workflow

### Step 1: Identify the Need

Before implementing, check if you need current documentation:
- Is this a library you're unfamiliar with?
- Has this library been updated recently?
- Are you unsure about the API?

### Step 2: Query Context7

Use specific, language-aware queries to get documentation.

### Step 3: Apply the Knowledge

Use the documentation to write correct, idiomatic code.

### Step 4: Verify

Test your implementation to ensure it works as expected.

## Agent-Specific Guidelines

### Rex (Rust Implementation Agent)

**Primary Use Cases:**
- Tokio async patterns
- Serde serialization
- Axum/Actix web frameworks
- Database libraries (sqlx, diesel)
- Error handling (anyhow, thiserror)

**Example Workflow:**
```javascript
// 1. Check current best practices
context7_get_library_docs({
  query: "tokio async runtime best practices in Rust 2024"
})

// 2. Implement based on documentation
// 3. Verify with tests
```

### Blaze (Frontend Agent)

**Primary Use Cases:**
- React patterns and hooks
- Next.js routing and features
- UI component libraries (shadcn/ui)
- State management (Zustand, React Query)
- TypeScript patterns

**Example Workflow:**
```javascript
// 1. Check component API
context7_get_library_docs({
  query: "shadcn/ui dialog component with form validation TypeScript"
})

// 2. Implement component
// 3. Test in browser
```

### Cleo (Code Quality Agent)

**Primary Use Cases:**
- Linting configurations
- Testing frameworks
- Code formatting tools
- Quality tools (clippy, eslint)

**Example Workflow:**
```javascript
// Check latest linting rules
context7_get_library_docs({
  query: "clippy pedantic lints recommended settings Rust"
})
```

### Tess (QA/Testing Agent)

**Primary Use Cases:**
- Testing frameworks
- E2E testing tools
- Assertion libraries
- Mocking patterns

**Example Workflow:**
```javascript
// Check testing patterns
context7_get_library_docs({
  query: "pytest async fixtures and mocking in Python"
})
```

### Cipher (Security Agent)

**Primary Use Cases:**
- Security libraries
- Authentication patterns
- Encryption libraries
- Security best practices

**Example Workflow:**
```javascript
// Check security patterns
context7_get_library_docs({
  query: "JWT authentication security best practices in Rust"
})
```

## Troubleshooting

### No Results Returned

**Problem:** Query returns no relevant documentation

**Solutions:**
1. Make query more specific
2. Include the programming language
3. Try alternative library names
4. Check spelling of library name

### Outdated Information

**Problem:** Documentation seems outdated

**Solutions:**
1. Include version number in query
2. Add "latest" or "2024" to query
3. Query for "migration guide" or "what's new"

### Too Many Results

**Problem:** Too much documentation returned

**Solutions:**
1. Be more specific about what you need
2. Focus on one aspect at a time
3. Include use case context in query

## Tips for Success

1. **Always include the language** - "in Rust", "with TypeScript", "in Python"
2. **Be specific about versions** - "Next.js 14", "React 18", "tokio 1.x"
3. **Focus on one topic per query** - Don't combine multiple questions
4. **Include your use case** - "for authentication", "for data fetching"
5. **Query before implementing** - Check documentation first, code second
6. **Verify with tests** - Documentation is a guide, not gospel

## Example Prompt Integration

When you're implementing a feature, follow this pattern:

```
I need to implement [feature] using [library].

1. First, let me check the current documentation:
   context7_get_library_docs({
     query: "[library] [feature] in [language]"
   })

2. Based on the documentation, I'll implement:
   [code implementation]

3. I'll verify it works with:
   [test/verification approach]
```

## Summary

Context7 is your **real-time documentation assistant**. Use it to:
- ✅ Get current, accurate library documentation
- ✅ Find version-specific information
- ✅ Learn new libraries quickly
- ✅ Verify API usage patterns
- ✅ Check for breaking changes

Remember: **Specific, language-aware queries** get the best results!

---

**Quick Reference:**

```javascript
// Template
context7_get_library_docs({
  query: "[library] [specific feature] in [language]"
})

// Rust Example
context7_get_library_docs({
  query: "tokio async runtime setup in Rust"
})

// TypeScript Example
context7_get_library_docs({
  query: "React hooks useState with TypeScript"
})
```

