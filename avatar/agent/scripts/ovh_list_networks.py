#!/usr/bin/env python3
"""
OVH API helper to list private networks in the project
Uses credentials from 1Password via environment
"""
import subprocess
import json
import os
import hashlib
import sys

def ovh_call(method, path, body=None):
    """Make an OVH API call with proper authentication"""
    
    # Load credentials from environment
    ak = os.environ.get('OVH_AK')
    as_ = os.environ.get('OVH_AS')
    ck = os.environ.get('OVH_CK')
    
    if not all([ak, as_, ck]):
        print("Missing OVH credentials in environment", file=sys.stderr)
        return None
    
    # Get timestamp from OVH
    ts_result = subprocess.run(
        ['curl', '-s', 'https://ca.api.ovh.com/1.0/auth/time'],
        capture_output=True, text=True
    )
    timestamp = ts_result.stdout.strip()
    
    # Build signature
    url = f"https://ca.api.ovh.com/1.0{path}"
    sig_input = f"{as_}+{ck}+{method}+{url}+{body or ''}+{timestamp}"
    signature = f"$1${hashlib.sha1(sig_input.encode()).hexdigest()}"
    
    # Make request
    cmd = [
        'curl', '-s', '-X', method, url,
        '-H', f'X-Ovh-Application: {ak}',
        '-H', f'X-Ovh-Consumer: {ck}',
        '-H', f'X-Ovh-Timestamp: {timestamp}',
        '-H', f'X-Ovh-Signature: {signature}'
    ]
    
    if body:
        cmd.extend(['-H', 'Content-Type: application/json', '-d', body])
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    try:
        return json.loads(result.stdout)
    except:
        return result.stdout

if __name__ == '__main__':
    project_id = "6093a51de65b458e8b20a7c570a4f2c1"
    
    print("=== OVH Private Networks ===")
    networks = ovh_call('GET', f'/cloud/project/{project_id}/network/private')
    if networks:
        print(json.dumps(networks, indent=2))
    else:
        print("No networks found or error occurred")
    
    print("\n=== OVH Regions ===")
    regions = ovh_call('GET', f'/cloud/project/{project_id}/region')
    if regions:
        print(json.dumps(regions, indent=2))
