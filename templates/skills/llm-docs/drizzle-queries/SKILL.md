# Drizzle ORM Queries

Query patterns and best practices for Drizzle ORM with PostgreSQL.

## When to Use

- Building database queries with Drizzle ORM
- Implementing CRUD operations
- Writing complex queries with joins and aggregations
- Optimizing database performance

## Basic Queries

### Select

```typescript
import { db } from './db';
import { users, posts } from './schema';
import { eq, and, or, gt, lt, like, desc, asc } from 'drizzle-orm';

// Select all
const allUsers = await db.select().from(users);

// Select specific columns
const names = await db.select({
  id: users.id,
  name: users.name,
}).from(users);

// With conditions
const activeUsers = await db.select()
  .from(users)
  .where(eq(users.status, 'active'));

// Multiple conditions
const results = await db.select()
  .from(users)
  .where(
    and(
      eq(users.role, 'admin'),
      gt(users.createdAt, new Date('2024-01-01'))
    )
  );

// OR conditions
const filtered = await db.select()
  .from(users)
  .where(
    or(
      eq(users.status, 'active'),
      eq(users.status, 'pending')
    )
  );
```

### Insert

```typescript
// Single insert
const newUser = await db.insert(users)
  .values({
    name: 'John',
    email: 'john@example.com',
  })
  .returning();

// Bulk insert
const newUsers = await db.insert(users)
  .values([
    { name: 'Alice', email: 'alice@example.com' },
    { name: 'Bob', email: 'bob@example.com' },
  ])
  .returning();

// Upsert (on conflict)
await db.insert(users)
  .values({ email: 'john@example.com', name: 'John Updated' })
  .onConflictDoUpdate({
    target: users.email,
    set: { name: 'John Updated' },
  });
```

### Update

```typescript
// Update with condition
await db.update(users)
  .set({ status: 'inactive' })
  .where(eq(users.id, 1));

// Update multiple fields
await db.update(users)
  .set({
    name: 'New Name',
    updatedAt: new Date(),
  })
  .where(eq(users.id, 1))
  .returning();
```

### Delete

```typescript
// Delete with condition
await db.delete(users)
  .where(eq(users.id, 1));

// Soft delete pattern
await db.update(users)
  .set({ deletedAt: new Date() })
  .where(eq(users.id, 1));
```

## Joins

### Inner Join

```typescript
const usersWithPosts = await db.select({
  user: users,
  post: posts,
})
.from(users)
.innerJoin(posts, eq(users.id, posts.authorId));
```

### Left Join

```typescript
const usersWithOptionalPosts = await db.select({
  user: users,
  post: posts,
})
.from(users)
.leftJoin(posts, eq(users.id, posts.authorId));
```

### Multiple Joins

```typescript
const fullData = await db.select({
  user: users,
  post: posts,
  comment: comments,
})
.from(users)
.leftJoin(posts, eq(users.id, posts.authorId))
.leftJoin(comments, eq(posts.id, comments.postId));
```

## Aggregations

### Count

```typescript
import { count, sum, avg, max, min } from 'drizzle-orm';

const userCount = await db.select({
  count: count(),
}).from(users);

// Count with condition
const activeCount = await db.select({
  count: count(),
})
.from(users)
.where(eq(users.status, 'active'));
```

### Group By

```typescript
const postsByAuthor = await db.select({
  authorId: posts.authorId,
  postCount: count(),
})
.from(posts)
.groupBy(posts.authorId);

// With having
const prolificAuthors = await db.select({
  authorId: posts.authorId,
  postCount: count(),
})
.from(posts)
.groupBy(posts.authorId)
.having(gt(count(), 5));
```

## Ordering and Pagination

```typescript
// Order by
const ordered = await db.select()
  .from(users)
  .orderBy(desc(users.createdAt));

// Multiple order columns
const sorted = await db.select()
  .from(users)
  .orderBy(asc(users.lastName), desc(users.createdAt));

// Pagination
const page = 1;
const pageSize = 20;

const paginated = await db.select()
  .from(users)
  .orderBy(desc(users.createdAt))
  .limit(pageSize)
  .offset((page - 1) * pageSize);
```

## Transactions

```typescript
// Basic transaction
await db.transaction(async (tx) => {
  const [user] = await tx.insert(users)
    .values({ name: 'New User', email: 'new@example.com' })
    .returning();

  await tx.insert(profiles)
    .values({ userId: user.id, bio: 'Hello!' });
});

// With rollback
try {
  await db.transaction(async (tx) => {
    await tx.insert(users).values({ name: 'Test' });
    
    if (someCondition) {
      tx.rollback(); // Explicit rollback
    }
    
    await tx.insert(posts).values({ title: 'Test Post' });
  });
} catch (e) {
  // Transaction was rolled back
}
```

## Prepared Statements

```typescript
// Prepare for reuse
const getUserById = db.query.users
  .findFirst({
    where: eq(users.id, placeholder('id')),
  })
  .prepare();

// Execute with parameter
const user = await getUserById.execute({ id: 1 });
```

## Query Builder vs Relational Queries

### Query Builder (SQL-like)

```typescript
// More control, SQL-like syntax
const result = await db.select()
  .from(users)
  .leftJoin(posts, eq(users.id, posts.authorId))
  .where(eq(users.status, 'active'));
```

### Relational Queries (with relations)

```typescript
// Define relations in schema
export const usersRelations = relations(users, ({ many }) => ({
  posts: many(posts),
}));

// Query with relations
const usersWithPosts = await db.query.users.findMany({
  with: {
    posts: true,
  },
  where: eq(users.status, 'active'),
});
```

## Best Practices

1. **Use `returning()`**: Get inserted/updated data without extra query
2. **Use transactions**: For operations that need to be atomic
3. **Index your queries**: Add indexes for frequently filtered columns
4. **Use prepared statements**: For repeated queries with different params
5. **Prefer relational queries**: When fetching nested data
6. **Handle null**: Use `leftJoin` when related data might not exist
