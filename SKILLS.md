# Available Skills

## X/Twitter Access (Bird CLI)

You have access to X/Twitter via the `bird` CLI tool. Use it for:

### Reading
```bash
bird home -n 10              # Home timeline
bird search "query" -n 10    # Search tweets
bird read <url-or-id>        # Read specific tweet
bird bookmarks -n 10         # Your bookmarks
bird user-tweets @handle     # User's tweets
```

### Account Info
```bash
bird whoami                  # Check logged-in account (@JonathonFritz)
bird check                   # Verify credentials
```

### Posting (use sparingly - can trigger rate limits)
```bash
bird tweet "text"            # Post new tweet
bird reply <id> "text"       # Reply to tweet
```

**Note**: Credentials are pre-configured. Just run commands directly.
