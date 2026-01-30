/**
 * Stitch test fixture for Nova (Node.js/Bun backend agent)
 * 
 * This file contains intentional issues for testing remediation:
 * - Type safety issues
 * - Missing error handling
 * - ESLint warnings
 */

import { Elysia } from 'elysia'
import { Effect, pipe } from 'effect'
import * as Schema from '@effect/schema/Schema'

// User schema with Effect Schema
const User = Schema.Struct({
  id: Schema.Number,
  name: Schema.String,
  email: Schema.String,
})

type User = Schema.Schema.Type<typeof User>

// TODO: Intentional issue - any type
const users: any[] = [
  { id: 1, name: 'Alice', email: 'alice@example.com' },
  { id: 2, name: 'Bob', email: 'bob@example.com' },
]

// TODO: Intentional issue - unused variable
const unusedConfig = {
  debug: true,
  maxConnections: 100,
}

// TODO: Intentional issue - no error handling
const getUsers = () => {
  return users
}

// TODO: Intentional issue - unsafe type assertion
const getUserById = (id: string): User => {
  const user = users.find(u => u.id === parseInt(id))
  return user as User // unsafe - could be undefined
}

const app = new Elysia()
  .get('/users', () => getUsers())
  .get('/users/:id', ({ params }) => getUserById(params.id))
  .listen(3000)

console.log(`Nova server running at ${app.server?.hostname}:${app.server?.port}`)
