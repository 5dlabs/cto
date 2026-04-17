#!/usr/bin/env python3
"""Add GPU node to private network (vRack)"""

import hashlib
import json
import subprocess
import sys
import time
import urllib.request
import urllib.error

OVH_ENDPOINT = "https://ca.api.ovh.com/1.0"
PROJECT_ID = "6093a51de65b458e8b20a7c570a4f2c1"

# GPU instance ID
GPU_INSTANCE_ID = "04b7bfa7-7136-4230-84a0-10e5431ccdf5"

# Private network ID
PRIVATE_NETWORK_ID = "pn-157353_10"


def get_op_secret(path: str) -> str:
    result = subprocess.run(
        ["op", "read", path],
        capture_output=True,
        text=True,
        check=True
    )
    return result.stdout.strip()


def ovh_call(method: str, path: str, body: str | None = None) -> tuple[int, dict | str]:
    """Make an authenticated OVH API call. Returns (status_code, response)"""
    ovh_ak = get_op_secret("op://Automation/OVH CA API/application_key")
    ovh_as = get_op_secret("op://Automation/OVH CA API/application_secret")
    ovh_ck = get_op_secret("op://Automation/OVH CA API/consumer_key")

    ts_resp = urllib.request.urlopen(f"{OVH_ENDPOINT}/auth/time", timeout=30)
    ts = ts_resp.read().decode().strip()

    sig_str = f"{ovh_as}+{ovh_ck}+{method}+{OVH_ENDPOINT}{path}+{body or ''}+{ts}"
    sig_hash = hashlib.sha1(sig_str.encode()).hexdigest()
    sig = f"$1${sig_hash}"

    url = f"{OVH_ENDPOINT}{path}"
    headers = {
        "X-Ovh-Application": ovh_ak,
        "X-Ovh-Consumer": ovh_ck,
        "X-Ovh-Timestamp": ts,
        "X-Ovh-Signature": sig,
        "Content-Type": "application/json",
        "Accept": "application/json",
    }

    req = urllib.request.Request(url, method=method, headers=headers)
    if body:
        req.data = body.encode()

    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = resp.read().decode()
            if data:
                return (resp.status, json.loads(data))
            return (resp.status, {})
    except urllib.error.HTTPError as e:
        body_data = e.read().decode()
        try:
            return (e.code, json.loads(body_data))
        except:
            return (e.code, body_data)


def main():
    print("Adding GPU node to private network...")
    print(f"Instance ID: {GPU_INSTANCE_ID}")
    print(f"Private Network ID: {PRIVATE_NETWORK_ID}")
    print()

    # First, get the network details to find the subnet
    print("1. Getting private network details...")
    status, resp = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/network/private/{PRIVATE_NETWORK_ID}")
    if status != 200:
        print(f"   ERROR: Failed to get network details: {resp}")
        sys.exit(1)

    print(f"   Network: {resp.get('name')}")
    print(f"   VLAN ID: {resp.get('vlanId')}")
    regions = resp.get('regions', [])
    print(f"   Regions: {[r.get('region') for r in regions]}")

    # Find the GRA9 region subnet
    gra9_region = None
    for region in regions:
        if region.get('region') == 'GRA9':
            gra9_region = region
            break

    if not gra9_region:
        print("   ERROR: GRA9 region not found in network")
        sys.exit(1)

    print("\n2. Adding network interface to GPU instance...")
    body = json.dumps({
        "networkId": PRIVATE_NETWORK_ID
        # No IP specified - let DHCP assign one
    })

    status, resp = ovh_call(
        "POST",
        f"/cloud/project/{PROJECT_ID}/instance/{GPU_INSTANCE_ID}/interface",
        body
    )

    if status not in [200, 201, 202]:
        print(f"   ERROR: Failed to add interface: {resp}")
        sys.exit(1)

    print(f"   Success! Interface added.")
    print(f"   Response: {json.dumps(resp, indent=2)}")

    # Wait for the interface to be active
    print("\n3. Waiting for interface to be active...")
    interface_id = resp.get('id')
    max_retries = 30
    for i in range(max_retries):
        status, resp = ovh_call(
            "GET",
            f"/cloud/project/{PROJECT_ID}/instance/{GPU_INSTANCE_ID}/interface/{interface_id}"
        )
        if status == 200:
            state = resp.get('state')
            print(f"   Attempt {i+1}: Interface state = {state}")
            if state == 'ACTIVE':
                print(f"   Interface is active!")
                print(f"   IP Address: {resp.get('ip')}")
                break
        time.sleep(2)
    else:
        print("   WARNING: Interface may not be fully active yet")

    print("\n4. Done! The GPU node should now have a private IP.")
    print("   You may need to reboot the instance for the network changes to take effect.")
    print("   After reboot, the RKE2 agent should be able to reach the server on 10.0.0.181")


if __name__ == "__main__":
    main()
