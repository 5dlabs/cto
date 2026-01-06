---
name: webapp-testing
description: Patterns for testing local web applications using Playwright. Use when verifying frontend functionality, debugging UI behavior, capturing screenshots, or testing dynamic web apps. Provides decision tree for choosing approach, reconnaissance-then-action pattern, and multi-server testing.
---

# Web Application Testing

Test local web applications using Playwright with a systematic approach.

## Decision Tree: Choose Your Approach

```
User task → Is it static HTML?
    │
    ├─ Yes → Read HTML file directly to identify selectors
    │         └─ Write Playwright script using discovered selectors
    │
    └─ No (dynamic webapp) → Is the server already running?
            │
            ├─ No → Start server first, then test
            │       (use process management or test framework)
            │
            └─ Yes → Use Reconnaissance-Then-Action pattern:
                    1. Navigate and wait for networkidle
                    2. Take screenshot or inspect DOM
                    3. Identify selectors from rendered state
                    4. Execute actions with discovered selectors
```

## Reconnaissance-Then-Action Pattern

For dynamic apps, **always inspect before acting**:

### Step 1: Navigate and Wait

```python
from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page()
    page.goto('http://localhost:5173')
    
    # CRITICAL: Wait for JS to execute
    page.wait_for_load_state('networkidle')
```

### Step 2: Inspect Rendered DOM

```python
# Option A: Screenshot for visual inspection
page.screenshot(path='/tmp/inspect.png', full_page=True)

# Option B: Get page content
content = page.content()

# Option C: Find specific elements
buttons = page.locator('button').all()
for btn in buttons:
    print(btn.text_content())
```

### Step 3: Identify Selectors

From inspection results, determine the right selectors:

| Selector Type | Example | When to Use |
|--------------|---------|-------------|
| Text | `text=Submit` | Visible button/link text |
| Role | `role=button[name="Submit"]` | Accessibility-friendly |
| CSS | `#submit-btn`, `.primary-action` | Unique IDs or classes |
| Data attributes | `[data-testid="submit"]` | Test-specific attributes |

### Step 4: Execute Actions

```python
# Now interact with discovered selectors
page.locator('text=Submit').click()
page.locator('#email').fill('test@example.com')
page.locator('form').press('Enter')
```

## Common Pitfall

❌ **Don't** inspect DOM before waiting for `networkidle` on dynamic apps
✅ **Do** wait for `page.wait_for_load_state('networkidle')` before inspection

Without this wait, you'll see the initial HTML before JavaScript renders the actual UI.

## Multi-Server Testing

When testing apps with separate frontend and backend:

```python
import subprocess
import time

# Start backend
backend = subprocess.Popen(['python', 'server.py'], cwd='backend/')

# Start frontend  
frontend = subprocess.Popen(['npm', 'run', 'dev'], cwd='frontend/')

# Wait for servers to be ready
time.sleep(5)  # Or use health check polling

try:
    # Run your tests
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        page.goto('http://localhost:5173')
        # ... test logic
finally:
    backend.terminate()
    frontend.terminate()
```

## Testing Patterns

### Form Testing

```python
# Fill form
page.locator('#name').fill('Test User')
page.locator('#email').fill('test@example.com')
page.locator('select#country').select_option('US')
page.locator('input[type="checkbox"]').check()

# Submit and verify
page.locator('button[type="submit"]').click()
page.wait_for_selector('.success-message')
assert page.locator('.success-message').is_visible()
```

### Navigation Testing

```python
# Click link and verify navigation
page.locator('text=About').click()
page.wait_for_url('**/about')
assert 'About' in page.title()
```

### API Response Testing

```python
# Intercept network requests
with page.expect_response('**/api/users') as response_info:
    page.locator('button.load-users').click()

response = response_info.value
assert response.status == 200
data = response.json()
assert len(data['users']) > 0
```

### Visual Regression

```python
# Compare screenshots
page.goto('http://localhost:5173/dashboard')
page.wait_for_load_state('networkidle')

# Take screenshot for comparison
page.screenshot(path='dashboard-current.png', full_page=True)

# Compare with baseline (use image diff tool)
```

## Console & Error Monitoring

```python
# Capture console messages
console_messages = []
page.on('console', lambda msg: console_messages.append(msg.text))

# Capture errors
errors = []
page.on('pageerror', lambda err: errors.append(str(err)))

# Run test
page.goto('http://localhost:5173')
page.wait_for_load_state('networkidle')

# Check for issues
assert len(errors) == 0, f"Page errors: {errors}"
assert not any('error' in msg.lower() for msg in console_messages)
```

## Wait Strategies

| Method | Use When |
|--------|----------|
| `wait_for_load_state('networkidle')` | Initial page load, SPA navigation |
| `wait_for_selector('.element')` | Waiting for specific element to appear |
| `wait_for_url('**/path')` | After navigation actions |
| `wait_for_response('**/api/**')` | After triggering API calls |
| `wait_for_timeout(1000)` | Last resort for timing-dependent UIs |

## Best Practices

1. **Always launch headless**: `browser = p.chromium.launch(headless=True)`
2. **Always close browser**: Use `with` context or explicit `browser.close()`
3. **Use descriptive selectors**: Prefer `text=`, `role=`, or `data-testid=` over fragile CSS
4. **Add appropriate waits**: Don't assume elements are immediately available
5. **Capture evidence**: Screenshot on failures for debugging
6. **Isolate tests**: Each test should set up its own state

## Debugging Tips

```python
# Slow down for debugging
browser = p.chromium.launch(headless=False, slow_mo=500)

# Pause for manual inspection
page.pause()

# Get element state
element = page.locator('#my-button')
print(f"Visible: {element.is_visible()}")
print(f"Enabled: {element.is_enabled()}")
print(f"Text: {element.text_content()}")
```
