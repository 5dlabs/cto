#!/usr/bin/env python3
"""
Proper GPU provisioning with dual networks defined at creation time
Prevents the SSH bricking issue by defining both networks upfront
"""
import subprocess
import json
import os
import hashlib
import sys
import time

def ovh_call(method, path, body=None):
    """Make an OVH API call with proper authentication"""
    
    # Try environment first, then 1Password
    ak = os.environ.get('OVH_AK')
    as_ = os.environ.get('OVH_AS')
    ck = os.environ.get('OVH_CK')
    
    # If not in env, try to read from 1Password
    if not all([ak, as_, ck]):
        try:
            ak = subprocess.check_output(['op', 'read', 'op://Automation/OVH CA API/application_key'], text=True).strip()
            as_ = subprocess.check_output(['op', 'read', 'op://Automation/OVH CA API/application_secret'], text=True).strip()
            ck = subprocess.check_output(['op', 'read', 'op://Automation/OVH CA API/consumer_key'], text=True).strip()
        except:
            print("Failed to get OVH credentials from 1Password", file=sys.stderr)
            return None
    
    if not all([ak, as_, ck]):
        print("Missing OVH credentials", file=sys.stderr)
        return None
    
    ts_result = subprocess.run(
        ['curl', '-s', 'https://ca.api.ovh.com/1.0/auth/time'],
        capture_output=True, text=True
    )
    timestamp = ts_result.stdout.strip()
    
    url = f"https://ca.api.ovh.com/1.0{path}"
    sig_input = f"{as_}+{ck}+{method}+{url}+{body or ''}+{timestamp}"
    signature = f"$1${hashlib.sha1(sig_input.encode()).hexdigest()}"
    
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
    
    # Configuration
    old_instance_id = "04b7bfa7-7136-4230-84a0-10e5431ccdf5"
    flavor_id = "cf725cae-96ec-4687-b188-4ac34c53046b"  # t2-45
    image_id = "5aa86b9f-bb1c-440a-a5fa-46826e7807a3"   # Ubuntu 22.04 GRA9
    private_network_id = "pn-157353_10"
    
    print("=== Step 1: Delete old instance ===")
    print(f"Deleting {old_instance_id}...")
    result = ovh_call('DELETE', f'/cloud/project/{project_id}/instance/{old_instance_id}')
    print(json.dumps(result, indent=2) if isinstance(result, dict) else result)
    
    # Wait for deletion
    print("\nWaiting for deletion to complete...")
    time.sleep(30)
    
    print("\n=== Step 2: Discover network configuration ===")
    
    # Get public network info from existing instances
    print("Getting public network info from existing instances...")
    public_network_id = None
    
    instances = ovh_call('GET', f'/cloud/project/{project_id}/instance')
    if instances and isinstance(instances, list) and len(instances) > 0:
        for existing in instances:
            existing_id = existing.get('id')
            print(f"Checking instance {existing_id}...")
            interfaces = ovh_call('GET', f'/cloud/project/{project_id}/instance/{existing_id}/interface')
            if interfaces and isinstance(interfaces, list):
                for iface in interfaces:
                    net_id = iface.get('networkId', '')
                    ip = iface.get('ip', 'N/A')
                    print(f"  Interface: {net_id} -> {ip}")
                    # Public IPs don't start with 10.
                    if ip and not ip.startswith('10.') and not net_id.startswith('pn-'):
                        public_network_id = net_id
                        print(f"    ^^ FOUND public network: {public_network_id}")
                        break
            if public_network_id:
                break
    
    if not public_network_id:
        print("\nWARNING: Could not find public network ID!")
        print("Will try creating with private network only (OVH should add public automatically)")
    else:
        print(f"\n✓ Found public network: {public_network_id}")
    
    # Get SSH key
    print("\nGetting SSH keys...")
    ssh_keys = ovh_call('GET', f'/cloud/project/{project_id}/sshkey')
    ssh_key_id = None
    if ssh_keys and isinstance(ssh_keys, list) and len(ssh_keys) > 0:
        ssh_key_id = ssh_keys[0].get('id')
        print(f"Using SSH key: {ssh_key_id}")
    
    # Build create payload
    if public_network_id:
        print(f"\n=== Creating instance with dual networks ===")
        create_payload = {
            "name": "musetalk-gpu-1",
            "flavorId": flavor_id,
            "imageId": image_id,
            "region": "GRA9",
            "sshKeyId": ssh_key_id,
            "networks": [
                {"networkId": public_network_id, "ip": None},
                {"networkId": private_network_id, "ip": None}
            ],
            "monthlyBilling": False
        }
    else:
        print(f"\n=== Creating instance with private network only ===")
        create_payload = {
            "name": "musetalk-gpu-1",
            "flavorId": flavor_id,
            "imageId": image_id,
            "region": "GRA9",
            "sshKeyId": ssh_key_id,
            "networks": [
                {"networkId": private_network_id, "ip": None}
            ],
            "monthlyBilling": False
        }
    
    print(f"\nPayload:")
    print(json.dumps(create_payload, indent=2))
    
    result = ovh_call('POST', f'/cloud/project/{project_id}/instance', json.dumps(create_payload))
    print("\nResult:")
    print(json.dumps(result, indent=2) if isinstance(result, dict) else result)
    
    if isinstance(result, dict) and 'id' in result:
        new_id = result['id']
        print(f"\n=== New instance created: {new_id} ===")
        
        # Wait for ACTIVE status
        print("Waiting for instance to become ACTIVE...")
        for i in range(30):
            time.sleep(10)
            status = ovh_call('GET', f'/cloud/project/{project_id}/instance/{new_id}')
            if isinstance(status, dict):
                state = status.get('status', 'UNKNOWN')
                ip = status.get('ipAddresses', [{}])[0].get('ip', 'N/A') if status.get('ipAddresses') else 'N/A'
                print(f"  [{i+1}] Status: {state}, IP: {ip}")
                if state == 'ACTIVE':
                    print(f"\n✅ Instance ACTIVE at {ip}")
                    break
