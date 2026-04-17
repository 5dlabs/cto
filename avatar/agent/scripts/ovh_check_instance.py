#!/usr/bin/env python3
"""
Check OVH instance status and attempt rescue mode if needed
"""
import subprocess
import json
import os
import hashlib
import sys
import time

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
    instance_id = "04b7bfa7-7136-4230-84a0-10e5431ccdf5"
    
    print("=== OVH Instance Status ===")
    status = ovh_call('GET', f'/cloud/project/{project_id}/instance/{instance_id}')
    if status:
        print(json.dumps(status, indent=2))
    else:
        print("Failed to get status")
    
    print("\n=== Instance Interfaces ===")
    interfaces = ovh_call('GET', f'/cloud/project/{project_id}/instance/{instance_id}/interface')
    if interfaces:
        print(json.dumps(interfaces, indent=2))
    else:
        print("Failed to get interfaces")
