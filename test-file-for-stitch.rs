// Test file for Stitch inline review demo
// This file contains intentional issues for Stitch to find

use std::fs::File;
use std::io::Read;

fn main() {
    // Issue 1: Hardcoded credentials (security vulnerability)
    let api_key = "sk-1234567890abcdef";
    let password = "admin123";
    
    // Issue 2: Unwrap without error handling
    let file = File::open("config.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    
    // Issue 3: SQL injection vulnerability
    let user_input = get_user_input();
    let query = format!("SELECT * FROM users WHERE name = '{}'", user_input);
    execute_sql(&query);
    
    // Issue 4: Unused variable
    let unused_data = vec![1, 2, 3, 4, 5];
    
    // Issue 5: Magic number without explanation
    if contents.len() > 42 {
        println!("File is too large");
    }
    
    println!("API Key: {}", api_key);
    println!("Password: {}", password);
}

fn get_user_input() -> String {
    "test".to_string()
}

fn execute_sql(query: &str) {
    println!("Executing: {}", query);
}


