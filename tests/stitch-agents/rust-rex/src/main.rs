//! Stitch test fixture for Rex (Rust backend agent)
//! 
//! This file contains intentional issues for testing remediation:
//! - Missing error handling
//! - Clippy warnings
//! - Formatting issues

use axum::{routing::get, Router, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

// TODO: Intentional issue - unused variable warning
async fn get_users() -> Json<Vec<User>> {
    let unused_var = 42;
    
    let users = vec![
        User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
        User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
    ];
    
    Json(users)
}

// TODO: Intentional issue - unwrap without error handling
async fn get_user_by_id(id: i64) -> Json<User> {
    let users = vec![
        User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
    ];
    
    let user = users.into_iter().find(|u| u.id == id).unwrap();
    Json(user)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user_by_id));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
