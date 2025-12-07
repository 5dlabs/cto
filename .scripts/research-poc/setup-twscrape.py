#!/usr/bin/env python3
"""
Setup twscrape with Chrome cookies.
Extracts cookies from Chrome and adds them to twscrape.

Usage:
    python setup-twscrape.py
"""

import asyncio
import json
import os
import shutil
import sqlite3
import subprocess
import sys
from base64 import b64decode
from pathlib import Path

# Check for required packages
try:
    from Crypto.Cipher import AES
    from Crypto.Protocol.KDF import PBKDF2
except ImportError:
    print("Installing pycryptodome...")
    subprocess.run([sys.executable, "-m", "pip", "install", "pycryptodome"], check=True)
    from Crypto.Cipher import AES
    from Crypto.Protocol.KDF import PBKDF2

from twscrape import API


def get_chrome_encryption_key():
    """Get Chrome's encryption key from macOS Keychain."""
    try:
        # Get the key from Keychain
        result = subprocess.run(
            [
                "security", "find-generic-password",
                "-w",  # Just output the password
                "-s", "Chrome Safe Storage",
                "-a", "Chrome"
            ],
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            password = result.stdout.strip()
            # Derive the key using PBKDF2
            key = PBKDF2(password.encode(), b'saltysalt', dkLen=16, count=1003)
            return key
    except Exception as e:
        print(f"Error getting Chrome encryption key: {e}")
    return None


def decrypt_chrome_cookie(encrypted_value: bytes, key: bytes) -> str:
    """Decrypt a Chrome cookie value."""
    try:
        # Chrome prepends 'v10' or 'v11' to encrypted data
        if encrypted_value[:3] == b'v10' or encrypted_value[:3] == b'v11':
            encrypted_value = encrypted_value[3:]
            
            # AES-CBC with 16-byte IV of spaces
            iv = b' ' * 16
            cipher = AES.new(key, AES.MODE_CBC, iv)
            decrypted = cipher.decrypt(encrypted_value)
            
            # Remove padding
            padding_len = decrypted[-1]
            if isinstance(padding_len, int):
                decrypted = decrypted[:-padding_len]
            
            return decrypted.decode('utf-8', errors='ignore')
    except Exception as e:
        pass
    return ""


def extract_chrome_cookies(profile: str = "Profile 2"):
    """Extract X/Twitter cookies from Chrome."""
    chrome_dir = Path(os.path.expanduser("~/Library/Application Support/Google/Chrome"))
    cookies_db = chrome_dir / profile / "Cookies"
    
    if not cookies_db.exists():
        print(f"Cookies database not found: {cookies_db}")
        return None
    
    # Get encryption key
    key = get_chrome_encryption_key()
    if not key:
        print("Failed to get Chrome encryption key")
        return None
    
    # Copy database (Chrome may have it locked)
    temp_db = "/tmp/chrome_cookies_temp.db"
    shutil.copy2(cookies_db, temp_db)
    
    conn = sqlite3.connect(temp_db)
    cursor = conn.cursor()
    
    # Get X/Twitter cookies
    cursor.execute('''
        SELECT name, encrypted_value, host_key, path, expires_utc, is_secure, is_httponly
        FROM cookies 
        WHERE host_key LIKE '%x.com%' OR host_key LIKE '%twitter.com%'
    ''')
    
    cookies = {}
    for row in cursor.fetchall():
        name, encrypted_value, host, path, expires, secure, httponly = row
        
        # Decrypt the cookie value
        value = decrypt_chrome_cookie(encrypted_value, key)
        if value:
            cookies[name] = value
    
    conn.close()
    os.remove(temp_db)
    
    return cookies


def format_cookies_for_twscrape(cookies: dict) -> str:
    """Format cookies as a string for twscrape."""
    return "; ".join(f"{k}={v}" for k, v in cookies.items())


async def setup_twscrape(cookies_str: str):
    """Set up twscrape with cookies."""
    db_path = Path(__file__).parent / "twscrape.db"
    api = API(str(db_path))
    
    # Add account with cookies
    # Using placeholder credentials since we're using cookies
    await api.pool.add_account(
        username="x_user",
        password="not_used",
        email="not_used@example.com",
        email_password="not_used",
        cookies=cookies_str
    )
    
    print(f"✓ Account added to twscrape database: {db_path}")
    return api


async def test_api(api):
    """Test the API by fetching some data."""
    print("\nTesting API...")
    
    # Try to get logged-in user info
    try:
        me = await api.user_by_login("elonmusk")
        print(f"✓ API working! Fetched user: @{me.username} ({me.displayname})")
        return True
    except Exception as e:
        print(f"✗ API test failed: {e}")
        return False


async def main():
    print("Extracting Chrome cookies for X/Twitter...")
    print("(Using Profile 2)")
    print()
    
    cookies = extract_chrome_cookies("Profile 2")
    
    if not cookies:
        print("Failed to extract cookies")
        return
    
    # Check for required auth cookies
    required = ['auth_token', 'ct0']
    missing = [c for c in required if c not in cookies]
    
    if missing:
        print(f"Missing required cookies: {missing}")
        print(f"Found cookies: {list(cookies.keys())}")
        return
    
    print(f"✓ Extracted {len(cookies)} cookies")
    print(f"✓ Found auth cookies: {[c for c in cookies if c in ['auth_token', 'ct0', 'twid']]}")
    
    # Format for twscrape
    cookies_str = format_cookies_for_twscrape(cookies)
    
    # Save cookies to file for debugging
    cookies_file = Path(__file__).parent / "x_cookies.json"
    with open(cookies_file, "w") as f:
        json.dump(cookies, f, indent=2)
    print(f"✓ Saved cookies to {cookies_file}")
    
    # Setup twscrape
    print("\nSetting up twscrape...")
    api = await setup_twscrape(cookies_str)
    
    # Test the API
    await test_api(api)


if __name__ == "__main__":
    asyncio.run(main())

