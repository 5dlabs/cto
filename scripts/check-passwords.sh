#!/bin/bash
grep -r "password:" infra/gitops --include="*.yaml" | grep -v "secretKeyRef" | grep -v "CHANGE_THIS" | grep -v "PASSWORD" && (echo "Found hardcoded password!" && exit 1) || exit 0