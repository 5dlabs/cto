//! Test file to validate Stitch gh CLI integration
//!
//! This file contains intentional issues that Stitch should flag:
//! - Unused variable
//! - Missing error handling

fn main() {
    let unused_variable = 42;  // Clippy: unused variable
    
    let result = might_fail();  // Missing error handling
    println!("Result: {:?}", result);
}

fn might_fail() -> Result<i32, String> {
    Err("This always fails".to_string())
}
