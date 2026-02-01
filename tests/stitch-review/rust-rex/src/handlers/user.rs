//! User API handlers with intentional code issues for Stitch review testing.

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
    pub password_hash: String,  // ISSUE: Exposing password_hash in API response
}

/// Application state
pub struct AppState {
    pub users: RwLock<Vec<User>>,
}

/// Get user by ID
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> Result<Json<User>, StatusCode> {
    let users = state.users.read().await;
    
    // ISSUE: Linear search O(n) - should use HashMap for O(1) lookup
    for user in users.iter() {
        if user.id == user_id {
            return Ok(Json(user.clone()));  // ISSUE: Returns password_hash to client
        }
    }
    
    Err(StatusCode::NOT_FOUND)
}

/// Create new user
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    // ISSUE: No input validation on email format
    // ISSUE: No password strength validation
    
    let mut users = state.users.write().await;
    
    // ISSUE: Race condition - ID generation not atomic
    let new_id = users.len() as u64 + 1;
    
    // SECURITY: Password stored in plain text - must hash with bcrypt/argon2!
    let user = User {
        id: new_id,
        name: payload.name,
        email: payload.email,
        password_hash: payload.password,  // CRITICAL: Not actually hashing!
    };
    
    users.push(user.clone());
    
    Ok(Json(user))  // ISSUE: Returns password in response
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Delete user
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<u64>,
) -> StatusCode {
    let mut users = state.users.write().await;
    
    // SECURITY: No authorization check - anyone can delete any user!
    // ISSUE: No audit logging for compliance
    
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
    
    // ISSUE: No pagination - could return millions of results
    // ISSUE: Case-sensitive search is poor UX
    
    let results: Vec<User> = users
        .iter()
        .filter(|u| u.name.contains(&query))
        .cloned()  // ISSUE: Clones include password_hash
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
    
    // SECURITY: No verification of current password!
    // ISSUE: No password strength requirements
    
    for user in users.iter_mut() {
        if user.id == user_id {
            // CRITICAL: Password stored without hashing
            user.password_hash = payload.new_password.clone();
            return StatusCode::OK;
        }
    }
    
    StatusCode::NOT_FOUND
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub new_password: String,  // ISSUE: Should require old_password verification
}

#[cfg(test)]
mod tests {
    // ISSUE: No tests - 0% coverage
}
