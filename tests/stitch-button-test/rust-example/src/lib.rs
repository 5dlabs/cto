//! Test file for Stitch language detection and remediation buttons
//! 
//! This Rust file contains intentional issues to verify:
//! 1. Stitch detects this as Rust code
//! 2. Stitch suggests "Fix with Rex" button (the Rust agent)

use std::collections::HashMap;

/// User data with security issues
pub struct User {
    pub id: u64,
    pub email: String,
    pub password: String, // SECURITY: Password stored in plain text!
}

/// Insecure user store
pub struct UserStore {
    users: Vec<User>, // ISSUE: O(n) lookups, should use HashMap
}

impl UserStore {
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }

    /// Find user by ID - inefficient implementation
    pub fn find(&self, id: u64) -> Option<&User> {
        // ISSUE: Linear search O(n) instead of O(1) HashMap lookup
        for user in &self.users {
            if user.id == id {
                return Some(user);
            }
        }
        None
    }

    /// Add user without validation
    pub fn add(&mut self, email: String, password: String) -> u64 {
        // ISSUE: No email format validation
        // ISSUE: No password strength check
        // SECURITY: Password not hashed!
        
        let id = self.users.len() as u64 + 1; // ISSUE: Race condition risk
        
        self.users.push(User {
            id,
            email,
            password, // CRITICAL: Plain text password storage
        });
        
        id
    }

    /// Delete without authorization check
    pub fn delete(&mut self, id: u64) -> bool {
        // SECURITY: No authorization - anyone can delete any user!
        let len_before = self.users.len();
        self.users.retain(|u| u.id != id);
        self.users.len() < len_before
    }
}

// ISSUE: No tests!
