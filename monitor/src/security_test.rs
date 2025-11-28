//! Security test file - INTENTIONALLY VULNERABLE
//! This file contains security issues to test Bugbot detection
//!
//! DO NOT USE THIS CODE IN PRODUCTION

#![allow(dead_code)]
#![allow(unused_variables)]

use std::process::Command;

// SECURITY ISSUE 1: Hardcoded credentials
pub const DATABASE_PASSWORD: &str = "super_secret_password_123!";
pub const API_KEY: &str = "sk-live-abcdef123456789";
pub const AWS_SECRET_KEY: &str = "AKIAIOSFODNN7EXAMPLE/wJalrXUtnFEMI/K7MDENG/bPxRfiCY";
pub const GITHUB_TOKEN: &str = "ghp_1234567890abcdefghijklmnopqrstuvwxyz";

// SECURITY ISSUE 2: Hardcoded private key
pub const PRIVATE_KEY: &str = r#"
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0Z3VS5JJcds3xfn/ygWyF8PbnGy0AHB7MmC3W9eSRL5GtKyD
abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOP
-----END RSA PRIVATE KEY-----
"#;

/// SECURITY ISSUE 3: Command injection vulnerability
pub fn run_user_command(user_input: &str) -> Result<String, std::io::Error> {
    // BAD: Directly executing user input without sanitization
    let output = Command::new("sh")
        .arg("-c")
        .arg(user_input)  // Command injection! User can run arbitrary commands
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// SECURITY ISSUE 4: SQL injection vulnerability (simulated)
pub fn get_user_by_name(name: &str) -> String {
    // BAD: String concatenation for SQL query - allows SQL injection
    format!("SELECT * FROM users WHERE name = '{name}'")
}

/// SECURITY ISSUE 5: Path traversal vulnerability  
pub fn read_user_file(filename: &str) -> Result<String, std::io::Error> {
    // BAD: No validation of path, allows ../../../etc/passwd
    let path = format!("/data/uploads/{filename}");
    std::fs::read_to_string(path)
}

/// SECURITY ISSUE 6: Logging sensitive data
pub fn authenticate_user(username: &str, password: &str) -> bool {
    // BAD: Logging password in plaintext - exposes credentials in logs
    println!("DEBUG: Authenticating user {} with password {}", username, password);
    eprintln!("Auth attempt: user={}, pass={}", username, password);
    
    // Hardcoded admin check - another security issue
    password == "admin123"
}

/// SECURITY ISSUE 7: Insecure temporary file creation
pub fn create_temp_file(data: &str) -> Result<String, std::io::Error> {
    // BAD: Predictable temp file name, race condition vulnerability
    let path = "/tmp/myapp_temp_12345";
    std::fs::write(path, data)?;
    Ok(path.to_string())
}

/// SECURITY ISSUE 8: Unsafe deserialization (simulated)
pub fn deserialize_user_data(json_data: &str) -> Result<serde_json::Value, serde_json::Error> {
    // BAD: Deserializing untrusted data without validation
    serde_json::from_str(json_data)
}

/// SECURITY ISSUE 9: Eval-like behavior
pub fn eval_expression(expr: &str) -> String {
    // BAD: Evaluating user-provided expressions
    let output = Command::new("python3")
        .arg("-c")
        .arg(format!("print(eval('{}'))", expr))  // Code injection!
        .output();
    
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => "error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exposes_hardcoded_creds() {
        // Intentionally exposing credentials in test output
        println!("Database password: {}", DATABASE_PASSWORD);
        println!("API Key: {}", API_KEY);
        println!("AWS Key: {}", AWS_SECRET_KEY);
        assert!(!DATABASE_PASSWORD.is_empty());
    }

    #[test]
    fn test_command_injection() {
        // This would execute arbitrary commands if not careful
        let result = run_user_command("echo hello; cat /etc/passwd");
        // Intentional security issue in test
    }
}
