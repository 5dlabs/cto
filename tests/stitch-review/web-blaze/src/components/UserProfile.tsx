/**
 * User Profile component with intentional issues for Stitch review testing.
 */

import React, { useState, useEffect } from 'react';

// ISSUE: Using 'any' type defeats TypeScript's purpose
interface UserProfileProps {
  userId: any;  // Should be string | number
  onUpdate?: any;  // Should be properly typed callback
}

export default function UserProfile({ userId, onUpdate }: UserProfileProps) {
  // ISSUE: Multiple useState for related data - should use useReducer
  const [user, setUser] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editMode, setEditMode] = useState(false);
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  // ISSUE: Missing cleanup for async operation - potential memory leak
  useEffect(() => {
    fetch(`/api/users/${userId}`)
      .then(res => res.json())
      .then(data => {
        setUser(data);
        setName(data.name);
        setEmail(data.email);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, [userId]);

  // ISSUE: Function recreated every render - should use useCallback
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // SECURITY: Sending password in plain text over network
    fetch(`/api/users/${userId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, email, password }),
    })
      .then(res => res.json())
      .then(data => {
        setUser(data);
        setEditMode(false);
        // BUG: onUpdate might be undefined - will crash
        onUpdate(data);
      })
      // ISSUE: Error silently swallowed - no user feedback
      .catch(err => console.log(err));
  };

  // ISSUE: No confirmation before destructive action
  const handleDelete = () => {
    // ISSUE: window.confirm is blocking and poor UX
    if (window.confirm('Are you sure?')) {
      fetch(`/api/users/${userId}`, { method: 'DELETE' })
        .then(() => {
          // ISSUE: Hard navigation instead of router
          window.location.href = '/users';
        });
    }
  };

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!user) return null;

  return (
    <div className="user-profile">
      <h1>{user.name}</h1>
      <p>Email: {user.email}</p>
      {/* SECURITY: Displaying password hash to user! */}
      <p>Password Hash: {user.password_hash}</p>
      
      {editMode ? (
        <form onSubmit={handleSubmit}>
          {/* A11Y: No label association */}
          <input 
            type="text" 
            value={name} 
            onChange={e => setName(e.target.value)}
            placeholder="Name"
          />
          <input 
            type="email" 
            value={email} 
            onChange={e => setEmail(e.target.value)}
            placeholder="Email"
          />
          {/* SECURITY: Password visible as plain text! */}
          <input 
            type="text"  // Should be type="password"
            value={password} 
            onChange={e => setPassword(e.target.value)}
            placeholder="New Password"
          />
          <button>Save</button>
          <button onClick={() => setEditMode(false)}>Cancel</button>
        </form>
      ) : (
        <>
          {/* ISSUE: Inline styles instead of CSS */}
          <button style={{ color: 'blue' }} onClick={() => setEditMode(true)}>
            Edit
          </button>
          <button style={{ color: 'red' }} onClick={handleDelete}>
            Delete
          </button>
        </>
      )}
      
      {/* CRITICAL XSS: dangerouslySetInnerHTML with user data! */}
      <div dangerouslySetInnerHTML={{ __html: user.bio || '' }} />
    </div>
  );
}
