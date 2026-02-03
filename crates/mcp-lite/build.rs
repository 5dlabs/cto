//! Build script to generate build timestamp

fn main() {
    // Get current timestamp in ISO 8601 format using only std library
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    
    // Format as ISO 8601 / RFC 3339
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    
    // Create RFC 3339 formatted timestamp
    let timestamp = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
        1970 + (secs / 31557600) as i32, // Rough year approximation
        1, // month (simplified)
        1, // day (simplified)
        (secs / 3600) as u8 % 24,
        (secs / 60) as u8 % 60,
        secs as u8 % 60,
        nanos
    );
    
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
}
