// Code smells for Blaze to find:
// - missing key prop in list
// - inline styles
// - any types
// - useEffect with missing deps
// - direct DOM manipulation

import React, { useEffect, useState } from 'react';

interface User {
  id: number;
  name: string;
}

export function UserList() {
  const [users, setUsers] = useState<any[]>([]);
  const [count, setCount] = useState(0);

  // Missing dependency array items
  useEffect(() => {
    fetch('/api/users')
      .then(res => res.json())
      .then(data => setUsers(data));
    
    // Direct DOM manipulation - bad in React
    document.title = `${count} users`;
  }, []);

  // Inline styles - should use Tailwind
  return (
    <div style={{ padding: '20px', backgroundColor: '#f0f0f0' }}>
      <h1 style={{ color: 'blue', fontSize: '24px' }}>Users</h1>
      {/* Missing key prop */}
      {users.map((user) => (
        <div style={{ margin: '10px' }}>
          <span>{user.name}</span>
        </div>
      ))}
      <button onClick={() => setCount(count + 1)}>
        Clicked {count} times
      </button>
    </div>
  );
}
