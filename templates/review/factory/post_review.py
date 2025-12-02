#!/usr/bin/env python3
"""
Stitch PR Review Poster

Reads /tmp/review.json and posts inline review comments to GitHub.
Uses position-based comments like Cursor Bot for accurate diff placement.
Falls back to posting /tmp/review.md as a simple comment if JSON is not available.
"""

import json
import os
import re
import subprocess
import sys

SEVERITY_EMOJI = {
    'critical': '🔴',
    'important': '🟠', 
    'suggestion': '🟡',
    'info': '🔵'
}


def fetch_pr_diff(repo_slug, pr_number):
    """Fetch the PR diff using gh CLI."""
    result = subprocess.run(
        ['gh', 'pr', 'diff', str(pr_number), '--repo', repo_slug],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"⚠️ Failed to fetch diff: {result.stderr}")
        return None
    return result.stdout


def parse_diff_positions(diff_text):
    """
    Parse diff to build a map of file -> line -> position.
    Position is 1-indexed from the start of each file's diff.
    """
    if not diff_text:
        return {}
    
    positions = {}  # {filepath: {line_number: position}}
    current_file = None
    position = 0
    current_new_line = 0
    
    for line in diff_text.split('\n'):
        # New file in diff
        if line.startswith('diff --git'):
            # Extract filename from "diff --git a/path b/path"
            match = re.search(r'b/(.+)$', line)
            if match:
                current_file = match.group(1)
                positions[current_file] = {}
                position = 0
        
        # Hunk header: @@ -old_start,old_count +new_start,new_count @@
        elif line.startswith('@@') and current_file:
            position += 1
            match = re.search(r'\+(\d+)', line)
            if match:
                current_new_line = int(match.group(1))
        
        # Content lines
        elif current_file and (line.startswith('+') or line.startswith('-') or line.startswith(' ')):
            position += 1
            
            # Track line numbers for added/context lines (not removed)
            if not line.startswith('-'):
                positions[current_file][current_new_line] = position
                current_new_line += 1
    
    return positions


def build_review_payload(review_data, diff_positions):
    """Build GitHub PR Review API payload from review JSON."""
    comments = []
    
    for finding in review_data.get('findings', []):
        emoji = SEVERITY_EMOJI.get(finding.get('severity', 'info'), '🔵')
        title = finding.get('title', 'Issue')
        description = finding.get('description', '')
        file_path = finding.get('file', '')
        target_line = finding.get('end_line', finding.get('start_line', 1))
        
        # Build comment body
        body_parts = [
            f"### {emoji} {title}",
            "",
            description
        ]
        
        # Add suggestion block if provided
        if finding.get('suggestion'):
            body_parts.extend([
                "",
                "```suggestion",
                finding['suggestion'],
                "```"
            ])
        
        body_parts.extend([
            "",
            "---",
            "*Reviewed by Stitch 🧵*"
        ])
        
        if not file_path:
            continue
        
        comment = {
            'path': file_path,
            'body': '\n'.join(body_parts)
        }
        
        # Try to use position (like Cursor Bot) if we have diff info
        if file_path in diff_positions and target_line in diff_positions[file_path]:
            comment['position'] = diff_positions[file_path][target_line]
            print(f"  📍 {file_path}:{target_line} -> position {comment['position']}")
        else:
            # Fallback to line-based comment
            comment['line'] = target_line
            # Add start_line for multi-line comments
            start = finding.get('start_line')
            end = finding.get('end_line')
            if start and end and start != end:
                comment['start_line'] = start
            print(f"  📍 {file_path}:{target_line} (line-based fallback)")
        
        comments.append(comment)
    
    # Build summary body
    summary_parts = [
        "## 🔍 Stitch Review",
        "",
        "### Summary",
        review_data.get('summary', 'Review complete.'),
        ""
    ]
    
    # Add positive feedback
    positive = review_data.get('positive', [])
    if positive:
        summary_parts.append("### ✅ What's Good")
        for item in positive:
            summary_parts.append(f"- {item}")
        summary_parts.append("")
    
    # Add CI analysis if present
    ci_analysis = review_data.get('ci_analysis')
    if ci_analysis:
        summary_parts.extend([
            "### CI Analysis",
            ci_analysis,
            ""
        ])
    
    summary_parts.extend([
        "---",
        "*Reviewed by Stitch 🧵 | [Docs](https://github.com/5dlabs/cto)*"
    ])
    
    return {
        'event': review_data.get('verdict', 'COMMENT'),
        'body': '\n'.join(summary_parts),
        'comments': comments
    }


def post_review(repo_slug, pr_number, token, payload):
    """Post review to GitHub using curl."""
    url = f"https://api.github.com/repos/{repo_slug}/pulls/{pr_number}/reviews"
    
    result = subprocess.run([
        'curl', '-s', '-X', 'POST',
        '-H', f'Authorization: Bearer {token}',
        '-H', 'Accept: application/vnd.github+json',
        '-H', 'X-GitHub-Api-Version: 2022-11-28',
        url,
        '-d', json.dumps(payload)
    ], capture_output=True, text=True)
    
    return result.stdout


def post_comment_fallback(repo_slug, pr_number, body):
    """Post a simple comment as fallback."""
    result = subprocess.run([
        'gh', 'pr', 'comment', str(pr_number),
        '--repo', repo_slug,
        '--body', body
    ], capture_output=True, text=True)
    
    return result.returncode == 0


def main():
    repo_slug = os.environ.get('REPO_SLUG', '')
    pr_number = os.environ.get('PR_NUMBER', '')
    token = os.environ.get('GH_TOKEN', '')
    
    if not all([repo_slug, pr_number, token]):
        print("❌ Missing required environment variables")
        sys.exit(1)
    
    # Try JSON review first
    if os.path.exists('/tmp/review.json'):
        print("📝 Processing review JSON...")
        try:
            with open('/tmp/review.json', 'r') as f:
                review_data = json.load(f)
            
            # Fetch PR diff to calculate positions (like Cursor Bot)
            print("📥 Fetching PR diff for position calculation...")
            diff_text = fetch_pr_diff(repo_slug, pr_number)
            diff_positions = parse_diff_positions(diff_text)
            print(f"📊 Parsed diff for {len(diff_positions)} files")
            
            payload = build_review_payload(review_data, diff_positions)
            
            print(f"📊 Found {len(payload['comments'])} inline comments")
            print(f"📋 Verdict: {payload['event']}")
            
            response = post_review(repo_slug, pr_number, token, payload)
            
            if '"id"' in response:
                print("✅ Review posted with inline comments")
                return 0
            else:
                print(f"⚠️ Review response: {response[:200]}")
                # Continue to try posting anyway
        except json.JSONDecodeError as e:
            print(f"⚠️ Invalid JSON in review.json: {e}")
        except Exception as e:
            print(f"⚠️ Error processing review: {e}")
    
    # Fallback to markdown
    if os.path.exists('/tmp/review.md'):
        print("📝 Posting markdown review (fallback)...")
        with open('/tmp/review.md', 'r') as f:
            body = f.read()
        
        if post_comment_fallback(repo_slug, pr_number, body):
            print("✅ Markdown review posted")
            return 0
        else:
            print("❌ Failed to post markdown review")
            return 1
    
    print("⚠️ No review file found (/tmp/review.json or /tmp/review.md)")
    return 0


if __name__ == '__main__':
    sys.exit(main())

