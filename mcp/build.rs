use std::process::Command;

fn main() {
    // Get current timestamp in ISO 8601 format
    let output = Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%S%z")
        .output()
        .expect("Failed to get timestamp");

    let timestamp = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in timestamp")
        .trim()
        .to_string();

    println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");
    println!("cargo:rerun-if-changed=build.rs");
}
