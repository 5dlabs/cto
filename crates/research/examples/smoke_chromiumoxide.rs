//! Smoke test for chromiumoxide browser automation.
//!
//! Tests cookie persistence and bookmark page access using pure Rust.
//!
//! Run with: cargo run --example smoke_chromiumoxide

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;

const BOOKMARKS_URL: &str = "https://x.com/i/bookmarks";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔷 Chromiumoxide Smoke Test\n");

    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head() // Show browser for manual login
            .build()?,
    )
    .await?;

    // Spawn handler task
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    println!("🌐 Launching browser...");
    let page = browser.new_page(BOOKMARKS_URL).await?;

    // Wait for page load
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Check current URL
    let url = page.url().await?.unwrap_or_default();
    println!("📍 Current URL: {url}");

    if url.contains("login") || url.contains("flow") {
        println!("⚠️  Session expired - manual login required");
        println!("   Please log in to Twitter in the browser window...");
        println!("   Press Enter when done.");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Navigate to bookmarks
        page.goto(BOOKMARKS_URL).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // Get page content
    let html = page.content().await?;
    println!("📝 Page content length: {} bytes", html.len());

    // Count tweets
    let bookmark_count = html.matches("data-testid=\"tweet\"").count();
    println!("✅ Found {} tweets on bookmarks page", bookmark_count);

    // Extract cookies via JavaScript
    let cookies_js: String = page.evaluate("document.cookie").await?.into_value()?;
    println!("🍪 Cookies: {} chars", cookies_js.len());

    // Check for auth_token
    if cookies_js.contains("auth_token") {
        println!("✅ auth_token cookie present");
    } else {
        println!("❌ auth_token cookie NOT found");
    }

    browser.close().await?;
    handle.await?;

    println!("\n🎉 Chromiumoxide smoke test complete!");
    Ok(())
}
