---
name: react-best-practices
description: React and Next.js performance optimization guidelines. Use when writing, reviewing, or refactoring React/Next.js code. Applies to Blaze (web), Spark (desktop), and Tap (mobile) agents.
---

# React Best Practices

Comprehensive performance optimization guide for React and Next.js applications. Contains 45+ rules across 8 categories, prioritized by impact.

## When to Use

Reference these guidelines when:

- Writing new React components or Next.js pages
- Implementing data fetching (client or server-side)
- Reviewing code for performance issues
- Refactoring existing React/Next.js code
- Optimizing bundle size or load times

**Relevant Agents:** Blaze (React/Next.js), Spark (Electron), Tap (Expo/React Native)

---

## Rule Categories by Priority

| Priority | Category | Impact |
|----------|----------|--------|
| 1 | Eliminating Waterfalls | CRITICAL |
| 2 | Bundle Size Optimization | CRITICAL |
| 3 | Server-Side Performance | HIGH |
| 4 | Client-Side Data Fetching | MEDIUM-HIGH |
| 5 | Re-render Optimization | MEDIUM |
| 6 | Rendering Performance | MEDIUM |
| 7 | JavaScript Performance | LOW-MEDIUM |

---

## 1. Eliminating Waterfalls (CRITICAL)

Waterfalls are the #1 performance killer. Fix these first.

### Move await into branches

**Bad:**

```typescript
async function handleRequest(userId: string, skipProcessing: boolean) {
  const userData = await fetchUserData(userId);  // Always waits

  if (skipProcessing) {
    return { skipped: true };  // Waited for nothing
  }

  return processUserData(userData);
}
```

**Good:**

```typescript
async function handleRequest(userId: string, skipProcessing: boolean) {
  if (skipProcessing) {
    return { skipped: true };  // Returns immediately
  }

  const userData = await fetchUserData(userId);
  return processUserData(userData);
}
```

### Parallelize independent operations

**Bad:**

```typescript
const user = await fetchUser(id);
const posts = await fetchPosts(id);
const comments = await fetchComments(id);
// Total time: user + posts + comments
```

**Good:**

```typescript
const [user, posts, comments] = await Promise.all([
  fetchUser(id),
  fetchPosts(id),
  fetchComments(id),
]);
// Total time: max(user, posts, comments)
```

### Start promises early, await late

**Bad:**

```typescript
export async function GET() {
  const data = await fetchData();  // Blocks immediately
  const transformed = transform(data);
  return Response.json(transformed);
}
```

**Good:**

```typescript
export async function GET() {
  const dataPromise = fetchData();  // Start immediately
  // ... do other setup work ...
  const data = await dataPromise;   // Await when needed
  return Response.json(transform(data));
}
```

### Use Suspense boundaries for streaming

```tsx
<Suspense fallback={<Loading />}>
  <SlowComponent />
</Suspense>
```

---

## 2. Bundle Size Optimization (CRITICAL)

Every KB matters for initial load.

### Import directly, avoid barrel files

**Bad:**

```typescript
import { Button } from '@/components';  // Pulls entire barrel
```

**Good:**

```typescript
import { Button } from '@/components/Button';  // Only Button
```

### Use dynamic imports for heavy components

**Bad:**

```typescript
import { HeavyChart } from './HeavyChart';

function Dashboard() {
  return showChart ? <HeavyChart /> : null;
}
```

**Good:**

```typescript
import dynamic from 'next/dynamic';

const HeavyChart = dynamic(() => import('./HeavyChart'), {
  loading: () => <ChartSkeleton />,
});

function Dashboard() {
  return showChart ? <HeavyChart /> : null;
}
```

### Defer third-party scripts

**Bad:**

```typescript
import { Analytics } from '@analytics/lib';

function App() {
  useEffect(() => {
    Analytics.init();  // Blocks hydration
  }, []);
}
```

**Good:**

```typescript
function App() {
  useEffect(() => {
    // Load after hydration
    import('@analytics/lib').then(({ Analytics }) => {
      Analytics.init();
    });
  }, []);
}
```

### Preload on hover/focus

```typescript
function Link({ href, children }) {
  const preload = () => {
    const link = document.createElement('link');
    link.rel = 'prefetch';
    link.href = href;
    document.head.appendChild(link);
  };

  return (
    <a href={href} onMouseEnter={preload} onFocus={preload}>
      {children}
    </a>
  );
}
```

---

## 3. Server-Side Performance (HIGH)

### Use React.cache() for per-request deduplication

```typescript
import { cache } from 'react';

const getUser = cache(async (id: string) => {
  return await db.user.findUnique({ where: { id } });
});

// Multiple components can call getUser(id) - only one DB query
```

### Minimize data passed to client components

**Bad:**

```typescript
// Server Component
async function UserPage({ id }) {
  const user = await getFullUser(id);  // 50 fields
  return <ClientProfile user={user} />;
}
```

**Good:**

```typescript
// Server Component
async function UserPage({ id }) {
  const user = await getFullUser(id);
  return (
    <ClientProfile
      name={user.name}
      avatar={user.avatar}
      // Only what client needs
    />
  );
}
```

### Use after() for non-blocking operations

```typescript
import { after } from 'next/server';

export async function POST(request: Request) {
  const data = await request.json();
  const result = await saveToDb(data);

  after(async () => {
    await sendAnalytics(result);
    await notifyWebhooks(result);
  });

  return Response.json(result);  // Returns immediately
}
```

---

## 4. Client-Side Data Fetching (MEDIUM-HIGH)

### Use SWR for automatic deduplication

```typescript
import useSWR from 'swr';

function useUser(id: string) {
  return useSWR(`/api/users/${id}`, fetcher, {
    dedupingInterval: 2000,  // Dedup requests within 2s
  });
}

// Multiple components using useUser(same-id) = one request
```

### Deduplicate global event listeners

**Bad:**

```typescript
function Component() {
  useEffect(() => {
    window.addEventListener('resize', handler);  // Each instance adds one
    return () => window.removeEventListener('resize', handler);
  }, []);
}
```

**Good:**

```typescript
// Shared hook with ref counting
const listeners = new Set();

function useWindowResize(handler: () => void) {
  useEffect(() => {
    listeners.add(handler);
    if (listeners.size === 1) {
      window.addEventListener('resize', notifyAll);
    }
    return () => {
      listeners.delete(handler);
      if (listeners.size === 0) {
        window.removeEventListener('resize', notifyAll);
      }
    };
  }, [handler]);
}
```

---

## 5. Re-render Optimization (MEDIUM)

### Don't subscribe to state only used in callbacks

**Bad:**

```typescript
function Form() {
  const [value, setValue] = useState('');  // Re-renders on every keystroke

  const handleSubmit = () => {
    submitForm(value);
  };

  return <input onChange={e => setValue(e.target.value)} />;
}
```

**Good:**

```typescript
function Form() {
  const valueRef = useRef('');

  const handleSubmit = () => {
    submitForm(valueRef.current);
  };

  return <input onChange={e => { valueRef.current = e.target.value }} />;
}
```

### Extract expensive work into memoized components

**Bad:**

```typescript
function Parent({ data, filter }) {
  return (
    <div>
      <ExpensiveList data={data} />  {/* Re-renders when filter changes */}
      <Filter value={filter} />
    </div>
  );
}
```

**Good:**

```typescript
const MemoizedList = memo(ExpensiveList);

function Parent({ data, filter }) {
  return (
    <div>
      <MemoizedList data={data} />  {/* Only re-renders when data changes */}
      <Filter value={filter} />
    </div>
  );
}
```

### Use functional setState for stable callbacks

**Bad:**

```typescript
const increment = useCallback(() => {
  setCount(count + 1);  // Dependency on count
}, [count]);
```

**Good:**

```typescript
const increment = useCallback(() => {
  setCount(c => c + 1);  // No dependencies
}, []);
```

### Use startTransition for non-urgent updates

```typescript
import { startTransition } from 'react';

function SearchBox() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState([]);

  const handleChange = (e) => {
    setQuery(e.target.value);  // Urgent: update input

    startTransition(() => {
      setResults(search(e.target.value));  // Non-urgent: can be interrupted
    });
  };
}
```

---

## 6. Rendering Performance (MEDIUM)

### Animate wrapper divs, not SVG elements

**Bad:**

```tsx
<motion.svg animate={{ scale: 1.2 }}>  {/* Triggers SVG recalc */}
  <path d="..." />
</motion.svg>
```

**Good:**

```tsx
<motion.div animate={{ scale: 1.2 }}>  {/* GPU accelerated */}
  <svg><path d="..." /></svg>
</motion.div>
```

### Use content-visibility for long lists

```css
.list-item {
  content-visibility: auto;
  contain-intrinsic-size: 0 50px;
}
```

### Extract static JSX outside components

**Bad:**

```typescript
function Component() {
  return (
    <div>
      <header>Static Header</header>  {/* Recreated every render */}
      <DynamicContent />
    </div>
  );
}
```

**Good:**

```typescript
const StaticHeader = <header>Static Header</header>;

function Component() {
  return (
    <div>
      {StaticHeader}  {/* Same reference */}
      <DynamicContent />
    </div>
  );
}
```

### Use ternary, not && for conditionals

**Bad:**

```tsx
{items.length && <List items={items} />}  // Renders "0" when empty
```

**Good:**

```tsx
{items.length > 0 ? <List items={items} /> : null}
```

---

## 7. JavaScript Performance (LOW-MEDIUM)

### Build Map for repeated lookups

**Bad:**

```typescript
users.forEach(user => {
  const role = roles.find(r => r.userId === user.id);  // O(n) each time
});
```

**Good:**

```typescript
const roleMap = new Map(roles.map(r => [r.userId, r]));
users.forEach(user => {
  const role = roleMap.get(user.id);  // O(1)
});
```

### Combine multiple iterations

**Bad:**

```typescript
const active = users.filter(u => u.active);
const names = active.map(u => u.name);
const sorted = names.sort();
// 3 iterations
```

**Good:**

```typescript
const names = [];
for (const u of users) {
  if (u.active) names.push(u.name);
}
names.sort();
// 1 iteration + sort
```

### Check length before expensive operations

**Bad:**

```typescript
if (items.some(item => expensiveCheck(item))) { ... }
```

**Good:**

```typescript
if (items.length > 0 && items.some(item => expensiveCheck(item))) { ... }
```

### Use Set/Map for O(1) lookups

**Bad:**

```typescript
const isSelected = selectedIds.includes(id);  // O(n)
```

**Good:**

```typescript
const selectedSet = new Set(selectedIds);
const isSelected = selectedSet.has(id);  // O(1)
```

---

## Quick Reference Checklist

Before submitting React/Next.js code:

### Critical (Fix First)

- [ ] No sequential awaits for independent operations
- [ ] Dynamic imports for components >50KB
- [ ] No barrel file imports in hot paths
- [ ] Third-party scripts loaded after hydration

### High Priority

- [ ] Server components minimize client-passed data
- [ ] React.cache() for repeated data fetches
- [ ] Proper Suspense boundaries

### Medium Priority

- [ ] Memoized expensive components
- [ ] Stable callback references (functional setState)
- [ ] No unnecessary re-renders from state subscriptions

---

## Related Skills

- **test-driven-development** - TDD for React components
- **verification-before-completion** - Verify bundle size, performance metrics
