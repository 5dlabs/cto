//! Test fixture for Rex (Rust) detection
//! This file should trigger Rex agent selection

pub fn hello_rex() -> &'static str {
    "Hello from Rex!"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hello() {
        assert_eq!(hello_rex(), "Hello from Rex!");
    }
}
