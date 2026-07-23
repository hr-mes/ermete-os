#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
mkdir -p "$TMP_WORKSPACE/workspace"
WORKSPACE_REAL=$(realpath "$TMP_WORKSPACE/workspace")

# Attacker creates a symlink to /etc/passwd inside workspace
artifact_path="$TMP_WORKSPACE/workspace/forge_status_123.json"
ln -s /etc/passwd "$artifact_path"

echo "Artifact path: $artifact_path"
resolved_artifact=$(realpath "$artifact_path")
echo "Resolved artifact: $resolved_artifact"

if [[ "$resolved_artifact" == "$WORKSPACE_REAL/"* ]]; then
    echo "Check PASSED (VULNERABLE!)"
else
    echo "Check FAILED (Secure against symlink pointing outside)."
fi
