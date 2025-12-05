#!/usr/bin/env python3
"""
Proof of Concept: X Bookmarks Scraper
Uses Playwright for browser automation and Claude Code CLI for enrichment.

Usage:
    # First time - login manually:
    python scrape-bookmarks.py login
    
    # Scrape bookmarks:
    python scrape-bookmarks.py scrape
    
    # Scrape with Claude enrichment:
    python scrape-bookmarks.py scrape --enrich
"""

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path

# Check for playwright
try:
    from playwright.sync_api import sync_playwright
except ImportError:
    print("Installing playwright...")
    subprocess.run([sys.executable, "-m", "pip", "install", "playwright"], check=True)
    subprocess.run([sys.executable, "-m", "playwright", "install", "chromium"], check=True)
    from playwright.sync_api import sync_playwright

# Paths
SCRIPT_DIR = Path(__file__).parent
SESSION_DIR = SCRIPT_DIR / ".session"
OUTPUT_DIR = SCRIPT_DIR.parent.parent / "docs" / "xposts"
SESSION_FILE = SESSION_DIR / "state.json"


def ensure_dirs():
    """Create necessary directories."""
    SESSION_DIR.mkdir(parents=True, exist_ok=True)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)


def login(timeout_seconds: int = 120):
    """Open browser for manual login, then save session."""
    ensure_dirs()
    
    print("Opening browser for X login...")
    print(f"You have {timeout_seconds} seconds to log in.")
    print("The browser will wait until you reach the bookmarks page.")
    
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=False)
        context = browser.new_context()
        page = context.new_page()
        
        page.goto("https://x.com/login")
        
        # Wait for successful login by checking for bookmarks page
        print("\nWaiting for login... Navigate to your bookmarks when done.")
        try:
            # Wait for URL to contain bookmarks (user navigates there after login)
            page.wait_for_url("**/i/bookmarks**", timeout=timeout_seconds * 1000)
            print("✓ Detected bookmarks page!")
        except Exception:
            # Alternative: wait for home timeline
            print("Timeout waiting for bookmarks. Checking if logged in...")
            page.goto("https://x.com/i/bookmarks")
            page.wait_for_timeout(3000)
        
        # Verify we're on bookmarks
        if "bookmarks" in page.url.lower():
            print("✓ Successfully logged in!")
        else:
            print("⚠ May not be fully logged in. Saving session anyway...")
        
        # Save session state
        context.storage_state(path=str(SESSION_FILE))
        print(f"\n✓ Session saved to {SESSION_FILE}")
        
        browser.close()


def scrape_bookmarks(limit: int = 20) -> list[dict]:
    """Scrape bookmarks from X using saved session."""
    if not SESSION_FILE.exists():
        print("Error: No session found. Run 'login' first.")
        sys.exit(1)
    
    bookmarks = []
    
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(storage_state=str(SESSION_FILE))
        page = context.new_page()
        
        print("Navigating to bookmarks...")
        page.goto("https://x.com/i/bookmarks")
        page.wait_for_timeout(3000)
        
        # Check if logged in
        if "login" in page.url.lower():
            print("Error: Session expired. Run 'login' again.")
            browser.close()
            sys.exit(1)
        
        print(f"Scraping up to {limit} bookmarks...")
        
        seen_ids = set()
        scroll_count = 0
        max_scrolls = 10
        
        while len(bookmarks) < limit and scroll_count < max_scrolls:
            # Find all article elements (tweets)
            articles = page.query_selector_all("article")
            
            for article in articles:
                if len(bookmarks) >= limit:
                    break
                
                try:
                    bookmark = parse_article(article, page)
                    if bookmark and bookmark["id"] not in seen_ids:
                        seen_ids.add(bookmark["id"])
                        bookmarks.append(bookmark)
                        print(f"  Found: @{bookmark['author']['handle']} - {bookmark['content'][:50]}...")
                except Exception as e:
                    print(f"  Warning: Failed to parse article: {e}")
            
            # Scroll down
            page.evaluate("window.scrollBy(0, 1000)")
            page.wait_for_timeout(1500)
            scroll_count += 1
        
        # Update session (cookies may have refreshed)
        context.storage_state(path=str(SESSION_FILE))
        browser.close()
    
    print(f"\n✓ Scraped {len(bookmarks)} bookmarks")
    return bookmarks


def parse_article(article, page) -> dict | None:
    """Parse a tweet article element into structured data."""
    # Get tweet link to extract ID
    links = article.query_selector_all("a[href*='/status/']")
    if not links:
        return None
    
    tweet_url = None
    for link in links:
        href = link.get_attribute("href")
        if "/status/" in href and not "/photo/" in href and not "/analytics" in href:
            tweet_url = f"https://x.com{href}" if href.startswith("/") else href
            break
    
    if not tweet_url:
        return None
    
    # Extract ID from URL
    tweet_id = tweet_url.split("/status/")[-1].split("?")[0].split("/")[0]
    
    # Get author info
    author_link = article.query_selector("a[href^='/'][role='link']")
    author_handle = ""
    author_name = ""
    if author_link:
        href = author_link.get_attribute("href")
        if href:
            author_handle = href.strip("/").split("/")[0]
    
    # Get display name
    name_elem = article.query_selector("[data-testid='User-Name']")
    if name_elem:
        spans = name_elem.query_selector_all("span")
        for span in spans:
            text = span.inner_text().strip()
            if text and not text.startswith("@") and not text.startswith("·"):
                author_name = text
                break
    
    # Get tweet content
    content_elem = article.query_selector("[data-testid='tweetText']")
    content = content_elem.inner_text() if content_elem else ""
    
    # Get timestamp
    time_elem = article.query_selector("time")
    timestamp = time_elem.get_attribute("datetime") if time_elem else None
    
    # Check for verified badge
    verified = article.query_selector("[data-testid='icon-verified']") is not None
    
    return {
        "id": tweet_id,
        "url": tweet_url,
        "author": {
            "handle": author_handle,
            "name": author_name,
            "verified": verified
        },
        "content": content,
        "created_at": timestamp,
        "scraped_at": datetime.utcnow().isoformat() + "Z"
    }


def enrich_with_claude(bookmark: dict) -> dict:
    """Use Claude Code CLI to enrich a bookmark with tags and summary."""
    prompt = f"""Analyze this X post and provide:
1. 3-5 relevant tags (single words, lowercase, no hashtags)
2. A one-sentence summary
3. 2-3 key takeaways as bullet points

Post by @{bookmark['author']['handle']}:
{bookmark['content']}

Respond in this exact JSON format:
{{"tags": ["tag1", "tag2"], "summary": "...", "takeaways": ["...", "..."]}}"""

    try:
        result = subprocess.run(
            ["claude", "-p", prompt],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode == 0:
            # Parse JSON from response
            response = result.stdout.strip()
            # Find JSON in response
            start = response.find("{")
            end = response.rfind("}") + 1
            if start >= 0 and end > start:
                enrichment = json.loads(response[start:end])
                bookmark["tags"] = enrichment.get("tags", [])
                bookmark["summary"] = enrichment.get("summary", "")
                bookmark["takeaways"] = enrichment.get("takeaways", [])
        else:
            print(f"  Warning: Claude enrichment failed: {result.stderr}")
    except subprocess.TimeoutExpired:
        print("  Warning: Claude enrichment timed out")
    except Exception as e:
        print(f"  Warning: Claude enrichment error: {e}")
    
    return bookmark


def to_markdown(bookmark: dict) -> str:
    """Convert bookmark to markdown format."""
    author = bookmark["author"]
    verified = " ✓" if author.get("verified") else ""
    
    md = f"""# {bookmark.get('summary', bookmark['content'][:60] + '...')}

**Author**: @{author['handle']} ({author['name']}){verified}  
**Date**: {bookmark.get('created_at', 'Unknown')}  
**URL**: {bookmark['url']}  
"""
    
    if bookmark.get("tags"):
        tags = " ".join(f"#{tag}" for tag in bookmark["tags"])
        md += f"**Tags**: {tags}\n"
    
    md += f"""
---

> {bookmark['content']}

"""
    
    if bookmark.get("takeaways"):
        md += "## Key Takeaways\n\n"
        for takeaway in bookmark["takeaways"]:
            md += f"- {takeaway}\n"
        md += "\n"
    
    md += f"""---

*Scraped: {bookmark['scraped_at']}*
"""
    
    return md


def save_bookmarks(bookmarks: list[dict], enrich: bool = False):
    """Save bookmarks to markdown files."""
    ensure_dirs()
    
    saved = []
    for bookmark in bookmarks:
        if enrich:
            print(f"  Enriching @{bookmark['author']['handle']}...")
            bookmark = enrich_with_claude(bookmark)
        
        # Generate filename
        date = datetime.utcnow().strftime("%Y-%m-%d")
        filename = f"{date}-{bookmark['id']}.md"
        filepath = OUTPUT_DIR / filename
        
        # Skip if already exists
        if filepath.exists():
            print(f"  Skipping (exists): {filename}")
            continue
        
        # Write markdown
        md = to_markdown(bookmark)
        filepath.write_text(md)
        saved.append(filepath)
        print(f"  Saved: {filename}")
    
    # Also save raw JSON
    json_file = OUTPUT_DIR / f"{datetime.utcnow().strftime('%Y-%m-%d')}-bookmarks.json"
    with open(json_file, "w") as f:
        json.dump(bookmarks, f, indent=2)
    
    print(f"\n✓ Saved {len(saved)} new bookmarks to {OUTPUT_DIR}")
    print(f"✓ Raw JSON saved to {json_file}")


def main():
    parser = argparse.ArgumentParser(description="X Bookmarks Scraper PoC")
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # Login command
    subparsers.add_parser("login", help="Open browser for manual X login")
    
    # Scrape command
    scrape_parser = subparsers.add_parser("scrape", help="Scrape bookmarks")
    scrape_parser.add_argument("--limit", type=int, default=20, help="Max bookmarks to scrape")
    scrape_parser.add_argument("--enrich", action="store_true", help="Enrich with Claude")
    
    args = parser.parse_args()
    
    if args.command == "login":
        login()
    elif args.command == "scrape":
        bookmarks = scrape_bookmarks(limit=args.limit)
        if bookmarks:
            save_bookmarks(bookmarks, enrich=args.enrich)


if __name__ == "__main__":
    main()

