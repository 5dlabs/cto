#!/usr/bin/env python3
"""
OVH GPU Node Provisioning Script
Provisions a t2-45 (V100S 32GB) in GRA9 for MuseTalk avatar rendering
"""

import hashlib
import json
import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

# Configuration
PROJECT_ID = "6093a51de65b458e8b20a7c570a4f2c1"
REGION = "GRA9"
FLAVOR = "t2-45"
IMAGE = "Ubuntu 22.04"
INSTANCE_NAME = "musetalk-gpu-1"
OVH_ENDPOINT = "https://ca.api.ovh.com/1.0"


def get_op_secret(path: str) -> str:
    """Read a secret from 1Password."""
    result = subprocess.run(
        ["op", "read", path],
        capture_output=True,
        text=True,
        check=True
    )
    return result.stdout.strip()


def ovh_call(method: str, path: str, body: str | None = None) -> dict:
    """Make an authenticated OVH API call."""
    # Get OVH credentials
    ovh_ak = get_op_secret("op://Automation/OVH CA API/application_key")
    ovh_as = get_op_secret("op://Automation/OVH CA API/application_secret")
    ovh_ck = get_op_secret("op://Automation/OVH CA API/consumer_key")

    # Get server timestamp
    ts_resp = urllib.request.urlopen(f"{OVH_ENDPOINT}/auth/time", timeout=30)
    ts = ts_resp.read().decode().strip()

    # Build signature
    sig_str = f"{ovh_as}+{ovh_ck}+{method}+{OVH_ENDPOINT}{path}+{body or ''}+{ts}"
    sig_hash = hashlib.sha1(sig_str.encode()).hexdigest()
    sig = f"$1${sig_hash}"

    # Build request
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
                return json.loads(data)
            return {}
    except urllib.error.HTTPError as e:
        error_body = e.read().decode()
        try:
            error_data = json.loads(error_body)
            raise Exception(f"HTTP {e.code}: {error_data}")
        except json.JSONDecodeError:
            raise Exception(f"HTTP {e.code}: {error_body}")


def main():
    print("Loading SSH key from 1Password...")
    try:
        ssh_key = get_op_secret("op://Automation/OVH GRA9 GPU SSH/public_key")
    except subprocess.CalledProcessError:
        print("ERROR: SSH key not found in 1Password")
        sys.exit(1)

    print(f"Provisioning GPU node: {INSTANCE_NAME}")
    print(f"Region: {REGION}, Flavor: {FLAVOR}, Image: {IMAGE}")

    # Check for existing instance
    print("Checking for existing instance...")
    try:
        instances = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/instance")
        existing = [i for i in instances if i.get("name") == INSTANCE_NAME]

        if existing:
            instance = existing[0]
            instance_id = instance["id"]
            status = instance["status"]
            print(f"Instance {INSTANCE_NAME} already exists (ID: {instance_id})")
            print(f"Current status: {status}")

            if status == "ACTIVE":
                public_ips = [ip for ip in instance.get("ipAddresses", []) if ip.get("type") == "public"]
                if public_ips:
                    ip = public_ips[0]["ip"]
                    print(f"Instance is already active. IP: {ip}")
                    print(f"INSTANCE_ID={instance_id}")
                    print(f"INSTANCE_IP={ip}")
                    sys.exit(0)
            elif status == "ERROR":
                print("Instance in ERROR state. Deleting and recreating...")
                ovh_call("DELETE", f"/cloud/project/{PROJECT_ID}/instance/{instance_id}")
                time.sleep(10)
            else:
                print("Waiting for instance to become ACTIVE...")
                for _ in range(30):
                    time.sleep(10)
                    inst = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/instance/{instance_id}")
                    status = inst.get("status")
                    print(f"Status: {status}")
                    if status == "ACTIVE":
                        public_ips = [ip for ip in inst.get("ipAddresses", []) if ip.get("type") == "public"]
                        if public_ips:
                            ip = public_ips[0]["ip"]
                            print(f"Instance is ACTIVE. IP: {ip}")
                            print(f"INSTANCE_ID={instance_id}")
                            print(f"INSTANCE_IP={ip}")
                            sys.exit(0)
                print("Timeout waiting for instance")
                sys.exit(1)
    except Exception as e:
        print(f"Error checking existing instances: {e}")

    # Get image ID - must be from GRA9 region
    print(f"Finding image ID for {IMAGE} in GRA9...")
    try:
        images = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/image")
        # Filter for GRA9 Ubuntu 22.04 image
        image = next((i for i in images if i.get("name") == IMAGE and i.get("region") == REGION), None)
        if not image:
            print(f"ERROR: Image '{IMAGE}' not found in region {REGION}")
            print("Available Ubuntu images in GRA9:")
            for i in images:
                if i.get('region') == REGION and 'ubuntu' in i.get('name', '').lower():
                    print(f"  - {i.get('name')}: {i.get('id')}")
            sys.exit(1)
        image_id = image["id"]
        print(f"Image ID: {image_id}")
    except Exception as e:
        print(f"Error finding image: {e}")
        sys.exit(1)

    # Get flavor ID
    print(f"Finding flavor ID for {FLAVOR}...")
    try:
        # Get all flavors and filter by region (case-sensitive: GRA9)
        flavors = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/flavor")
        region_flavors = [f for f in flavors if f.get("region") == REGION]
        print(f"Available {REGION} flavors:")
        for f in region_flavors[:10]:
            print(f"  {f.get('name')}: {f.get('id')}")

        flavor = next((f for f in region_flavors if f.get("name") == FLAVOR), None)
        if not flavor:
            print(f"ERROR: Flavor '{FLAVOR}' not found in region {REGION}")
            sys.exit(1)
        flavor_id = flavor["id"]
        print(f"Flavor ID: {flavor_id}")
    except Exception as e:
        print(f"Error finding flavor: {e}")
        sys.exit(1)

    # Get or create SSH key
    print("Finding SSH key...")
    try:
        keys = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/sshkey")
        key = next((k for k in keys if k.get("publicKey") == ssh_key), None)

        if key:
            key_id = key["id"]
            print(f"Using existing SSH key: {key_id}")
        else:
            print("SSH key not found, creating new key...")
            key_name = f"coder-gpu-key-{int(time.time())}"
            key_resp = ovh_call(
                "POST",
                f"/cloud/project/{PROJECT_ID}/sshkey",
                json.dumps({"name": key_name, "publicKey": ssh_key})
            )
            key_id = key_resp.get("id")
            if not key_id:
                print(f"ERROR: Failed to create SSH key. Response: {key_resp}")
                sys.exit(1)
            print(f"Created SSH key: {key_id}")
    except Exception as e:
        print(f"Error with SSH key: {e}")
        sys.exit(1)

    # Create instance
    print("Creating instance...")
    # OVH API expects specific field names
    request_data = {
        "name": INSTANCE_NAME,
        "region": REGION,
        "flavorId": flavor_id,
        "imageId": image_id,
        "sshKeyId": key_id,
        "monthlyBilling": False
    }
    body = json.dumps(request_data)
    print(f"Request body: {body}")
    print(f"Request data: {request_data}")

    try:
        create_resp = ovh_call("POST", f"/cloud/project/{PROJECT_ID}/instance", body)
        instance_id = create_resp.get("id")

        if not instance_id:
            print(f"ERROR: Failed to create instance. Response: {create_resp}")
            sys.exit(1)

        print(f"Instance created: {instance_id}")
        print("Waiting for ACTIVE status...")

        # Wait for instance to be ready
        for i in range(60):
            time.sleep(10)
            inst = ovh_call("GET", f"/cloud/project/{PROJECT_ID}/instance/{instance_id}")
            status = inst.get("status")
            print(f"Status: {status}")

            if status == "ACTIVE":
                public_ips = [ip for ip in inst.get("ipAddresses", []) if ip.get("type") == "public"]
                if public_ips:
                    ip = public_ips[0]["ip"]
                    print("Instance is ACTIVE!")
                    print(f"Instance ID: {instance_id}")
                    print(f"Public IP: {ip}")
                    print()
                    print(f"INSTANCE_ID={instance_id}")
                    print(f"INSTANCE_IP={ip}")
                    sys.exit(0)
            elif status == "ERROR":
                print("ERROR: Instance failed to start")
                sys.exit(1)

        print("ERROR: Timeout waiting for instance to become ACTIVE")
        sys.exit(1)
    except Exception as e:
        print(f"Error creating instance: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
