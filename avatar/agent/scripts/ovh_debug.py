#!/usr/bin/env python3
"""Debug OVH API endpoints"""

import hashlib
import json
import subprocess
import urllib.request
import urllib.error

OVH_ENDPOINT = "https://ca.api.ovh.com/1.0"
PROJECT_ID = "6093a51de65b458e8b20a7c570a4f2c1"


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
    print("Testing OVH API endpoints...")
    print()

    # Test project access
    print("1. Testing project access...")
    status, resp = ovh_call("GET", f"/cloud/project/{PROJECT_ID}")
    print(f"   Status: {status}")
    print(f"   Response: {json.dumps(resp, indent=2)[:500]}")
    print()

    # Test regions
    print("2. Testing regions endpoint...")
    status, resp = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/region")
    print(f"   Status: {status}")
    print(f"   Response: {json.dumps(resp, indent=2)[:500]}")
    print()

    # Test private networks
    print("3. Testing private networks...")
    status, resp = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/network/private")
    print(f"   Status: {status}")
    if status == 200:
        print(f"   Found {len(resp)} private networks")
        for net in resp:
            print(f"     - {net.get('name')}: {net.get('id')} (VLAN: {net.get('vlanId')})")
            print(f"       Regions: {net.get('regions', [])}")
    else:
        print(f"   Error: {resp}")
    print()

    # Test instances with IPs
    print("4. Testing instances endpoint...")
    status, resp = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/instance")
    print(f"   Status: {status}")
    if status == 200:
        print(f"   Found {len(resp)} instances")
        for inst in resp:
            ip_addresses = inst.get('ipAddresses', [])
            public_ips = [ip.get('ip') for ip in ip_addresses if ip.get('type') == 'public']
            private_ips = [ip.get('ip') for ip in ip_addresses if ip.get('type') == 'private']
            print(f"     - {inst.get('name')}: {inst.get('status')} ({inst.get('id')})")
            print(f"       Public IPs: {public_ips}")
            print(f"       Private IPs: {private_ips}")
            # Check if this is the GPU instance
            if inst.get('id') == GPU_INSTANCE_ID:
                print(f"       *** This is the GPU instance ***")
                print(f"       Instance status: {inst.get('status')}")
    else:
        print(f"   Error: {resp}")
    print()

    # Test subnets for private network
    print("5. Testing subnets for private network...")
    status, resp = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/network/private/pn-157353_10/subnet")
    print(f"   Status: {status}")
    if status == 200:
        print(f"   Found {len(resp)} subnets")
        for subnet in resp:
            print(f"     - Region: {subnet.get('region')}")
            print(f"       Network: {subnet.get('network')}")
            print(f"       Gateway: {subnet.get('gateway')}")
            print(f"       DHCP: {subnet.get('dhcp')}")
    else:
        print(f"   Error: {resp}")


GPU_INSTANCE_ID = "04b7bfa7-7136-4230-84a0-10e5431ccdf5"

if __name__ == "__main__":
    main()
