# Go Code Review

Best practices and common issues to check when reviewing Go code.

## When to Use

- Reviewing Go code for the Grizz agent
- Conducting code quality assessments
- Preparing code for production
- Learning Go idioms and best practices

## Error Handling

### Check All Errors

```go
// BAD: Ignoring error
result, _ := doSomething()

// GOOD: Handle error
result, err := doSomething()
if err != nil {
    return fmt.Errorf("failed to do something: %w", err)
}
```

### Error Wrapping

```go
// BAD: Losing context
if err != nil {
    return err
}

// GOOD: Add context
if err != nil {
    return fmt.Errorf("processing user %s: %w", userID, err)
}
```

### Sentinel Errors

```go
// Define sentinel errors at package level
var (
    ErrNotFound = errors.New("not found")
    ErrInvalid  = errors.New("invalid input")
)

// Check with errors.Is
if errors.Is(err, ErrNotFound) {
    // handle not found
}
```

## Concurrency

### Data Races

```go
// BAD: Data race
var counter int
go func() { counter++ }()
go func() { counter++ }()

// GOOD: Use mutex or atomic
var counter atomic.Int64
go func() { counter.Add(1) }()
go func() { counter.Add(1) }()
```

### Channel Best Practices

```go
// Always close channels from sender side
ch := make(chan int)
go func() {
    defer close(ch) // Sender closes
    for i := 0; i < 10; i++ {
        ch <- i
    }
}()

// Range over channel
for v := range ch {
    fmt.Println(v)
}
```

### Context Propagation

```go
// BAD: Ignoring context
func DoWork(ctx context.Context) error {
    // context not used
    return expensiveOperation()
}

// GOOD: Respect context
func DoWork(ctx context.Context) error {
    select {
    case <-ctx.Done():
        return ctx.Err()
    default:
    }
    return expensiveOperation()
}
```

## Resource Management

### defer for Cleanup

```go
// BAD: May not close on early return
func readFile(path string) error {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    // What if processing fails?
    process(f)
    f.Close()
    return nil
}

// GOOD: defer ensures cleanup
func readFile(path string) error {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    defer f.Close()
    return process(f)
}
```

### HTTP Clients

```go
// BAD: Using default client (no timeout)
resp, err := http.Get(url)

// GOOD: Client with timeout
client := &http.Client{
    Timeout: 10 * time.Second,
}
resp, err := client.Get(url)
```

### Response Body

```go
// ALWAYS close response body
resp, err := http.Get(url)
if err != nil {
    return err
}
defer resp.Body.Close()

// Read body
body, err := io.ReadAll(resp.Body)
```

## Struct Design

### Use Meaningful Zero Values

```go
// GOOD: Zero value is useful
type Config struct {
    Host string // defaults to ""
    Port int    // defaults to 0, might need explicit default
}

func NewConfig() Config {
    return Config{
        Host: "localhost",
        Port: 8080, // explicit default
    }
}
```

### Embed for Composition

```go
// Instead of inheritance, use embedding
type Logger struct {
    // ...
}

type Server struct {
    Logger // Embedded, Server has Logger methods
    addr   string
}
```

## Common Review Checklist

### Must Check

- [ ] All errors are handled (no `_` for errors)
- [ ] Errors have context when wrapped
- [ ] `defer` used for cleanup (files, locks, connections)
- [ ] Context propagated and respected
- [ ] HTTP response bodies closed
- [ ] Goroutine leaks avoided (channels closed, context cancelled)

### Should Check

- [ ] Exported functions have doc comments
- [ ] Package names are lowercase, single word
- [ ] Interface names end in `-er` when appropriate
- [ ] Tests cover error paths, not just happy path
- [ ] No `panic` in library code
- [ ] Slice/map nil checks before access

### Nice to Have

- [ ] Consistent naming conventions
- [ ] Functions are small and focused
- [ ] Complex logic has comments
- [ ] No magic numbers (use constants)
- [ ] Table-driven tests where appropriate

## Code Smells

| Smell | Problem | Fix |
|-------|---------|-----|
| `interface{}` everywhere | Type safety lost | Use generics or specific types |
| Global variables | Hard to test | Dependency injection |
| Long functions | Hard to understand | Extract helper functions |
| Deep nesting | Hard to follow | Early returns |
| Commented code | Clutter | Delete it (git has history) |
| `panic` in packages | Crashes caller | Return errors instead |
