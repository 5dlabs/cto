/**
 * Test file for Stitch language detection and remediation buttons
 * 
 * This React/TypeScript file contains intentional issues to verify:
 * 1. Stitch detects this as TypeScript/React code
 * 2. Stitch suggests "Fix with Blaze" button (the React agent)
 */

import React, { useState, useEffect } from 'react';

interface User {
  id: number;
  email: string;
  password: string; // SECURITY: Password exposed in client-side model!
}

// ISSUE: No error boundary
export function UserProfile({ userId }: { userId: number }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // ISSUE: No abort controller for cleanup
  // ISSUE: No error handling
  useEffect(() => {
    fetch(`/api/users/${userId}`)
      .then(res => res.json())
      .then(data => {
        setUser(data);
        setLoading(false);
      });
  }, [userId]);

  // ISSUE: Renders password in DOM (even if hidden)!
  // SECURITY: XSS vulnerability - dangerouslySetInnerHTML
  const renderBio = (bio: string) => {
    return <div dangerouslySetInnerHTML={{ __html: bio }} />;
  };

  // ISSUE: No loading skeleton
  if (loading) return <div>Loading...</div>;
  
  // ISSUE: No null check UI
  if (!user) return <div>User not found</div>;

  return (
    <div className="user-profile">
      <h1>{user.email}</h1>
      {/* SECURITY: This would expose password if it existed in response */}
      <input 
        type="password" 
        value={user.password} 
        readOnly 
        // ISSUE: No aria-label for accessibility
      />
      {/* SECURITY: XSS via user-controlled content */}
      {renderBio((user as any).bio)}
      
      {/* ISSUE: Inline styles instead of CSS */}
      <button 
        style={{ backgroundColor: 'red', color: 'white' }}
        onClick={() => {
          // ISSUE: No confirmation dialog for destructive action
          // SECURITY: No CSRF protection
          fetch(`/api/users/${userId}`, { method: 'DELETE' });
        }}
      >
        Delete Account
      </button>
    </div>
  );
}

// ISSUE: No prop types validation
// ISSUE: No tests
export default UserProfile;
