#!/usr/bin/env python3
"""Test OVH instance creation with different request formats"""

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


def ovh_call_full(method: str, path: str, body: str | None = None) -> tuple[int, dict | str]:
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
    print("Testing OVH instance creation...")
    print()

    # Get required IDs
    print("1. Getting required resources...")

    # Image ID - must be from GRA9 region
    images_resp = ovh_call_full("GET", f"/cloud/project/{PROJECT_ID}/image")
    images = images_resp[1] if isinstance(images_resp[1], list) else []
    image = next((i for i in images if i.get("name") == "Ubuntu 22.04" and i.get("region") == "GRA9"), None)
    image_id = image["id"] if image else None
    print(f"   Image ID: {image_id} (from GRA9)")

    # Flavor ID
    flavors_resp = ovh_call_full("GET", f"/cloud/project/{PROJECT_ID}/flavor")
    flavors = flavors_resp[1] if isinstance(flavors_resp[1], list) else []
    flavor = next((f for f in flavors if f.get("name") == "t2-45" and f.get("region") == "GRA9"), None)
    flavor_id = flavor["id"] if flavor else None
    print(f"   Flavor ID: {flavor_id}")

    # SSH Key
    ssh_key = get_op_secret("op://Automation/OVH GRA9 GPU SSH/public_key")
    keys_resp = ovh_call_full("GET", f"/cloud/project/{PROJECT_ID}/sshkey")
    keys = keys_resp[1] if isinstance(keys_resp[1], list) else []
    key = next((k for k in keys if k.get("publicKey") == ssh_key), None)
    if key:
        key_id = key["id"]
        print(f"   SSH Key ID: {key_id} (existing)")
    else:
        key_resp = ovh_call_full(
            "POST",
            f"/cloud/project/{PROJECT_ID}/sshkey",
            json.dumps({"name": f"test-key-{int(time.time())}", "publicKey": ssh_key})
        )[1]
        key_id = key_resp.get("id") if isinstance(key_resp, dict) else None
        print(f"   SSH Key ID: {key_id} (created)")

    print()

    # Test different request formats
    test_formats = [
        {
            "name": "Standard format",
            "body": {
                "name": "test-gpu-1",
                "region": "GRA9",
                "flavorId": flavor_id,
                "imageId": image_id,
                "sshKeyId": key_id,
                "monthlyBilling": False
            }
        },
        {
            "name": "With networks array",
            "body": {
                "name": "test-gpu-2",
                "region": "GRA9",
                "flavorId": flavor_id,
                "imageId": image_id,
                "sshKeyId": key_id,
                "monthlyBilling": False,
                "networks": []
            }
        },
        {
            "name": "Minimal format",
            "body": {
                "name": "test-gpu-3",
                "region": "GRA9",
                "flavorId": flavor_id,
                "imageId": image_id
            }
        },
    ]

    for test in test_formats:
        print(f"Testing: {test['name']}")
        body = json.dumps(test["body"])
        print(f"   Body: {body}")

        status, resp = ovh_call_full(
            "POST",
            f"/cloud/project/{PROJECT_ID}/instance",
            body
        )

        print(f"   Status: {status}")
        if status >= 400:
            print(f"   Error: {resp}")
        else:
            print(f"   Success: {resp.get('id')}")
            # Clean up
            if resp.get('id'):
                ovh_call_full("DELETE", f"/cloud/project/{PROJECT_ID}/instance/{resp['id']}")
                print("   Cleaned up test instance")
        print()


if __name__ == "__main__":
    import time
    main()
