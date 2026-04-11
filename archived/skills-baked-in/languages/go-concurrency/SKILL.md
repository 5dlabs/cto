# Go Concurrency Patterns

Common concurrency patterns and best practices in Go.

## When to Use

- Implementing concurrent operations in Go
- Working with goroutines and channels
- Building scalable, concurrent systems
- Avoiding common concurrency bugs

## Fundamental Patterns

### Worker Pool

```go
func workerPool(jobs <-chan int, results chan<- int, numWorkers int) {
    var wg sync.WaitGroup
    
    for i := 0; i < numWorkers; i++ {
        wg.Add(1)
        go func(workerID int) {
            defer wg.Done()
            for job := range jobs {
                results <- process(job)
            }
        }(i)
    }
    
    // Wait for all workers and close results
    go func() {
        wg.Wait()
        close(results)
    }()
}

// Usage
jobs := make(chan int, 100)
results := make(chan int, 100)

workerPool(jobs, results, 5)

// Send jobs
for i := 0; i < 100; i++ {
    jobs <- i
}
close(jobs)

// Collect results
for result := range results {
    fmt.Println(result)
}
```

### Fan-Out, Fan-In

```go
// Fan-out: multiple goroutines read from same channel
func fanOut(in <-chan int, numWorkers int) []<-chan int {
    outputs := make([]<-chan int, numWorkers)
    for i := 0; i < numWorkers; i++ {
        outputs[i] = worker(in)
    }
    return outputs
}

// Fan-in: merge multiple channels into one
func fanIn(inputs ...<-chan int) <-chan int {
    out := make(chan int)
    var wg sync.WaitGroup
    
    for _, in := range inputs {
        wg.Add(1)
        go func(ch <-chan int) {
            defer wg.Done()
            for v := range ch {
                out <- v
            }
        }(in)
    }
    
    go func() {
        wg.Wait()
        close(out)
    }()
    
    return out
}
```

### Pipeline

```go
func generator(nums ...int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for _, n := range nums {
            out <- n
        }
    }()
    return out
}

func square(in <-chan int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for n := range in {
            out <- n * n
        }
    }()
    return out
}

func double(in <-chan int) <-chan int {
    out := make(chan int)
    go func() {
        defer close(out)
        for n := range in {
            out <- n * 2
        }
    }()
    return out
}

// Usage
result := double(square(generator(1, 2, 3, 4)))
for v := range result {
    fmt.Println(v) // 2, 8, 18, 32
}
```

## Context Patterns

### With Cancellation

```go
func doWork(ctx context.Context) error {
    for {
        select {
        case <-ctx.Done():
            return ctx.Err()
        default:
            // Do work
            if err := processItem(); err != nil {
                return err
            }
        }
    }
}

// Usage
ctx, cancel := context.WithCancel(context.Background())
defer cancel()

go func() {
    if err := doWork(ctx); err != nil {
        log.Printf("work failed: %v", err)
    }
}()

// Cancel when needed
cancel()
```

### With Timeout

```go
func fetchWithTimeout(url string) ([]byte, error) {
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()
    
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return nil, err
    }
    
    resp, err := http.DefaultClient.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()
    
    return io.ReadAll(resp.Body)
}
```

## Synchronization

### sync.WaitGroup

```go
func processItems(items []Item) {
    var wg sync.WaitGroup
    
    for _, item := range items {
        wg.Add(1)
        go func(it Item) {
            defer wg.Done()
            process(it)
        }(item)
    }
    
    wg.Wait()
}
```

### sync.Once

```go
var (
    instance *Database
    once     sync.Once
)

func GetDB() *Database {
    once.Do(func() {
        instance = connectToDatabase()
    })
    return instance
}
```

### sync.Mutex

```go
type SafeCounter struct {
    mu    sync.Mutex
    value int
}

func (c *SafeCounter) Inc() {
    c.mu.Lock()
    defer c.mu.Unlock()
    c.value++
}

func (c *SafeCounter) Value() int {
    c.mu.Lock()
    defer c.mu.Unlock()
    return c.value
}
```

### sync.RWMutex

```go
type SafeMap struct {
    mu sync.RWMutex
    m  map[string]int
}

func (sm *SafeMap) Get(key string) (int, bool) {
    sm.mu.RLock()
    defer sm.mu.RUnlock()
    v, ok := sm.m[key]
    return v, ok
}

func (sm *SafeMap) Set(key string, value int) {
    sm.mu.Lock()
    defer sm.mu.Unlock()
    sm.m[key] = value
}
```

## Select Patterns

### Timeout Pattern

```go
select {
case result := <-resultCh:
    return result, nil
case <-time.After(5 * time.Second):
    return nil, errors.New("timeout")
}
```

### Non-blocking Send/Receive

```go
// Non-blocking receive
select {
case msg := <-ch:
    fmt.Println(msg)
default:
    fmt.Println("no message")
}

// Non-blocking send
select {
case ch <- msg:
    fmt.Println("sent")
default:
    fmt.Println("channel full")
}
```

## Common Pitfalls

### Goroutine Leak

```go
// BAD: Goroutine leaks if ctx is never cancelled
func badFetch(ctx context.Context, url string) {
    go func() {
        resp, _ := http.Get(url)
        // If ctx is cancelled, this goroutine hangs forever
    }()
}

// GOOD: Goroutine respects cancellation
func goodFetch(ctx context.Context, url string) error {
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return err
    }
    resp, err := http.DefaultClient.Do(req)
    if err != nil {
        return err
    }
    defer resp.Body.Close()
    return nil
}
```

### Closure Variable Capture

```go
// BAD: All goroutines see same i
for i := 0; i < 10; i++ {
    go func() {
        fmt.Println(i) // Usually prints 10
    }()
}

// GOOD: Pass as parameter
for i := 0; i < 10; i++ {
    go func(n int) {
        fmt.Println(n)
    }(i)
}
```

## Best Practices

1. **Close channels from sender**: The goroutine that sends should close
2. **Use context for cancellation**: Propagate context through call chain
3. **Avoid shared memory**: Prefer channels for communication
4. **Keep critical sections small**: Lock only what needs locking
5. **Use `errgroup` for error handling**: `golang.org/x/sync/errgroup`
6. **Test with `-race`**: `go test -race ./...`
