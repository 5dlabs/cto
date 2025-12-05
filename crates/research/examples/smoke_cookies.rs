//! Smoke test for cookie-only approach (baseline).
//!
//! Tests if we can access Twitter with just cookies and reqwest.
//!
//! Run with: cargo run --example smoke_cookies
//!
//! First, create a cookies file with your auth_token from browser DevTools:
//! echo '{"auth_token": "...", "ct0": "..."}' > .twitter-cookies.json

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};

const SESSION_FILE: &str = ".twitter-cookies.json";
const BOOKMARKS_URL: &str = "https://x.com/i/bookmarks";

#[derive(Deserialize, Serialize)]
struct TwitterCookies {
    auth_token: String,
    #[serde(default)]
    ct0: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍪 Cookie-Only Smoke Test (Baseline)\n");

    // Load cookies from file
    let cookies: TwitterCookies = match std::fs::read_to_string(SESSION_FILE) {
        Ok(s) => serde_json::from_str(&s)?,
        Err(_) => {
            println!("❌ No cookies file found at {SESSION_FILE}");
            println!("   Create it with:");
            println!("   {{\"auth_token\": \"...\", \"ct0\": \"...\"}}");
            println!("\n   Get these from browser DevTools > Application > Cookies > x.com");
            return Ok(());
        }
    };

    println!("📂 Loaded cookies from {SESSION_FILE}");
    println!(
        "   auth_token: {}...",
        &cookies.auth_token[..20.min(cookies.auth_token.len())]
    );
    if let Some(ct0) = &cookies.ct0 {
        println!("   ct0: {}...", &ct0[..20.min(ct0.len())]);
    }

    let mut headers = HeaderMap::new();

    // Build cookie string
    let cookie_str = if let Some(ct0) = &cookies.ct0 {
        format!("auth_token={}; ct0={}", cookies.auth_token, ct0)
    } else {
        format!("auth_token={}", cookies.auth_token)
    };

    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str)?);

    if let Some(ct0) = &cookies.ct0 {
        headers.insert("x-csrf-token", HeaderValue::from_str(ct0)?);
    }

    headers.insert(
        "User-Agent",
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        ),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    println!("\n🔖 Fetching bookmarks page...");
    let response = client.get(BOOKMARKS_URL).send().await?;

    println!("📊 Status: {}", response.status());
    println!("📍 Final URL: {}", response.url());

    if response.status().is_success() {
        let html = response.text().await?;
        println!("📝 Response length: {} bytes", html.len());

        // Check if we got actual content vs redirect
        if html.contains("data-testid=\"tweet\"") {
            let count = html.matches("data-testid=\"tweet\"").count();
            println!("✅ Found {count} tweets!");
        } else if html.contains("login") || html.contains("LoginForm") {
            println!("⚠️  Got redirected to login - cookies may be invalid");
        } else if html.contains("primaryColumn") {
            println!("⚠️  Got page structure but no tweets - may need JS rendering");
        } else {
            println!("❓ Got response but couldn't identify content type");
            // Print first 500 chars for debugging
            println!("   First 500 chars: {}...", &html[..500.min(html.len())]);
        }
    } else {
        println!("❌ Request failed: {}", response.status());
    }

    println!("\n🎉 Cookie-only smoke test complete!");
    println!("\n💡 Conclusion: Cookie-only approach likely won't work for bookmarks");
    println!("   Twitter's bookmarks page requires JavaScript rendering.");
    println!("   Use chromiumoxide or playwright for full browser automation.");

    Ok(())
}
