//! User API handlers with intentional code smells for Stitch testing.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub password_hash: String,  // BUG: Exposing password_hash in API response
}

/// Application state
pub struct AppState {
    pub users: RwLock<Vec<User>>,
}

/// Get user by ID
/// 
/// # Errors
/// Returns 404 if user not found
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> Result<Json<User>, StatusCode> {
    let users = state.users.read().await;
    
    // BUG: Linear search O(n) instead of using HashMap
    for user in users.iter() {
        if user.id == user_id {
            return Ok(Json(user.clone()));  // BUG: Cloning entire user including password
        }
    }
    
    Err(StatusCode::NOT_FOUND)
}

/// Create new user
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    // BUG: No input validation on email format
    // BUG: No password strength validation
    
    let mut users = state.users.write().await;
    
    // BUG: Race condition - ID generation not atomic
    let new_id = users.len() as u64 + 1;
    
    // BUG: Storing password in plain text (should hash)
    let user = User {
        id: new_id,
        name: payload.name,
        email: payload.email,
        password_hash: payload.password,  // BUG: Not actually hashing!
    };
    
    users.push(user.clone());
    
    Ok(Json(user))  // BUG: Returning password in response
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Delete user - DANGEROUS!
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> StatusCode {
    let mut users = state.users.write().await;
    
    // BUG: No authorization check - anyone can delete any user!
    // BUG: No audit logging
    
    let initial_len = users.len();
    users.retain(|u| u.id != user_id);
    
    if users.len() < initial_len {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Search users by name
pub async fn search_users(
    State(state): State<Arc<AppState>>,
    Path(query): Path<String>,
) -> Json<Vec<User>> {
    let users = state.users.read().await;
    
    // BUG: SQL injection risk if this were a real query
    // BUG: No pagination - could return millions of results
    // BUG: Case-sensitive search is poor UX
    
    let results: Vec<User> = users
        .iter()
        .filter(|u| u.name.contains(&query))
        .cloned()  // BUG: Cloning with password_hash exposed
        .collect();
    
    Json(results)
}

/// Update user password
pub async fn update_password(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> StatusCode {
    let mut users = state.users.write().await;
    
    // BUG: No verification of old password!
    // BUG: No password strength requirements
    // BUG: Password stored as plain text
    
    for user in users.iter_mut() {
        if user.id == user_id {
            user.password_hash = payload.new_password.clone();  // Not hashed!
            return StatusCode::OK;
        }
    }
    
    StatusCode::NOT_FOUND
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub new_password: String,  // BUG: Should require old_password too
}

#[cfg(test)]
mod tests {
    // BUG: No tests!
}
