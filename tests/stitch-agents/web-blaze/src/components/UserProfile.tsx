/**
 * User Profile component with intentional code smells for Stitch testing.
 */

import React, { useState, useEffect } from 'react';

// BUG: Using 'any' type defeats TypeScript's purpose
interface UserProfileProps {
  userId: any;  // Should be string | number
  onUpdate?: any;  // Should be properly typed callback
}

// BUG: Not using React.FC or proper typing
export default function UserProfile({ userId, onUpdate }: UserProfileProps) {
  // BUG: Multiple useState for related data - should use useReducer or single object
  const [user, setUser] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editMode, setEditMode] = useState(false);
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  // BUG: useEffect with no dependency array causes infinite loop risk
  // BUG: No cleanup function for async operation
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

  // BUG: Function recreated on every render - should use useCallback
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // BUG: Sending password in plain text
    // BUG: No input validation
    fetch(`/api/users/${userId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, email, password }),
    })
      .then(res => res.json())
      .then(data => {
        setUser(data);
        setEditMode(false);
        // BUG: onUpdate might be undefined - using without optional chaining
        onUpdate(data);
      })
      // BUG: Error silently caught and logged - no user feedback
      .catch(err => console.log(err));
  };

  // BUG: Inline function in onClick creates new function each render
  const handleDelete = () => {
    // BUG: No confirmation dialog before destructive action
    // BUG: Using window.confirm is blocking and bad UX
    if (window.confirm('Are you sure?')) {
      fetch(`/api/users/${userId}`, { method: 'DELETE' })
        .then(() => {
          // BUG: Hard navigation - should use router
          window.location.href = '/users';
        });
    }
  };

  if (loading) return <div>Loading...</div>;  // BUG: No loading skeleton/spinner
  if (error) return <div>Error: {error}</div>;  // BUG: No retry mechanism
  if (!user) return null;  // BUG: Should show 404 state

  return (
    <div className="user-profile">
      {/* BUG: No key prop on list items */}
      {/* BUG: Displaying sensitive data (password hash) */}
      <h1>{user.name}</h1>
      <p>Email: {user.email}</p>
      <p>Password Hash: {user.password_hash}</p>
      
      {editMode ? (
        <form onSubmit={handleSubmit}>
          {/* BUG: No label association for accessibility */}
          {/* BUG: No aria attributes */}
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
          {/* BUG: Password field visible in form */}
          <input 
            type="text"  // BUG: Should be type="password"
            value={password} 
            onChange={e => setPassword(e.target.value)}
            placeholder="New Password"
          />
          {/* BUG: Button without type attribute */}
          <button>Save</button>
          <button onClick={() => setEditMode(false)}>Cancel</button>
        </form>
      ) : (
        <>
          {/* BUG: Inline style instead of CSS class */}
          <button style={{ color: 'blue' }} onClick={() => setEditMode(true)}>
            Edit
          </button>
          {/* BUG: Destructive action with no protection */}
          <button style={{ color: 'red' }} onClick={handleDelete}>
            Delete
          </button>
        </>
      )}
      
      {/* BUG: Using dangerouslySetInnerHTML with user data - XSS vulnerability */}
      <div dangerouslySetInnerHTML={{ __html: user.bio || '' }} />
    </div>
  );
}

// BUG: Component not exported as named export for tree-shaking
// BUG: No PropTypes or runtime type checking fallback
// BUG: No error boundary wrapper
// BUG: No React.memo for performance optimization
