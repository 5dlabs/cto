//! Intentional lint failures for testing utils alerts
//!
//! DELETE THIS FILE AFTER TESTING

#![allow(dead_code)]
#![allow(clippy::disallowed_macros)]

// Clippy: redundant clone
fn redundant_clone() {
    let s = String::from("hello");
    let _s2 = s.clone();  // s is not used after this, clone is redundant
}

// Clippy: needless return
fn needless_return(x: i32) -> i32 {
    return x + 1;  // should just be: x + 1
}

// Clippy: manual implementation of Option::map
fn manual_map(opt: Option<i32>) -> Option<i32> {
    match opt {
        Some(x) => Some(x * 2),
        None => None,
    }
}

// Clippy: useless format
fn useless_format() -> String {
    let name = "test";
    format!("{}", name)  // should just be: name.to_string()
}

// Unused variable (warning)
fn unused_var() {
    let x = 42;
    let y = 10;  // y is unused
    println!("{}", x);
}

// Clippy: len_zero instead of is_empty
fn len_zero_check(v: &Vec<i32>) -> bool {
    v.len() == 0  // should use is_empty()
}

