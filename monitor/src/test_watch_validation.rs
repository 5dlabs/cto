//! Test file for E2E Watch script validation
//! This file intentionally has issues to test the Watch system
//!
//! Issues introduced:
//! - Clippy warning: unused variable
//! - Clippy warning: needless return
//! - Formatting issue (will fail fmt check)

#![allow(dead_code)]

/// A test function with intentional clippy warnings
fn test_function_with_issues() -> i32 {
    let unused_variable = 42;  // Clippy: unused variable
    
    let result = 10 + 20;
    
    return result;  // Clippy: needless return
}

/// Another function with more issues
fn another_problematic_function(x: i32) -> bool {
    // Clippy: this could be simplified
    if x > 0 {
        return true;
    } else {
        return false;
    }
}

// This line has trailing whitespace intentionally   
fn poorly_formatted(  ) {
    let     x   =   1;  // Poor formatting
    println!("{}",x);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_that_will_fail() {
        // This test is designed to fail
        assert_eq!(test_function_with_issues(), 999, "Intentional failure for testing");
    }
    
    #[test]
    fn test_that_passes() {
        assert!(another_problematic_function(5));
    }
}

