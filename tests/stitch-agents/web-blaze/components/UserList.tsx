/**
 * Stitch test fixture for Blaze (React/Next.js frontend agent)
 * 
 * This file contains intentional issues for testing remediation:
 * - Missing key prop
 * - useEffect dependency issues
 * - Type errors
 */

'use client'

import { useState, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'

interface User {
  id: number
  name: string
  email: string
}

// TODO: Intentional issue - any type
async function fetchUsers(): Promise<any> {
  const response = await fetch('/api/users')
  return response.json()
}

export function UserList() {
  const [filter, setFilter] = useState('')
  const [count, setCount] = useState(0)
  
  // TODO: Intentional issue - missing dependency in useEffect
  useEffect(() => {
    console.log('Filter changed:', filter)
    setCount(count + 1)
  }, []) // missing filter and count dependencies

  const { data: users, isLoading, error } = useQuery({
    queryKey: ['users'],
    queryFn: fetchUsers,
  })

  if (isLoading) return <div>Loading...</div>
  if (error) return <div>Error loading users</div>

  // TODO: Intentional issue - missing key prop
  return (
    <div className="p-4">
      <input
        type="text"
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        placeholder="Filter users..."
        className="border p-2 rounded"
      />
      <ul className="mt-4 space-y-2">
        {users?.map((user: User) => (
          <li className="p-2 bg-gray-100 rounded">
            {user.name} - {user.email}
          </li>
        ))}
      </ul>
    </div>
  )
}
