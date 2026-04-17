#!/usr/bin/env python3
"""Fix port security on OVH network interface to allow DHCP"""

import hashlib
import json
import subprocess
import sys
import urllib.request
import urllib.error

OVH_ENDPOINT = "https://ca.api.ovh.com/1.0"
PROJECT_ID = "6093a51de65b458e8b20a7c570a4f2c1"

# GPU instance ID
GPU_INSTANCE_ID = "04b7bfa7-7136-4230-84a0-10e5431ccdf5"

# Interface ID (from previous attachment)
INTERFACE_ID = "94cdc8db-2e3a-4820-bc9d-f97d43ea5e52"


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
    print("Checking network interface details...")
    print(f"Instance ID: {GPU_INSTANCE_ID}")
    print(f"Interface ID: {INTERFACE_ID}")
    print()

    # Get interface details
    print("1. Getting interface details...")
    status, resp = ovh_call(
        "GET",
        f"/cloud/project/{PROJECT_ID}/instance/{GPU_INSTANCE_ID}/interface/{INTERFACE_ID}"
    )
    if status != 200:
        print(f"   ERROR: Failed to get interface details: {resp}")
        sys.exit(1)

    print(f"   Interface state: {resp.get('state')}")
    print(f"   IP: {resp.get('ip')}")
    print(f"   MAC: {resp.get('macAddress')}")
    print(f"   Network ID: {resp.get('networkId')}")
    print(f"   Subnet ID: {resp.get('subnetId')}")

    # Check if there's a way to disable port security
    # OVH OpenStack API might have port security settings
    print("\n2. Checking OpenStack port security settings...")

    # Try to get subnet details
    if resp.get('subnetId'):
        subnet_id = resp.get('subnetId')
        print(f"   Subnet ID: {subnet_id}")

        # Note: OVH API doesn't directly expose port security settings
        # We may need to use OpenStack CLI or Horizon
        print("\n   Note: OVH API doesn't expose port security settings directly.")
        print("   We may need to:")
        print("   a) Use OpenStack CLI to disable port security")
        print("   b) Use Horizon web interface")
        print("   c) Configure static IP instead of DHCP")

    print("\n3. Alternative: Configure static IP instead of DHCP")
    print("   Since DHCP might be blocked by port security, we can:")
    print("   a) Use the assigned IP 10.0.0.159 as static")
    print("   b) Configure netplan with static IP instead of DHCP")
    print("   c) This avoids the DHCP discovery issue entirely")

    print("\n   Recommended approach:")
    print("   Once SSH is available, configure /etc/netplan/60-ens8.yaml with:")
    print("""
network:
  version: 2
  ethernets:
    ens8:
      dhcp4: false
      dhcp6: false
      addresses:
        - 10.0.0.159/24
      routes:
        - to: 10.0.0.0/24
          via: 10.0.0.1
      optional: true
""")

    print("\nDone!")


if __name__ == "__main__":
    main()
